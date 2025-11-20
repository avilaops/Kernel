//! Window system abstraction
//!
//! Sistema completo de janelas com suporte a:
//! - Criação e gerenciamento de janelas
//! - Fullscreen e modos de display
//! - Eventos (input, resize, close, etc)
//! - Input de teclado e mouse
//! - Cursor management
//! - Multi-monitor support

use std::fmt;

pub mod events;
pub mod input;

pub use events::{Event, EventLoop, KeyEvent, KeyState, MouseEvent, WindowEvent};
pub use input::{InputState, Key, KeyCode, ModifierKeys, MouseButton};

/// Posição da janela
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

impl WindowPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const CENTERED: Self = Self {
        x: i32::MIN,
        y: i32::MIN,
    };
}

/// Tamanho da janela
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

/// Modo de exibição da janela
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Janela normal com bordas e barra de título
    Windowed,
    /// Fullscreen exclusivo (muda resolução do monitor)
    FullscreenExclusive,
    /// Fullscreen borderless (mantém resolução do desktop)
    FullscreenBorderless,
    /// Maximizada mas com bordas
    Maximized,
}

/// Configuração da janela
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: WindowSize,
    pub position: WindowPosition,
    pub display_mode: DisplayMode,
    pub resizable: bool,
    pub decorated: bool,
    pub transparent: bool,
    pub vsync: bool,
    pub min_size: Option<WindowSize>,
    pub max_size: Option<WindowSize>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Avila Window".to_string(),
            size: WindowSize::new(1280, 720),
            position: WindowPosition::CENTERED,
            display_mode: DisplayMode::Windowed,
            resizable: true,
            decorated: true,
            transparent: false,
            vsync: true,
            min_size: None,
            max_size: None,
        }
    }
}

impl WindowConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = WindowSize::new(width, height);
        self
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.position = WindowPosition::new(x, y);
        self
    }

    pub fn with_display_mode(mut self, mode: DisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

/// Informações do monitor
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub size: WindowSize,
    pub position: WindowPosition,
    pub refresh_rate: u32,
    pub scale_factor: f32,
    pub is_primary: bool,
}

/// Handle da janela (abstração cross-platform)
pub struct Window {
    config: WindowConfig,
    is_open: bool,
    is_focused: bool,
    cursor_visible: bool,
    cursor_position: (f64, f64),
}

impl Window {
    /// Cria uma nova janela
    pub fn new(config: WindowConfig) -> Result<Self, WindowError> {
        // Em uma implementação real, aqui criaria a janela nativa
        // (Win32 API, X11, Wayland, Cocoa, etc.)
        Ok(Self {
            config,
            is_open: true,
            is_focused: true,
            cursor_visible: true,
            cursor_position: (0.0, 0.0),
        })
    }

    /// Cria uma janela com configuração padrão
    pub fn default_window() -> Result<Self, WindowError> {
        Self::new(WindowConfig::default())
    }

    /// Verifica se a janela está aberta
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Fecha a janela
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Verifica se a janela tem foco
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Define o foco da janela
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Retorna o título da janela
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Define o título da janela
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.config.title = title.into();
    }

    /// Retorna o tamanho da janela
    pub fn size(&self) -> WindowSize {
        self.config.size
    }

    /// Define o tamanho da janela
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        if let Some(min) = self.config.min_size {
            if width < min.width || height < min.height {
                return Err(WindowError::InvalidSize);
            }
        }
        if let Some(max) = self.config.max_size {
            if width > max.width || height > max.height {
                return Err(WindowError::InvalidSize);
            }
        }
        self.config.size = WindowSize::new(width, height);
        Ok(())
    }

    /// Retorna a posição da janela
    pub fn position(&self) -> WindowPosition {
        self.config.position
    }

    /// Define a posição da janela
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.config.position = WindowPosition::new(x, y);
    }

    /// Centraliza a janela no monitor
    pub fn center(&mut self) {
        self.config.position = WindowPosition::CENTERED;
    }

    /// Retorna o modo de exibição
    pub fn display_mode(&self) -> DisplayMode {
        self.config.display_mode
    }

    /// Define o modo de exibição
    pub fn set_display_mode(&mut self, mode: DisplayMode) -> Result<(), WindowError> {
        self.config.display_mode = mode;
        Ok(())
    }

    /// Muda para fullscreen exclusivo
    pub fn set_fullscreen(&mut self) -> Result<(), WindowError> {
        self.set_display_mode(DisplayMode::FullscreenExclusive)
    }

    /// Muda para fullscreen borderless
    pub fn set_fullscreen_borderless(&mut self) -> Result<(), WindowError> {
        self.set_display_mode(DisplayMode::FullscreenBorderless)
    }

    /// Muda para modo janela
    pub fn set_windowed(&mut self) -> Result<(), WindowError> {
        self.set_display_mode(DisplayMode::Windowed)
    }

    /// Verifica se está em fullscreen
    pub fn is_fullscreen(&self) -> bool {
        matches!(
            self.config.display_mode,
            DisplayMode::FullscreenExclusive | DisplayMode::FullscreenBorderless
        )
    }

    /// Maximiza a janela
    pub fn maximize(&mut self) -> Result<(), WindowError> {
        self.set_display_mode(DisplayMode::Maximized)
    }

    /// Minimiza a janela
    pub fn minimize(&mut self) {
        // Implementação específica da plataforma
    }

    /// Restaura o tamanho normal da janela
    pub fn restore(&mut self) -> Result<(), WindowError> {
        self.set_display_mode(DisplayMode::Windowed)
    }

    /// Mostra o cursor
    pub fn show_cursor(&mut self) {
        self.cursor_visible = true;
    }

    /// Esconde o cursor
    pub fn hide_cursor(&mut self) {
        self.cursor_visible = false;
    }

    /// Verifica se o cursor está visível
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Define a posição do cursor
    pub fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.cursor_position = (x, y);
    }

    /// Retorna a posição do cursor
    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_position
    }

    /// Captura o cursor (trava na janela)
    pub fn grab_cursor(&mut self, grab: bool) {
        // Implementação específica da plataforma
    }

    /// Ativa/desativa VSync
    pub fn set_vsync(&mut self, vsync: bool) {
        self.config.vsync = vsync;
    }

    /// Verifica se VSync está ativo
    pub fn vsync(&self) -> bool {
        self.config.vsync
    }

    /// Solicita atenção do usuário (taskbar flash, etc)
    pub fn request_attention(&self) {
        // Implementação específica da plataforma
    }

    /// Lista todos os monitores disponíveis
    pub fn available_monitors() -> Vec<MonitorInfo> {
        // Implementação específica da plataforma
        // Por enquanto retorna um monitor fictício
        vec![MonitorInfo {
            name: "Primary Monitor".to_string(),
            size: WindowSize::new(1920, 1080),
            position: WindowPosition::new(0, 0),
            refresh_rate: 60,
            scale_factor: 1.0,
            is_primary: true,
        }]
    }

    /// Retorna o monitor primário
    pub fn primary_monitor() -> Option<MonitorInfo> {
        Self::available_monitors()
            .into_iter()
            .find(|m| m.is_primary)
    }

    /// Retorna o monitor atual
    pub fn current_monitor(&self) -> Option<MonitorInfo> {
        Self::primary_monitor()
    }

    /// Move a janela para outro monitor
    pub fn move_to_monitor(&mut self, monitor: &MonitorInfo) {
        self.set_position(monitor.position.x, monitor.position.y);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // Limpeza da janela nativa
        self.close();
    }
}

/// Erros relacionados a janelas
#[derive(Debug, Clone)]
pub enum WindowError {
    CreationFailed(String),
    InvalidSize,
    InvalidPosition,
    DisplayModeNotSupported,
    MonitorNotFound,
    PlatformError(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            Self::InvalidSize => write!(f, "Invalid window size"),
            Self::InvalidPosition => write!(f, "Invalid window position"),
            Self::DisplayModeNotSupported => write!(f, "Display mode not supported"),
            Self::MonitorNotFound => write!(f, "Monitor not found"),
            Self::PlatformError(msg) => write!(f, "Platform error: {}", msg),
        }
    }
}

impl std::error::Error for WindowError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let config = WindowConfig::new("Test Window")
            .with_size(800, 600)
            .resizable(false);

        let window = Window::new(config).unwrap();
        assert!(window.is_open());
        assert_eq!(window.title(), "Test Window");
        assert_eq!(window.size().width, 800);
        assert_eq!(window.size().height, 600);
    }

    #[test]
    fn test_window_display_modes() {
        let mut window = Window::default_window().unwrap();

        assert!(!window.is_fullscreen());

        window.set_fullscreen().unwrap();
        assert!(window.is_fullscreen());
        assert_eq!(window.display_mode(), DisplayMode::FullscreenExclusive);

        window.set_windowed().unwrap();
        assert!(!window.is_fullscreen());
    }

    #[test]
    fn test_window_size() {
        let mut window = Window::default_window().unwrap();

        window.set_size(1920, 1080).unwrap();
        let size = window.size();
        assert_eq!(size.width, 1920);
        assert_eq!(size.height, 1080);
    }

    #[test]
    fn test_cursor_management() {
        let mut window = Window::default_window().unwrap();

        assert!(window.is_cursor_visible());

        window.hide_cursor();
        assert!(!window.is_cursor_visible());

        window.show_cursor();
        assert!(window.is_cursor_visible());
    }

    #[test]
    fn test_aspect_ratio() {
        let size = WindowSize::new(1920, 1080);
        let ratio = size.aspect_ratio();
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_monitor_info() {
        let monitors = Window::available_monitors();
        assert!(!monitors.is_empty());

        let primary = Window::primary_monitor();
        assert!(primary.is_some());
    }
}
