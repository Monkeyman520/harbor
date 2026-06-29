use std::{collections::HashMap, mem};

use anyhow::Result;
use fontdue::Font;


use crate::{
    font::load_system_font,
    render::Render,
    terminal::{Terminal, TerminalSize},
};

const FONT_SIZE: f32 = 24.0;
const TEXT_PADDING: f32 = 16.0;
const ATLAS_PADDING: u32 = 1;
const SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var text_texture: texture_2d<f32>;
@group(0) @binding(1) var text_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(text_texture, text_sampler, input.tex_coords).r;
    return vec4<f32>(0.95, 0.90, 0.80, alpha);
}
"#;

/// A GPU vertex containing clip-space position and texture coordinates.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    /// Returns the wgpu vertex buffer layout matching `Vertex` memory layout.
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    /// Builds one glyph quad from pixel-space bounds and atlas UV bounds.
    fn glyph_quad(
        surface_width: f32,
        surface_height: f32,
        left_px: f32,
        top_px: f32,
        right_px: f32,
        bottom_px: f32,
        uv: AtlasUv,
    ) -> [Self; 6] {
        let left = left_px / surface_width * 2.0 - 1.0;
        let right = right_px / surface_width * 2.0 - 1.0;
        let top = 1.0 - top_px / surface_height * 2.0;
        let bottom = 1.0 - bottom_px / surface_height * 2.0;

        [
            Self {
                position: [left, top],
                tex_coords: [uv.left, uv.top],
            },
            Self {
                position: [left, bottom],
                tex_coords: [uv.left, uv.bottom],
            },
            Self {
                position: [right, bottom],
                tex_coords: [uv.right, uv.bottom],
            },
            Self {
                position: [left, top],
                tex_coords: [uv.left, uv.top],
            },
            Self {
                position: [right, bottom],
                tex_coords: [uv.right, uv.bottom],
            },
            Self {
                position: [right, top],
                tex_coords: [uv.right, uv.top],
            },
        ]
    }
}

/// Font-derived measurements used to map window pixels to terminal cells.
#[derive(Clone, Copy)]
struct TextMetrics {
    /// Advance width of the representative monospace glyph used as a cell width.
    cell_width: f32,
    /// Full baseline-to-baseline distance including font line gap.
    line_height: f32,
    /// Baseline offset used when placing glyph rasters inside each cell.
    ascent: f32,
}

impl TextMetrics {
    fn new(font: &Font) -> Self {
        let line_metrics = font
            .horizontal_line_metrics(FONT_SIZE)
            .expect("system font should have horizontal metrics");
        let cell_width = font.metrics('M', FONT_SIZE).advance_width.ceil().max(1.0);
        let line_height = (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap)
            .ceil()
            .max(1.0);

        Self {
            cell_width,
            line_height,
            ascent: line_metrics.ascent,
        }
    }

    // Keep at least one row and column even when padding exceeds the surface size;
    // downstream terminal storage and wgpu buffers assume non-empty dimensions.
    fn terminal_size(self, width: u32, height: u32) -> TerminalSize {
        let text_width = (width as f32 - TEXT_PADDING * 2.0).max(self.cell_width);
        let text_height = (height as f32 - TEXT_PADDING * 2.0).max(self.line_height);

        TerminalSize {
            rows: (text_height / self.line_height).floor().max(1.0) as usize,
            cols: (text_width / self.cell_width).floor().max(1.0) as usize,
        }
    }
}

/// Holds the text render pipeline and size-dependent draw resources.
pub(crate) struct TextRenderer {
    font: Font,
    pipeline: wgpu::RenderPipeline,
    _bind_group_layout: wgpu::BindGroupLayout,

    // Persistent GPU resources
    vertices_buffer: wgpu::Buffer,
    max_vertices: usize,
    active_vertex_count: u32,

    atlas_texture: wgpu::Texture,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,

    atlas: GlyphAtlas,
}

impl TextRenderer {
    /// Loads the font, creates the pipeline, and prepares terminal draw data.
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        terminal: &Terminal,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let font = load_system_font()?;
        let bind_group_layout = Self::create_bind_group_layout(device);
        let pipeline = Self::create_pipeline(device, format, &bind_group_layout);

        let mut atlas = GlyphAtlas::new(&font);

        // Pre-allocate 1024x1024 texture for glyph atlas
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas texture"),
            size: wgpu::Extent3d {
                width: atlas.width,
                height: atlas.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Clear texture with zeroes
        let zeroes = vec![0u8; (atlas.width * atlas.height) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &zeroes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas.width),
                rows_per_image: Some(atlas.height),
            },
            wgpu::Extent3d {
                width: atlas.width,
                height: atlas.height,
                depth_or_array_layers: 1,
            },
        );

        // Pre-warm with ASCII characters (33..=126)
        for code in 33..=126 {
            if let Some(ch) = char::from_u32(code) {
                if let Some((glyph, bitmap, x, y)) = atlas.allocate_glyph(&font, ch) {
                    if !bitmap.is_empty() {
                        queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &atlas_texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d { x, y, z: 0 },
                                aspect: wgpu::TextureAspect::All,
                            },
                            &bitmap,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(glyph.width),
                                rows_per_image: Some(glyph.height),
                            },
                            wgpu::Extent3d {
                                width: glyph.width,
                                height: glyph.height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                }
            }
        }

        let texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph atlas sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph atlas bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Pre-allocate maximum possible capacity vertex buffer
        let max_vertices = (terminal.rows * terminal.cols * 6).max(6);
        let vertices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terminal cell vertices"),
            size: (max_vertices * mem::size_of::<Vertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            font,
            pipeline,
            _bind_group_layout: bind_group_layout,
            vertices_buffer,
            max_vertices,
            active_vertex_count: 0,
            atlas_texture,
            _sampler: sampler,
            bind_group,
            atlas,
        };

        renderer.update(device, queue, terminal, width, height);

        Ok(renderer)
    }

    pub(crate) fn terminal_size(&self, width: u32, height: u32) -> TerminalSize {
        TextMetrics::new(&self.font).terminal_size(width, height)
    }

    /// Rebuilds text resources after terminal contents or surface size changes.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        terminal: &Terminal,
        width: u32,
        height: u32,
    ) {
        let mut new_glyphs = Vec::new();
        let mut needs_eviction = false;

        // 1. Scan terminal and find characters not yet in atlas
        for r in 0..terminal.rows {
            for cell in terminal.row_cells(r) {
                if cell.ch != ' ' && !self.atlas.glyphs.contains_key(&cell.ch) {
                    if let Some((glyph, bitmap, x, y)) = self.atlas.allocate_glyph(&self.font, cell.ch) {
                        if !bitmap.is_empty() {
                            new_glyphs.push((glyph, bitmap, x, y));
                        }
                    } else {
                        needs_eviction = true;
                        break;
                    }
                }
            }
            if needs_eviction {
                break;
            }
        }

        // 2. Handle eviction if out of space
        if needs_eviction {
            let rebuild_glyphs = self.atlas.clear_and_rebuild(&self.font, terminal);

            // Clear GPU texture with zeroes
            let zeroes = vec![0u8; (self.atlas.width * self.atlas.height) as usize];
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &zeroes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.atlas.width),
                    rows_per_image: Some(self.atlas.height),
                },
                wgpu::Extent3d {
                    width: self.atlas.width,
                    height: self.atlas.height,
                    depth_or_array_layers: 1,
                },
            );

            // Upload all rebuilt glyphs
            for (glyph, bitmap, x, y) in rebuild_glyphs {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.atlas_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bitmap,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(glyph.width),
                        rows_per_image: Some(glyph.height),
                    },
                    wgpu::Extent3d {
                        width: glyph.width,
                        height: glyph.height,
                        depth_or_array_layers: 1,
                    },
                );
            }
        } else {
            // Upload new incremental glyphs
            for (glyph, bitmap, x, y) in new_glyphs {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.atlas_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bitmap,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(glyph.width),
                        rows_per_image: Some(glyph.height),
                    },
                    wgpu::Extent3d {
                        width: glyph.width,
                        height: glyph.height,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        // 3. Generate vertices
        let vertices = self.atlas.vertices(terminal, width as f32, height as f32);
        self.active_vertex_count = vertices.len() as u32;

        if !vertices.is_empty() {
            // 4. Resize vertex buffer if needed
            if vertices.len() > self.max_vertices {
                self.max_vertices = (terminal.rows * terminal.cols * 6).max(vertices.len());
                self.vertices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terminal cell vertices"),
                    size: (self.max_vertices * mem::size_of::<Vertex>()) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            // 5. Upload new vertices using write_buffer
            queue.write_buffer(&self.vertices_buffer, 0, bytemuck::cast_slice(&vertices));
        }
    }
}

impl<'pass> Render<&mut wgpu::RenderPass<'pass>> for TextRenderer {
    /// Binds the glyph atlas and issues the batched cell draw call.
    fn render(&mut self, render_pass: &mut wgpu::RenderPass<'pass>) {
        if self.active_vertex_count > 0 {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertices_buffer.slice(..));
            render_pass.draw(0..self.active_vertex_count, 0..1);
        }
    }
}

impl TextRenderer {
    /// Creates the bind layout used by the fragment shader for the atlas texture and sampler.
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph atlas bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    /// Creates the text render pipeline from the embedded WGSL shader.
    fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glyph atlas shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("glyph atlas pipeline layout"),
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("glyph atlas pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    }
}

/// Atlas placement and metrics for one rasterized glyph.
#[derive(Clone, Copy)]
struct AtlasGlyph {
    uv: AtlasUv,
    width: u32,
    height: u32,
    xmin: i32,
    ymin: i32,
}

#[derive(Clone, Copy)]
struct AtlasUv {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

/// CPU-built glyph atlas plus enough metrics to build cell quads.
struct GlyphAtlas {
    width: u32,
    height: u32,
    glyphs: HashMap<char, AtlasGlyph>,
    cell_width: f32,
    line_height: f32,
    ascent: f32,

    current_x: u32,
    current_y: u32,
    next_y: u32,
}

impl GlyphAtlas {
    fn new(font: &Font) -> Self {
        let text_metrics = TextMetrics::new(font);
        Self {
            width: 1024,
            height: 1024,
            glyphs: HashMap::new(),
            cell_width: text_metrics.cell_width,
            line_height: text_metrics.line_height,
            ascent: text_metrics.ascent,
            current_x: 0,
            current_y: 0,
            next_y: 0,
        }
    }

    fn allocate_glyph(&mut self, font: &Font, ch: char) -> Option<(AtlasGlyph, Vec<u8>, u32, u32)> {
        let (metrics, bitmap) = font.rasterize(ch, FONT_SIZE);
        let glyph_w = metrics.width as u32;
        let glyph_h = metrics.height as u32;

        if glyph_w == 0 || glyph_h == 0 {
            let glyph = AtlasGlyph {
                uv: AtlasUv {
                    left: 0.0,
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                },
                width: 0,
                height: 0,
                xmin: 0,
                ymin: 0,
            };
            self.glyphs.insert(ch, glyph);
            return Some((glyph, Vec::new(), 0, 0));
        }

        if self.current_x + glyph_w + ATLAS_PADDING > self.width {
            self.current_x = 0;
            self.current_y = self.next_y;
        }

        if self.current_y + glyph_h > self.height {
            return None;
        }

        let left = self.current_x as f32 / self.width as f32;
        let right = (self.current_x + glyph_w) as f32 / self.width as f32;
        let top = self.current_y as f32 / self.height as f32;
        let bottom = (self.current_y + glyph_h) as f32 / self.height as f32;

        let glyph = AtlasGlyph {
            uv: AtlasUv {
                left,
                top,
                right,
                bottom,
            },
            width: glyph_w,
            height: glyph_h,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
        };

        let x = self.current_x;
        let y = self.current_y;
        self.current_x += glyph_w + ATLAS_PADDING;
        if self.current_y + glyph_h + ATLAS_PADDING > self.next_y {
            self.next_y = self.current_y + glyph_h + ATLAS_PADDING;
        }

        self.glyphs.insert(ch, glyph);
        Some((glyph, bitmap, x, y))
    }

    fn clear_and_rebuild(&mut self, font: &Font, terminal: &Terminal) -> Vec<(AtlasGlyph, Vec<u8>, u32, u32)> {
        self.glyphs.clear();
        self.current_x = 0;
        self.current_y = 0;
        self.next_y = 0;

        let mut uploads = Vec::new();

        // Re-add ASCII (33..=126)
        for code in 33..=126 {
            if let Some(ch) = char::from_u32(code) {
                if let Some((glyph, bitmap, x, y)) = self.allocate_glyph(font, ch) {
                    if !bitmap.is_empty() {
                        uploads.push((glyph, bitmap, x, y));
                    }
                }
            }
        }

        // Re-add currently visible glyphs in the terminal
        for r in 0..terminal.rows {
            for cell in terminal.row_cells(r) {
                if cell.ch != ' ' && !self.glyphs.contains_key(&cell.ch) {
                    if let Some((glyph, bitmap, x, y)) = self.allocate_glyph(font, cell.ch) {
                        if !bitmap.is_empty() {
                            uploads.push((glyph, bitmap, x, y));
                        }
                    }
                }
            }
        }

        uploads
    }

    /// Converts non-empty terminal cells into one batched vertex list using atlas UVs.
    fn vertices(
        &self,
        terminal: &Terminal,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for row in 0..terminal.rows {
            let baseline = TEXT_PADDING + self.ascent.ceil() + row as f32 * self.line_height;
            for (col, cell) in terminal.row_cells(row).iter().enumerate() {
                let Some(glyph) = self.glyphs.get(&cell.ch) else {
                    continue;
                };
                if glyph.width == 0 || glyph.height == 0 {
                    continue;
                }

                let cell_x = TEXT_PADDING + col as f32 * self.cell_width;
                let glyph_left = cell_x + glyph.xmin as f32;
                let glyph_bottom = baseline - glyph.ymin as f32;
                let glyph_top = glyph_bottom - glyph.height as f32;
                let glyph_right = glyph_left + glyph.width as f32;

                vertices.extend_from_slice(&Vertex::glyph_quad(
                    surface_width,
                    surface_height,
                    glyph_left,
                    glyph_top,
                    glyph_right,
                    glyph_bottom,
                    glyph.uv,
                ));
            }
        }

        vertices
    }
}

#[cfg(test)]
mod tests {
    use super::GlyphAtlas;
    use crate::{font::load_system_font, terminal::Terminal};

    #[test]
    fn atlas_contains_each_visible_glyph_once() {
        let font = load_system_font().expect("load monospace test font");
        let mut terminal = Terminal::new(2, 5);

        terminal.put_str("aa b\nc a");
        let mut atlas = GlyphAtlas::new(&font);
        atlas.clear_and_rebuild(&font, &terminal);

        assert_eq!(atlas.glyphs.len(), 94); // All visible characters 'a', 'b', 'c' are already in the 94 printable ASCII (33..=126)
        assert!(atlas.glyphs.contains_key(&'a'));
        assert!(atlas.glyphs.contains_key(&'b'));
        assert!(atlas.glyphs.contains_key(&'c'));
        assert!(!atlas.glyphs.contains_key(&' '));
    }

    #[test]
    fn vertices_emit_one_quad_per_visible_cell() {
        let font = load_system_font().expect("load monospace test font");
        let mut terminal = Terminal::new(2, 4);

        terminal.put_str("a b\n c ");
        let mut atlas = GlyphAtlas::new(&font);
        atlas.clear_and_rebuild(&font, &terminal);
        let vertices = atlas.vertices(&terminal, 800.0, 600.0);

        assert_eq!(vertices.len(), 18);
        assert!(vertices.iter().all(|vertex| {
            vertex.position[0] >= -1.0
                && vertex.position[0] <= 1.0
                && vertex.position[1] >= -1.0
                && vertex.position[1] <= 1.0
        }));
        assert!(vertices.iter().all(|vertex| {
            vertex.tex_coords[0] >= 0.0
                && vertex.tex_coords[0] <= 1.0
                && vertex.tex_coords[1] >= 0.0
                && vertex.tex_coords[1] <= 1.0
        }));
    }

    #[test]
    fn empty_grid_builds_empty_draw_batch() {
        let font = load_system_font().expect("load monospace test font");
        let terminal = Terminal::new(2, 4);

        let mut atlas = GlyphAtlas::new(&font);
        atlas.clear_and_rebuild(&font, &terminal);
        let vertices = atlas.vertices(&terminal, 800.0, 600.0);

        assert_eq!(atlas.glyphs.len(), 94);
        assert!(vertices.is_empty());
    }
}
