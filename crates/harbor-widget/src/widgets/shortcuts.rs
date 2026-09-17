use crate::input::event::{Key, Modifiers};
use crate::layout::{BoxConstraints, Point, Size};
use crate::view::{ActionProviderView, AnyView, BuildCx, Component, View};
use std::any::Any;

/// A keyboard key plus its complete modifier set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyChord {
    pub const fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    fn matches(self, other: Self) -> bool {
        if self.modifiers != other.modifiers {
            return false;
        }
        match (self.key, other.key) {
            (Key::Character(left), Key::Character(right)) => left.eq_ignore_ascii_case(&right),
            (left, right) => left == right,
        }
    }
}

/// Maps key chords to typed action values for one descendant subtree.
#[derive(Clone)]
pub struct Shortcuts<A: Clone + 'static> {
    bindings: Vec<(KeyChord, A)>,
    child: Option<View>,
}

impl<A: Clone + 'static> Shortcuts<A> {
    /// Creates an empty shortcut map for declarative `view!` composition.
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
            child: None,
        }
    }

    pub fn new(child: impl crate::IntoChildView) -> Self {
        Self {
            bindings: Vec::new(),
            child: Some(child.into_child_view()),
        }
    }

    pub fn bind(mut self, chord: KeyChord, action: A) -> Self {
        self.bindings.push((chord, action));
        self
    }
}

impl<A: Clone + 'static> Component for Shortcuts<A> {
    fn build(&self, _cx: &mut BuildCx) -> View {
        View::new(self.clone(), self.child.iter().cloned().collect(), None)
    }
}

impl<A: Clone + 'static> crate::WithChildren for Shortcuts<A> {
    fn with_children(
        mut self,
        children: crate::Children,
    ) -> Result<Self, crate::ChildConstructionError> {
        children.into_single("Shortcuts", &mut self.child)?;
        Ok(self)
    }
}

impl<A: Clone + 'static> AnyView for Shortcuts<A> {
    fn layout_children(
        &self,
        constraints: BoxConstraints,
        child_sizes: &[Size],
        _metrics: &crate::text::TextMetrics,
    ) -> (Size, Vec<Point>) {
        let size = constraints.constrain(child_sizes.first().copied().unwrap_or(Size::ZERO));
        (size, vec![Point::ZERO; child_sizes.len()])
    }

    fn as_action_provider(&self) -> Option<&dyn ActionProviderView> {
        Some(self)
    }
}

impl<A: Clone + 'static> ActionProviderView for Shortcuts<A> {
    fn shortcut_action(&self, chord: KeyChord) -> Option<Box<dyn Any>> {
        self.bindings
            .iter()
            .find(|(binding, _)| binding.matches(chord))
            .map(|(_, action)| Box::new(action.clone()) as Box<dyn Any>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_chords_ignore_ascii_case_but_not_modifiers() {
        let ctrl_lower = KeyChord::new(
            Key::Character('q'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        let ctrl_upper = KeyChord::new(
            Key::Character('Q'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        let ctrl_shift_upper = KeyChord::new(
            Key::Character('Q'),
            Modifiers {
                ctrl: true,
                shift: true,
                ..Modifiers::default()
            },
        );

        assert!(ctrl_lower.matches(ctrl_upper));
        assert!(!ctrl_lower.matches(ctrl_shift_upper));
    }
}
