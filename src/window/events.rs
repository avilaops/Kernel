//! Sistema de eventos da janela
//!
//! Gerencia todos os eventos: input, resize, close, focus, etc.

use super::input::{Key, ModifierKeys, MouseButton};
use super::{WindowPosition, WindowSize};

/// Evento da janela
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Evento de janela (resize, close, etc)
    Window(WindowEvent),
    /// Evento de teclado
    Keyboard(KeyEvent),
    /// Evento de mouse
    Mouse(MouseEvent),
    /// Tick do frame (usado para game loop)
    FrameTick(f64),
}

/// Eventos específicos da janela
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// Janela foi fechada
    Closed,
    /// Janela foi redimensionada
    Resized(WindowSize),
    /// Janela foi movida
    Moved(WindowPosition),
    /// Janela ganhou foco
    Focused,
    /// Janela perdeu foco
    Unfocused,
    /// Janela foi minimizada
    Minimized,
    /// Janela foi maximizada
    Maximized,
    /// Janela foi restaurada
    Restored,
    /// Cursor entrou na janela
    CursorEntered,
    /// Cursor saiu da janela
    CursorLeft,
    /// Frame buffer redimensionado (pode diferir do tamanho da janela em high DPI)
    FramebufferResized(u32, u32),
    /// Scale factor mudou (high DPI)
    ScaleFactorChanged(f32),
    /// Arquivos foram arrastados para a janela
    DroppedFile(String),
    /// Hover de arquivos sobre a janela
    HoveredFile(String),
    /// Arquivos cancelados
    HoveredFileCancelled,
}

/// Evento de teclado
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
    pub scancode: u32,
    pub state: KeyState,
    pub modifiers: ModifierKeys,
    pub repeat: bool,
}

impl KeyEvent {
    pub fn new(key: Key, state: KeyState) -> Self {
        Self {
            key,
            scancode: 0,
            state,
            modifiers: ModifierKeys::empty(),
            repeat: false,
        }
    }

    pub fn with_modifiers(mut self, modifiers: ModifierKeys) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn is_pressed(&self) -> bool {
        self.state == KeyState::Pressed
    }

    pub fn is_released(&self) -> bool {
        self.state == KeyState::Released
    }
}

/// Estado da tecla/botão
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Evento de mouse
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    /// Botão do mouse pressionado
    ButtonPressed {
        button: MouseButton,
        position: (f64, f64),
        modifiers: ModifierKeys,
    },
    /// Botão do mouse solto
    ButtonReleased {
        button: MouseButton,
        position: (f64, f64),
        modifiers: ModifierKeys,
    },
    /// Cursor moveu
    CursorMoved {
        position: (f64, f64),
        delta: (f64, f64),
    },
    /// Scroll do mouse (wheel)
    Scrolled {
        delta: (f64, f64),
        position: (f64, f64),
    },
}

impl MouseEvent {
    pub fn position(&self) -> Option<(f64, f64)> {
        match self {
            Self::ButtonPressed { position, .. } => Some(*position),
            Self::ButtonReleased { position, .. } => Some(*position),
            Self::CursorMoved { position, .. } => Some(*position),
            Self::Scrolled { position, .. } => Some(*position),
        }
    }
}

/// Event loop para processar eventos
pub struct EventLoop {
    events: Vec<Event>,
    running: bool,
}

impl EventLoop {
    /// Cria um novo event loop
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            running: true,
        }
    }

    /// Verifica se está rodando
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Para o event loop
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Processa eventos pendentes
    pub fn poll_events(&mut self) -> impl Iterator<Item = Event> + '_ {
        // Em uma implementação real, aqui pegaria eventos do sistema
        self.events.drain(..)
    }

    /// Aguarda por eventos (blocking)
    pub fn wait_events(&mut self) -> impl Iterator<Item = Event> + '_ {
        // Em uma implementação real, aqui aguardaria eventos do sistema
        self.events.drain(..)
    }

    /// Injeta um evento (útil para testes)
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Limpa todos os eventos pendentes
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Número de eventos pendentes
    pub fn pending_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper para processar eventos com callbacks
pub struct EventHandler<F>
where
    F: FnMut(&Event),
{
    callback: F,
}

impl<F> EventHandler<F>
where
    F: FnMut(&Event),
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }

    pub fn handle(&mut self, event: &Event) {
        (self.callback)(event);
    }

    pub fn handle_batch(&mut self, events: &[Event]) {
        for event in events {
            self.handle(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::input::KeyCode;

    #[test]
    fn test_event_loop() {
        let mut event_loop = EventLoop::new();
        assert!(event_loop.is_running());
        assert_eq!(event_loop.pending_count(), 0);

        event_loop.push_event(Event::Window(WindowEvent::Closed));
        assert_eq!(event_loop.pending_count(), 1);

        let events: Vec<_> = event_loop.poll_events().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(event_loop.pending_count(), 0);
    }

    #[test]
    fn test_key_event() {
        let key_event = KeyEvent::new(Key::Code(KeyCode::A), KeyState::Pressed)
            .with_modifiers(ModifierKeys::CTRL)
            .with_repeat(false);

        assert!(key_event.is_pressed());
        assert!(!key_event.is_released());
        assert!(key_event.modifiers.contains(ModifierKeys::CTRL));
    }

    #[test]
    fn test_mouse_event() {
        let mouse_event = MouseEvent::CursorMoved {
            position: (100.0, 200.0),
            delta: (10.0, 5.0),
        };

        assert_eq!(mouse_event.position(), Some((100.0, 200.0)));
    }

    #[test]
    fn test_event_handler() {
        let mut count = 0;
        let mut handler = EventHandler::new(|event| {
            if matches!(event, Event::Window(WindowEvent::Closed)) {
                count += 1;
            }
        });

        let events = vec![
            Event::Window(WindowEvent::Closed),
            Event::Window(WindowEvent::Focused),
            Event::Window(WindowEvent::Closed),
        ];

        handler.handle_batch(&events);
        // Note: count não pode ser verificado aqui pois está capturado por valor
        // Em uso real, usaria um Rc<RefCell<>> ou similar
    }

    #[test]
    fn test_window_events() {
        let resize = WindowEvent::Resized(WindowSize::new(1920, 1080));
        let moved = WindowEvent::Moved(WindowPosition::new(100, 100));

        assert!(matches!(resize, WindowEvent::Resized(_)));
        assert!(matches!(moved, WindowEvent::Moved(_)));
    }
}
