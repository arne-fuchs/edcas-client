// On native targets crossterm compiles fine — just re-export its types.
#[cfg(not(target_arch = "wasm32"))]
pub use crossterm::event::{KeyCode, KeyEvent};

// On wasm32 crossterm can't compile (no unix/windows sys layer).
// Provide minimal compatible types so the view code can stay unchanged.
#[cfg(target_arch = "wasm32")]
mod wasm_keys {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum KeyCode {
        Backspace,
        Enter,
        Left,
        Right,
        Up,
        Down,
        Home,
        End,
        PageUp,
        PageDown,
        Tab,
        BackTab,
        Delete,
        Insert,
        F(u8),
        Char(char),
        Null,
        Esc,
        CapsLock,
        ScrollLock,
        NumLock,
        PrintScreen,
        Pause,
        Menu,
        KeypadBegin,
    }

    bitflags::bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct KeyModifiers: u8 {
            const SHIFT   = 0b0000_0001;
            const CONTROL = 0b0000_0010;
            const ALT     = 0b0000_0100;
            const SUPER   = 0b0000_1000;
            const HYPER   = 0b0001_0000;
            const META    = 0b0010_0000;
            const NONE    = 0b0000_0000;
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyEvent {
        pub code: KeyCode,
        pub modifiers: KeyModifiers,
    }

    impl KeyEvent {
        pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
            Self { code, modifiers }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_keys::{KeyCode, KeyEvent, KeyModifiers};
