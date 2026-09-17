//! Configurable keybinding definitions, chord representations, and defaults.

use std::fmt;
use std::str::FromStr;

/// A keyboard key that can participate in a key chord.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Character(char),
    Tab,
    Enter,
    Space,
    Escape,
    Backspace,
    Insert,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Character(c) => write!(f, "{c}"),
            Self::Tab => write!(f, "Tab"),
            Self::Enter => write!(f, "Enter"),
            Self::Space => write!(f, "Space"),
            Self::Escape => write!(f, "Escape"),
            Self::Backspace => write!(f, "Backspace"),
            Self::Insert => write!(f, "Insert"),
            Self::Delete => write!(f, "Delete"),
            Self::ArrowUp => write!(f, "Up"),
            Self::ArrowDown => write!(f, "Down"),
            Self::ArrowLeft => write!(f, "Left"),
            Self::ArrowRight => write!(f, "Right"),
            Self::Home => write!(f, "Home"),
            Self::End => write!(f, "End"),
            Self::PageUp => write!(f, "PageUp"),
            Self::PageDown => write!(f, "PageDown"),
            Self::F(n) => write!(f, "F{n}"),
        }
    }
}

/// Active keyboard modifier flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub const fn empty() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub const fn is_empty(&self) -> bool {
        !self.ctrl && !self.alt && !self.shift && !self.meta
    }
}

/// A keyboard key plus its required modifier set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyChord {
    pub const fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn ctrl(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
        }
    }

    pub const fn ctrl_shift(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers {
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            },
        }
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.modifiers.alt {
            write!(f, "Alt+")?;
        }
        if self.modifiers.shift {
            write!(f, "Shift+")?;
        }
        if self.modifiers.meta {
            write!(f, "Meta+")?;
        }
        write!(f, "{}", self.key)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseKeyChordError(pub String);

impl fmt::Display for ParseKeyChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseKeyChordError {}

impl FromStr for KeyChord {
    type Err = ParseKeyChordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ParseKeyChordError("key chord cannot be empty".into()));
        }

        let mut modifiers = Modifiers::empty();
        let parts: Vec<&str> = trimmed.split('+').map(str::trim).collect();
        if parts.is_empty() {
            return Err(ParseKeyChordError("key chord cannot be empty".into()));
        }

        let key_str = parts.last().unwrap();
        for mod_str in &parts[..parts.len() - 1] {
            match mod_str.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "opt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "meta" | "super" | "win" | "cmd" | "command" => modifiers.meta = true,
                unknown => {
                    return Err(ParseKeyChordError(format!("unknown modifier `{unknown}`")));
                }
            }
        }

        let key = parse_key(key_str)?;
        Ok(KeyChord { key, modifiers })
    }
}

fn parse_key(s: &str) -> Result<Key, ParseKeyChordError> {
    match s.to_ascii_lowercase().as_str() {
        "tab" => Ok(Key::Tab),
        "enter" | "return" => Ok(Key::Enter),
        "space" => Ok(Key::Space),
        "esc" | "escape" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "insert" | "ins" => Ok(Key::Insert),
        "delete" | "del" => Ok(Key::Delete),
        "up" | "arrowup" => Ok(Key::ArrowUp),
        "down" | "arrowdown" => Ok(Key::ArrowDown),
        "left" | "arrowleft" => Ok(Key::ArrowLeft),
        "right" | "arrowright" => Ok(Key::ArrowRight),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "pgup" => Ok(Key::PageUp),
        "pagedown" | "pgdn" => Ok(Key::PageDown),
        f if f.starts_with('f') && f.len() >= 2 => {
            if let Ok(num) = f[1..].parse::<u8>()
                && (1..=12).contains(&num)
            {
                return Ok(Key::F(num));
            }
            Err(ParseKeyChordError(format!("unrecognized key `{s}`")))
        }
        _ => {
            let mut chars = s.chars();
            if let Some(c) = chars.next()
                && chars.next().is_none()
            {
                return Ok(Key::Character(c));
            }
            Err(ParseKeyChordError(format!("unrecognized key `{s}`")))
        }
    }
}

/// Identifies supported Harbor UI actions in the application root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiAction {
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    SelectTab(u8),
    Paste,
}

impl UiAction {
    pub const ALL: &'static [Self] = &[
        Self::NewTab,
        Self::CloseTab,
        Self::NextTab,
        Self::PreviousTab,
        Self::SelectTab(1),
        Self::SelectTab(2),
        Self::SelectTab(3),
        Self::SelectTab(4),
        Self::SelectTab(5),
        Self::SelectTab(6),
        Self::SelectTab(7),
        Self::SelectTab(8),
        Self::SelectTab(9),
        Self::Paste,
    ];

    pub fn config_name(&self) -> &'static str {
        match self {
            Self::NewTab => "new_tab",
            Self::CloseTab => "close_tab",
            Self::NextTab => "next_tab",
            Self::PreviousTab => "previous_tab",
            Self::SelectTab(1) => "select_tab_1",
            Self::SelectTab(2) => "select_tab_2",
            Self::SelectTab(3) => "select_tab_3",
            Self::SelectTab(4) => "select_tab_4",
            Self::SelectTab(5) => "select_tab_5",
            Self::SelectTab(6) => "select_tab_6",
            Self::SelectTab(7) => "select_tab_7",
            Self::SelectTab(8) => "select_tab_8",
            Self::SelectTab(9) => "select_tab_9",
            Self::SelectTab(_) => "select_tab",
            Self::Paste => "paste",
        }
    }

    pub fn from_config_name(name: &str) -> Option<Self> {
        match name {
            "new_tab" => Some(Self::NewTab),
            "close_tab" => Some(Self::CloseTab),
            "next_tab" => Some(Self::NextTab),
            "previous_tab" => Some(Self::PreviousTab),
            "select_tab_1" => Some(Self::SelectTab(1)),
            "select_tab_2" => Some(Self::SelectTab(2)),
            "select_tab_3" => Some(Self::SelectTab(3)),
            "select_tab_4" => Some(Self::SelectTab(4)),
            "select_tab_5" => Some(Self::SelectTab(5)),
            "select_tab_6" => Some(Self::SelectTab(6)),
            "select_tab_7" => Some(Self::SelectTab(7)),
            "select_tab_8" => Some(Self::SelectTab(8)),
            "select_tab_9" => Some(Self::SelectTab(9)),
            "paste" => Some(Self::Paste),
            _ => None,
        }
    }
}

/// Configurable keybinding table mapping UI actions to assigned key chords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindingSettings {
    bindings: Vec<(UiAction, KeyChord)>,
}

impl Default for KeybindingSettings {
    fn default() -> Self {
        let mut settings = Self {
            bindings: Vec::new(),
        };
        settings.bind(UiAction::NewTab, KeyChord::ctrl(Key::Character('t')));
        settings.bind(UiAction::CloseTab, KeyChord::ctrl(Key::Character('w')));
        settings.bind(UiAction::NextTab, KeyChord::ctrl(Key::Tab));
        settings.bind(UiAction::PreviousTab, KeyChord::ctrl_shift(Key::Tab));
        for i in 1..=9 {
            let digit = char::from_digit(i as u32, 10).unwrap();
            settings.bind(
                UiAction::SelectTab(i),
                KeyChord::ctrl(Key::Character(digit)),
            );
        }
        settings.bind(UiAction::Paste, KeyChord::ctrl(Key::Character('v')));
        settings
    }
}

impl KeybindingSettings {
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn bind(&mut self, action: UiAction, chord: KeyChord) {
        self.bindings
            .retain(|(a, c)| *a == action || !same_chord(*c, chord));
        if let Some(entry) = self.bindings.iter_mut().find(|(a, _)| *a == action) {
            entry.1 = chord;
        } else {
            self.bindings.push((action, chord));
        }
    }

    pub fn unbind(&mut self, action: UiAction) {
        self.bindings.retain(|(a, _)| *a != action);
    }

    pub fn chord_for(&self, action: UiAction) -> Option<KeyChord> {
        self.bindings
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, c)| *c)
    }

    pub fn action_for_chord(&self, chord: &KeyChord) -> Option<UiAction> {
        self.bindings
            .iter()
            .find(|(_, candidate)| same_chord(*candidate, *chord))
            .map(|(a, _)| *a)
    }

    pub fn bindings(&self) -> &[(UiAction, KeyChord)] {
        &self.bindings
    }
}

fn same_chord(left: KeyChord, right: KeyChord) -> bool {
    if left.modifiers != right.modifiers {
        return false;
    }
    match (left.key, right.key) {
        (Key::Character(left), Key::Character(right)) => left.eq_ignore_ascii_case(&right),
        (left, right) => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_valid_key_chords() {
        assert_eq!(
            "Ctrl+T".parse::<KeyChord>().unwrap(),
            KeyChord::ctrl(Key::Character('T'))
        );
        assert_eq!(
            "ctrl+shift+tab".parse::<KeyChord>().unwrap(),
            KeyChord::ctrl_shift(Key::Tab)
        );
        assert_eq!(
            "Ctrl+1".parse::<KeyChord>().unwrap(),
            KeyChord::ctrl(Key::Character('1'))
        );
    }

    #[test]
    fn should_fail_on_invalid_key_chords() {
        assert!("".parse::<KeyChord>().is_err());
        assert!("Ctrl+".parse::<KeyChord>().is_err());
        assert!("UnknownMod+T".parse::<KeyChord>().is_err());
        assert!("Ctrl+UnknownKeyLong".parse::<KeyChord>().is_err());
        assert!("Ctrl+F13".parse::<KeyChord>().is_err());
    }

    #[test]
    fn should_resolve_default_actions() {
        let settings = KeybindingSettings::default();
        assert_eq!(
            settings.chord_for(UiAction::NewTab),
            Some(KeyChord::ctrl(Key::Character('t')))
        );
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl(Key::Character('t'))),
            Some(UiAction::NewTab)
        );
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl(Key::Character('T'))),
            Some(UiAction::NewTab)
        );
        assert_eq!(
            settings.chord_for(UiAction::CloseTab),
            Some(KeyChord::ctrl(Key::Character('w')))
        );
        assert_eq!(
            settings.chord_for(UiAction::NextTab),
            Some(KeyChord::ctrl(Key::Tab))
        );
        assert_eq!(
            settings.chord_for(UiAction::PreviousTab),
            Some(KeyChord::ctrl_shift(Key::Tab))
        );
        assert_eq!(
            settings.chord_for(UiAction::Paste),
            Some(KeyChord::ctrl(Key::Character('v')))
        );
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl(Key::Character('v'))),
            Some(UiAction::Paste)
        );
    }

    #[test]
    fn should_override_and_unbind_actions() {
        let mut settings = KeybindingSettings::default();
        settings.bind(UiAction::NewTab, KeyChord::ctrl_shift(Key::Character('T')));
        assert_eq!(
            settings.chord_for(UiAction::NewTab),
            Some(KeyChord::ctrl_shift(Key::Character('T')))
        );
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl(Key::Character('t'))),
            None
        );
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl_shift(Key::Character('t'))),
            Some(UiAction::NewTab)
        );

        settings.unbind(UiAction::NewTab);
        assert_eq!(settings.chord_for(UiAction::NewTab), None);
    }

    #[test]
    fn should_dislodge_previous_action_when_duplicate_chord_is_bound() {
        let mut settings = KeybindingSettings::default();
        assert_eq!(
            settings.chord_for(UiAction::NewTab),
            Some(KeyChord::ctrl(Key::Character('t')))
        );
        assert_eq!(
            settings.chord_for(UiAction::CloseTab),
            Some(KeyChord::ctrl(Key::Character('w')))
        );

        // Rebind CloseTab to Ctrl+T using different character case. Character
        // case does not create a distinct chord when Shift is unchanged.
        settings.bind(UiAction::CloseTab, KeyChord::ctrl(Key::Character('T')));
        assert_eq!(
            settings.chord_for(UiAction::CloseTab),
            Some(KeyChord::ctrl(Key::Character('T')))
        );
        // NewTab has been dislodged so Ctrl+T maps unambiguously to CloseTab
        assert_eq!(settings.chord_for(UiAction::NewTab), None);
        assert_eq!(
            settings.action_for_chord(&KeyChord::ctrl(Key::Character('t'))),
            Some(UiAction::CloseTab)
        );
    }
}
