//! Declarative widget definitions for Harbor's main workspace and dialogs.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use harbor_widget::{
    Actions, Button, Column, ComponentExt as _, ConstrainedBox, Dispatcher, Expanded, Focus,
    FocusScope, IconButton, KeyChord, Row, ScrollArea, Separator, Shortcuts,
    input::event::{Key, Modifiers},
    layout::Size,
    scene::primitive::Color,
    view::{BuildCx, Component, View},
    widgets::{
        padding::Padding, preview_pane::PreviewPane, sized_box::SizedBox, text_label::TextLabel,
    },
};

use super::{RailPresentation, TabCommand, TabCommandRequest, TabUiController, TabUiState};
use crate::{
    tab_manager::{TabIndex, TabSnapshot},
    terminal_view::{TerminalDecorationPreset, TerminalWidgetBridge, terminal_widget},
};

pub const CONFIRMATION_PREVIEW_VISIBLE_LINES: usize = 12;

/// Stable application-owned inputs used to construct the main-window root.
#[derive(Clone)]
pub struct MainWindowRootInputs {
    pub controller: TabUiController,
    pub backdrop_available: bool,
    pub backdrop_fallback: [f32; 3],
    pub keybindings: harbor_config::KeybindingSettings,
}

impl MainWindowRootInputs {
    pub fn new(
        controller: TabUiController,
        backdrop_available: bool,
        backdrop_fallback: [f32; 3],
    ) -> Self {
        Self {
            controller,
            backdrop_available,
            backdrop_fallback,
            keybindings: harbor_config::KeybindingSettings::default(),
        }
    }

    pub fn with_keybindings(mut self, keybindings: harbor_config::KeybindingSettings) -> Self {
        self.keybindings = keybindings;
        self
    }
}

#[allow(dead_code)]
pub fn tab_workspace(controller: TabUiController, backdrop_available: bool) -> impl Component {
    main_window_root(MainWindowRootInputs::new(
        controller,
        backdrop_available,
        harbor_config::WindowBackdropStyle::default().fallback,
    ))
}

pub fn tab_workspace_with_fallback(
    controller: TabUiController,
    backdrop_available: bool,
    backdrop_fallback: [f32; 3],
) -> impl Component {
    main_window_root(MainWindowRootInputs::new(
        controller,
        backdrop_available,
        backdrop_fallback,
    ))
}

/// Builds the static main-window root from the same stable inputs used by HMR.
pub fn main_window_root(inputs: MainWindowRootInputs) -> impl Component {
    move |cx: &mut BuildCx| render_tab_workspace(cx, &inputs)
}

fn render_tab_workspace(cx: &mut BuildCx, props: &MainWindowRootInputs) -> View {
    let state = props.controller.store.watch(cx).clone();
    let dispatcher = props.controller.store.dispatcher();
    let action_dispatcher = dispatcher.clone();
    let active_bridge = state
        .active_bridge
        .clone()
        .expect("workspace has an active terminal until window exit");
    let root = root_padding(props.backdrop_available, props.backdrop_fallback);
    let shortcuts = shortcuts(&props.keybindings);
    let actions = Actions::handler(move |request| action_dispatcher.dispatch(request));

    harbor_widget::view! { cx; root => {
        FocusScope::new() => {
            actions => {
                shortcuts => {
                    Row::new() => {
                        tab_rail(cx, &state, &props.controller, dispatcher);
                        Separator::vertical();
                        terminal_panel(cx, &props.controller, active_bridge);
                    }
                }
            }
        }
    } }
}

fn tab_rail(
    cx: &mut BuildCx,
    state: &TabUiState,
    controller: &TabUiController,
    dispatcher: Dispatcher<TabCommandRequest>,
) -> View {
    let new_dispatcher = dispatcher.clone();
    harbor_widget::view! { cx;
        ConstrainedBox::new()
            .min_width(state.presentation.width())
            .max_width(state.presentation.width()) => {
            Column::new() => {
                Expanded::new() => {
                    ScrollArea::new().controller(controller.scroll.clone()) => {
                        Column::new() => {
                            for snapshot in state.snapshots.iter() {
                                tab_row(
                                    cx,
                                    snapshot,
                                    state.presentation,
                                    controller,
                                    dispatcher.clone(),
                                );
                            }
                        }
                    }
                }
                match state.presentation {
                    RailPresentation::Expanded => {
                        Button::new("New terminal").on_click(move |_| {
                            new_dispatcher.dispatch(TabCommandRequest::rail(TabCommand::New));
                        });
                    },
                    RailPresentation::Compact => {
                        IconButton::new("+", "New terminal").on_click(move |_| {
                            new_dispatcher.dispatch(TabCommandRequest::rail(TabCommand::New));
                        });
                    }
                }
            }
        }
    }
}

fn tab_row(
    cx: &mut BuildCx,
    snapshot: &TabSnapshot,
    presentation: RailPresentation,
    controller: &TabUiController,
    dispatcher: Dispatcher<TabCommandRequest>,
) -> View {
    harbor_widget::view! { cx;
        Row::new().keyed(format!("terminal-tab-{}", snapshot.id)) => {
            Expanded::new() => {
                Focus::empty()
                    .handle(controller
                        .tab_focus(snapshot.id)
                        .expect("live tab has a focus handle")) => {
                    select_tab_button(snapshot, presentation, dispatcher.clone());
                }
            }
            close_tab_button(snapshot, dispatcher);
        }
    }
}

fn terminal_panel(
    cx: &mut BuildCx,
    controller: &TabUiController,
    active_bridge: TerminalWidgetBridge,
) -> View {
    harbor_widget::view! { cx; Expanded::new() => {
        controller.allocation.observer() => {
            TerminalDecorationPreset::container() => {
                Focus::empty().handle(controller.terminal_focus) => {
                    terminal_widget(active_bridge);
                }
            }
        }
    } }
}

/// Main-window root styling without attaching widget children by hand.
pub(crate) fn root_padding(backdrop_available: bool, fallback_rgb: [f32; 3]) -> Padding {
    // Product/native colors are sRGB; widget colors feed a linear-light shader.
    let fallback = fallback_rgb.map(|channel| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    });
    if backdrop_available {
        Padding::all(4.0)
    } else {
        Padding::all(4.0).background(Color {
            r: fallback[0],
            g: fallback[1],
            b: fallback[2],
            a: 1.0,
        })
    }
}

#[derive(Clone)]
struct ConfirmationDialogProps {
    header_text: String,
    wrapped_lines: Vec<String>,
    scroll_offset: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    confirmed: Arc<AtomicBool>,
    line_height: f32,
}

fn confirmation_dialog(cx: &mut BuildCx, props: &ConfirmationDialogProps) -> View {
    let cancelled = Arc::clone(&props.cancelled);
    let confirmed = Arc::clone(&props.confirmed);
    harbor_widget::view! { cx; FocusScope::new() => {
        Padding::new(24.0, 16.0, 24.0, 16.0) => {
            Column::new() => {
                TextLabel::new(props.header_text.clone());
                SizedBox::new(Size::new(0.0, 8.0));
                PreviewPane::new(
                    props.wrapped_lines.clone(),
                    Arc::clone(&props.scroll_offset),
                    props.line_height,
                    CONFIRMATION_PREVIEW_VISIBLE_LINES,
                );
                SizedBox::new(Size::new(0.0, 12.0));
                Row::new() => {
                    Button::new("Cancel").on_click(move |_| {
                        cancelled.store(true, Ordering::SeqCst);
                    });
                    SizedBox::new(Size::new(12.0, 0.0));
                    Button::new("Paste").on_click(move |_| {
                        confirmed.store(true, Ordering::SeqCst);
                    });
                }
            }
        }
    } }
}

pub fn build_confirmation_root(
    line_count: usize,
    wrapped_lines: Vec<String>,
    scroll_offset: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    confirmed: Arc<AtomicBool>,
    line_height: f32,
) -> impl Component {
    let props = ConfirmationDialogProps {
        header_text: format!("Paste {line_count} lines?"),
        wrapped_lines,
        scroll_offset,
        cancelled,
        confirmed,
        line_height,
    };
    move |cx: &mut BuildCx| confirmation_dialog(cx, &props)
}

fn to_widget_chord(chord: harbor_config::KeyChord) -> Option<KeyChord> {
    let key = match chord.key {
        harbor_config::Key::Tab => Key::Tab,
        harbor_config::Key::Enter => Key::Enter,
        harbor_config::Key::Space => Key::Space,
        harbor_config::Key::Escape => Key::Escape,
        harbor_config::Key::Backspace => Key::Backspace,
        harbor_config::Key::Insert => Key::Insert,
        harbor_config::Key::Delete => Key::Delete,
        harbor_config::Key::ArrowUp => Key::ArrowUp,
        harbor_config::Key::ArrowDown => Key::ArrowDown,
        harbor_config::Key::ArrowLeft => Key::ArrowLeft,
        harbor_config::Key::ArrowRight => Key::ArrowRight,
        harbor_config::Key::Home => Key::Home,
        harbor_config::Key::End => Key::End,
        harbor_config::Key::PageUp => Key::PageUp,
        harbor_config::Key::PageDown => Key::PageDown,
        harbor_config::Key::Character(c) => Key::Character(c),
        harbor_config::Key::F(1) => Key::F1,
        harbor_config::Key::F(2) => Key::F2,
        harbor_config::Key::F(3) => Key::F3,
        harbor_config::Key::F(4) => Key::F4,
        harbor_config::Key::F(5) => Key::F5,
        harbor_config::Key::F(6) => Key::F6,
        harbor_config::Key::F(7) => Key::F7,
        harbor_config::Key::F(8) => Key::F8,
        harbor_config::Key::F(9) => Key::F9,
        harbor_config::Key::F(10) => Key::F10,
        harbor_config::Key::F(11) => Key::F11,
        harbor_config::Key::F(12) => Key::F12,
        _ => return None,
    };
    let modifiers = Modifiers {
        shift: chord.modifiers.shift,
        ctrl: chord.modifiers.ctrl,
        alt: chord.modifiers.alt,
        meta: chord.modifiers.meta,
    };
    Some(KeyChord::new(key, modifiers))
}

fn shortcuts(bindings: &harbor_config::KeybindingSettings) -> Shortcuts<TabCommandRequest> {
    let mut shortcuts = Shortcuts::empty();

    if let Some(chord) = bindings
        .chord_for(harbor_config::UiAction::NewTab)
        .and_then(to_widget_chord)
    {
        shortcuts = shortcuts.bind(chord, TabCommandRequest::shortcut(TabCommand::New));
    }
    if let Some(chord) = bindings
        .chord_for(harbor_config::UiAction::CloseTab)
        .and_then(to_widget_chord)
    {
        shortcuts = shortcuts.bind(chord, TabCommandRequest::shortcut(TabCommand::CloseActive));
    }
    if let Some(chord) = bindings
        .chord_for(harbor_config::UiAction::NextTab)
        .and_then(to_widget_chord)
    {
        shortcuts = shortcuts.bind(chord, TabCommandRequest::shortcut(TabCommand::Next));
    }
    if let Some(chord) = bindings
        .chord_for(harbor_config::UiAction::PreviousTab)
        .and_then(to_widget_chord)
    {
        shortcuts = shortcuts.bind(chord, TabCommandRequest::shortcut(TabCommand::Previous));
    }
    for index in 1..=9 {
        if let Some(chord) = bindings
            .chord_for(harbor_config::UiAction::SelectTab(index))
            .and_then(to_widget_chord)
        {
            shortcuts = shortcuts.bind(
                chord,
                TabCommandRequest::shortcut(TabCommand::Numeric(TabIndex::from_valid_u8(index))),
            );
        }
    }

    shortcuts
}

#[cfg(test)]
pub(super) fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

fn select_tab_button(
    snapshot: &TabSnapshot,
    presentation: RailPresentation,
    dispatcher: Dispatcher<TabCommandRequest>,
) -> IconButton {
    let id = snapshot.id;
    let label = tab_label(snapshot);
    let glyph = match presentation {
        RailPresentation::Expanded => label.clone(),
        RailPresentation::Compact => abbreviation(snapshot),
    };
    IconButton::new(glyph, label)
        .selected(snapshot.active)
        .on_click(move |_| {
            dispatcher.dispatch(TabCommandRequest::rail(TabCommand::Activate(id)));
        })
}

fn close_tab_button(
    snapshot: &TabSnapshot,
    dispatcher: Dispatcher<TabCommandRequest>,
) -> IconButton {
    let id = snapshot.id;
    IconButton::new("×", format!("Close {}", snapshot.title)).on_click(move |_| {
        dispatcher.dispatch(TabCommandRequest::rail(TabCommand::Close(id)));
    })
}

pub(super) fn tab_label(snapshot: &TabSnapshot) -> String {
    if snapshot.unread {
        format!("• {}", snapshot.title)
    } else {
        snapshot.title.clone()
    }
}

pub(super) fn abbreviation(snapshot: &TabSnapshot) -> String {
    if snapshot.unread {
        "•".to_owned()
    } else {
        (snapshot.id.get() % 10).to_string()
    }
}
