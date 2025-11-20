//! Sistema de input (teclado e mouse)
//!
//! Define teclas, botões do mouse e estados de input

use std::collections::HashSet;

/// Representa uma tecla ou código de tecla
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Tecla com código específico
    Code(KeyCode),
    /// Tecla com caractere Unicode
    Character(char),
}

/// Códigos de teclas (baseado em layout físico)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letras
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Números
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    // Setas
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Navegação
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,

    // Edição
    Backspace,
    Enter,
    Tab,
    Space,
    Escape,

    // Modificadores
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    MetaLeft, // Windows/Command/Super
    MetaRight,

    // Lock keys
    CapsLock,
    NumLock,
    ScrollLock,

    // Numpad
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,

    // Símbolos
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,

    // Mídia
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    VolumeUp,
    VolumeDown,
    VolumeMute,

    // Outros
    PrintScreen,
    Pause,
    ContextMenu,
}

impl KeyCode {
    /// Verifica se é uma tecla de letra
    pub fn is_letter(&self) -> bool {
        matches!(
            self,
            Self::A
                | Self::B
                | Self::C
                | Self::D
                | Self::E
                | Self::F
                | Self::G
                | Self::H
                | Self::I
                | Self::J
                | Self::K
                | Self::L
                | Self::M
                | Self::N
                | Self::O
                | Self::P
                | Self::Q
                | Self::R
                | Self::S
                | Self::T
                | Self::U
                | Self::V
                | Self::W
                | Self::X
                | Self::Y
                | Self::Z
        )
    }

    /// Verifica se é uma tecla numérica
    pub fn is_digit(&self) -> bool {
        matches!(
            self,
            Self::Key0
                | Self::Key1
                | Self::Key2
                | Self::Key3
                | Self::Key4
                | Self::Key5
                | Self::Key6
                | Self::Key7
                | Self::Key8
                | Self::Key9
        )
    }

    /// Verifica se é uma tecla de função
    pub fn is_function_key(&self) -> bool {
        matches!(
            self,
            Self::F1
                | Self::F2
                | Self::F3
                | Self::F4
                | Self::F5
                | Self::F6
                | Self::F7
                | Self::F8
                | Self::F9
                | Self::F10
                | Self::F11
                | Self::F12
                | Self::F13
                | Self::F14
                | Self::F15
                | Self::F16
                | Self::F17
                | Self::F18
                | Self::F19
                | Self::F20
                | Self::F21
                | Self::F22
                | Self::F23
                | Self::F24
        )
    }

    /// Verifica se é uma tecla modificadora
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            Self::ShiftLeft
                | Self::ShiftRight
                | Self::ControlLeft
                | Self::ControlRight
                | Self::AltLeft
                | Self::AltRight
                | Self::MetaLeft
                | Self::MetaRight
        )
    }
}

/// Botões do mouse
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

impl MouseButton {
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            3 => Self::Back,
            4 => Self::Forward,
            n => Self::Other(n),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
            Self::Back => 3,
            Self::Forward => 4,
            Self::Other(n) => *n,
        }
    }
}

/// Teclas modificadoras (Ctrl, Shift, Alt, etc)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierKeys {
    bits: u8,
}

impl ModifierKeys {
    pub const NONE: Self = Self { bits: 0 };
    pub const SHIFT: Self = Self { bits: 1 << 0 };
    pub const CTRL: Self = Self { bits: 1 << 1 };
    pub const ALT: Self = Self { bits: 1 << 2 };
    pub const META: Self = Self { bits: 1 << 3 }; // Windows/Command/Super

    pub const fn empty() -> Self {
        Self::NONE
    }

    pub const fn new(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        let mut bits = 0;
        if shift {
            bits |= Self::SHIFT.bits;
        }
        if ctrl {
            bits |= Self::CTRL.bits;
        }
        if alt {
            bits |= Self::ALT.bits;
        }
        if meta {
            bits |= Self::META.bits;
        }
        Self { bits }
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }

    pub fn remove(&mut self, other: Self) {
        self.bits &= !other.bits;
    }

    pub fn has_shift(&self) -> bool {
        self.contains(Self::SHIFT)
    }

    pub fn has_ctrl(&self) -> bool {
        self.contains(Self::CTRL)
    }

    pub fn has_alt(&self) -> bool {
        self.contains(Self::ALT)
    }

    pub fn has_meta(&self) -> bool {
        self.contains(Self::META)
    }
}

/// Estado do input (rastreia teclas e botões pressionados)
pub struct InputState {
    pressed_keys: HashSet<Key>,
    pressed_buttons: HashSet<MouseButton>,
    cursor_position: (f64, f64),
    scroll_delta: (f64, f64),
    modifiers: ModifierKeys,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_buttons: HashSet::new(),
            cursor_position: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            modifiers: ModifierKeys::empty(),
        }
    }

    /// Marca uma tecla como pressionada
    pub fn press_key(&mut self, key: Key) {
        self.pressed_keys.insert(key);
        self.update_modifiers_from_key(key, true);
    }

    /// Marca uma tecla como solta
    pub fn release_key(&mut self, key: Key) {
        self.pressed_keys.remove(&key);
        self.update_modifiers_from_key(key, false);
    }

    /// Verifica se uma tecla está pressionada
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Verifica se um código de tecla está pressionado
    pub fn is_keycode_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed_keys.contains(&Key::Code(keycode))
    }

    /// Marca um botão do mouse como pressionado
    pub fn press_button(&mut self, button: MouseButton) {
        self.pressed_buttons.insert(button);
    }

    /// Marca um botão do mouse como solto
    pub fn release_button(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
    }

    /// Verifica se um botão do mouse está pressionado
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Define a posição do cursor
    pub fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.cursor_position = (x, y);
    }

    /// Retorna a posição do cursor
    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_position
    }

    /// Define o delta do scroll
    pub fn set_scroll_delta(&mut self, x: f64, y: f64) {
        self.scroll_delta = (x, y);
    }

    /// Retorna o delta do scroll
    pub fn scroll_delta(&self) -> (f64, f64) {
        self.scroll_delta
    }

    /// Reseta o delta do scroll (deve ser chamado a cada frame)
    pub fn reset_scroll_delta(&mut self) {
        self.scroll_delta = (0.0, 0.0);
    }

    /// Retorna os modificadores atuais
    pub fn modifiers(&self) -> ModifierKeys {
        self.modifiers
    }

    /// Limpa todo o estado
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.pressed_buttons.clear();
        self.scroll_delta = (0.0, 0.0);
        self.modifiers = ModifierKeys::empty();
    }

    fn update_modifiers_from_key(&mut self, key: Key, pressed: bool) {
        if let Key::Code(keycode) = key {
            match keycode {
                KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                    if pressed {
                        self.modifiers.insert(ModifierKeys::SHIFT);
                    } else {
                        self.modifiers.remove(ModifierKeys::SHIFT);
                    }
                }
                KeyCode::ControlLeft | KeyCode::ControlRight => {
                    if pressed {
                        self.modifiers.insert(ModifierKeys::CTRL);
                    } else {
                        self.modifiers.remove(ModifierKeys::CTRL);
                    }
                }
                KeyCode::AltLeft | KeyCode::AltRight => {
                    if pressed {
                        self.modifiers.insert(ModifierKeys::ALT);
                    } else {
                        self.modifiers.remove(ModifierKeys::ALT);
                    }
                }
                KeyCode::MetaLeft | KeyCode::MetaRight => {
                    if pressed {
                        self.modifiers.insert(ModifierKeys::META);
                    } else {
                        self.modifiers.remove(ModifierKeys::META);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_categories() {
        assert!(KeyCode::A.is_letter());
        assert!(KeyCode::Key5.is_digit());
        assert!(KeyCode::F10.is_function_key());
        assert!(KeyCode::ShiftLeft.is_modifier());
    }

    #[test]
    fn test_mouse_button_index() {
        assert_eq!(MouseButton::Left.to_index(), 0);
        assert_eq!(MouseButton::Right.to_index(), 1);
        assert_eq!(MouseButton::from_index(2), MouseButton::Middle);
    }

    #[test]
    fn test_modifier_keys() {
        let mods = ModifierKeys::new(true, true, false, false);
        assert!(mods.has_shift());
        assert!(mods.has_ctrl());
        assert!(!mods.has_alt());
        assert!(!mods.has_meta());
    }

    #[test]
    fn test_input_state() {
        let mut state = InputState::new();

        state.press_key(Key::Code(KeyCode::A));
        assert!(state.is_keycode_pressed(KeyCode::A));

        state.release_key(Key::Code(KeyCode::A));
        assert!(!state.is_keycode_pressed(KeyCode::A));

        state.press_button(MouseButton::Left);
        assert!(state.is_button_pressed(MouseButton::Left));

        state.release_button(MouseButton::Left);
        assert!(!state.is_button_pressed(MouseButton::Left));
    }

    #[test]
    fn test_input_state_modifiers() {
        let mut state = InputState::new();

        state.press_key(Key::Code(KeyCode::ShiftLeft));
        assert!(state.modifiers().has_shift());

        state.press_key(Key::Code(KeyCode::ControlLeft));
        assert!(state.modifiers().has_ctrl());

        state.release_key(Key::Code(KeyCode::ShiftLeft));
        assert!(!state.modifiers().has_shift());
        assert!(state.modifiers().has_ctrl());
    }

    #[test]
    fn test_cursor_position() {
        let mut state = InputState::new();

        state.set_cursor_position(100.0, 200.0);
        assert_eq!(state.cursor_position(), (100.0, 200.0));
    }

    #[test]
    fn test_scroll_delta() {
        let mut state = InputState::new();

        state.set_scroll_delta(0.0, 10.0);
        assert_eq!(state.scroll_delta(), (0.0, 10.0));

        state.reset_scroll_delta();
        assert_eq!(state.scroll_delta(), (0.0, 0.0));
    }
}
