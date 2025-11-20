//! Exemplo de aplicação usando o sistema de janelas
//!
//! Demonstra como criar uma aplicação com game loop, processamento de eventos,
//! input handling e integração com os outros sistemas da biblioteca.

use avila_math::memory::Arena;
use avila_math::os::{DeltaTime, FpsCounter};
use avila_math::window::{
    Event, EventLoop, InputState, Key, KeyCode, KeyEvent, MouseButton, MouseEvent, Window,
    WindowConfig, WindowEvent,
};
use avila_math::Vec3;

/// Aplicação exemplo
pub struct Application {
    window: Window,
    event_loop: EventLoop,
    input_state: InputState,
    fps_counter: FpsCounter,
    delta_time: DeltaTime,
    frame_arena: Arena,
    running: bool,
}

impl Application {
    /// Cria uma nova aplicação
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let config = WindowConfig::new(title)
            .with_size(width, height)
            .resizable(true)
            .vsync(true);

        let window = Window::new(config)?;
        let event_loop = EventLoop::new();
        let input_state = InputState::new();

        Ok(Self {
            window,
            event_loop,
            input_state,
            fps_counter: FpsCounter::new(),
            delta_time: DeltaTime::new(),
            frame_arena: Arena::new(4 * 1024 * 1024), // 4MB para dados temporários
            running: true,
        })
    }

    /// Roda o game loop
    pub fn run(&mut self) {
        while self.running && self.window.is_open() {
            // 1. Processa eventos
            self.process_events();

            // 2. Update da aplicação
            let dt = self.delta_time.update();
            self.update(dt.as_secs_f32());

            // 3. Render
            self.render();

            // 4. FPS tracking
            self.fps_counter.tick();

            // 5. Limpa arena temporária
            self.frame_arena.reset();
            self.input_state.reset_scroll_delta();
        }

        self.shutdown();
    }

    fn process_events(&mut self) {
        let events: Vec<_> = self.event_loop.poll_events().collect();
        for event in events {
            match event {
                Event::Window(window_event) => {
                    self.handle_window_event(window_event);
                }
                Event::Keyboard(key_event) => {
                    self.handle_keyboard_event(key_event);
                }
                Event::Mouse(mouse_event) => {
                    self.handle_mouse_event(mouse_event);
                }
                Event::FrameTick(_) => {}
            }
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Closed => {
                self.running = false;
            }
            WindowEvent::Resized(size) => {
                println!("Window resized to {}x{}", size.width, size.height);
            }
            WindowEvent::Focused => {
                self.window.set_focused(true);
            }
            WindowEvent::Unfocused => {
                self.window.set_focused(false);
            }
            WindowEvent::CursorEntered => {
                println!("Cursor entered window");
            }
            WindowEvent::CursorLeft => {
                println!("Cursor left window");
            }
            _ => {}
        }
    }

    fn handle_keyboard_event(&mut self, event: KeyEvent) {
        if event.is_pressed() {
            self.input_state.press_key(event.key);

            // Comandos especiais
            match event.key {
                Key::Code(KeyCode::Escape) => {
                    self.running = false;
                }
                Key::Code(KeyCode::F11) => {
                    if self.window.is_fullscreen() {
                        let _ = self.window.set_windowed();
                    } else {
                        let _ = self.window.set_fullscreen_borderless();
                    }
                }
                Key::Code(KeyCode::F12) => {
                    println!("FPS: {:.1}", self.fps_counter.fps());
                }
                Key::Code(KeyCode::F) if event.modifiers.has_ctrl() => {
                    println!("FPS: {:.1}", self.fps_counter.fps());
                }
                _ => {}
            }
        } else {
            self.input_state.release_key(event.key);
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::ButtonPressed {
                button, position, ..
            } => {
                self.input_state.press_button(button);
                self.input_state.set_cursor_position(position.0, position.1);
                println!("Mouse button {:?} pressed at {:?}", button, position);
            }
            MouseEvent::ButtonReleased {
                button, position, ..
            } => {
                self.input_state.release_button(button);
                self.input_state.set_cursor_position(position.0, position.1);
            }
            MouseEvent::CursorMoved { position, delta } => {
                self.input_state.set_cursor_position(position.0, position.1);

                // Exemplo: rotação de câmera com mouse se botão direito pressionado
                if self.input_state.is_button_pressed(MouseButton::Right) {
                    println!("Camera rotation: delta {:?}", delta);
                }
            }
            MouseEvent::Scrolled { delta, .. } => {
                self.input_state.set_scroll_delta(delta.0, delta.1);
                println!("Scroll: {:?}", delta);
            }
        }
    }

    fn update(&mut self, dt: f32) {
        // Exemplo de movimento com WASD
        let mut movement = Vec3::ZERO;

        if self.input_state.is_keycode_pressed(KeyCode::W) {
            movement.z += 1.0;
        }
        if self.input_state.is_keycode_pressed(KeyCode::S) {
            movement.z -= 1.0;
        }
        if self.input_state.is_keycode_pressed(KeyCode::A) {
            movement.x -= 1.0;
        }
        if self.input_state.is_keycode_pressed(KeyCode::D) {
            movement.x += 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
            let speed = if self.input_state.modifiers().has_shift() {
                10.0 // Sprint
            } else {
                5.0 // Walk
            };

            let velocity = movement * speed * dt;
            println!("Moving: {:?}", velocity);
        }

        // Exemplo de uso de arena temporária
        let _temp_data = self.frame_arena.alloc(1024, 8);
    }

    fn render(&mut self) {
        // Aqui viria o código de renderização
        // - Clear do framebuffer
        // - Draw calls
        // - Swap buffers
    }

    fn shutdown(&mut self) {
        println!("Shutting down application...");
        self.window.close();
        println!("Final stats:");
        println!("  FPS: {:.1}", self.fps_counter.fps());
    }
}

fn main() {
    println!("Window system example (stub - full implementation requires platform layer)");
    println!("Run tests with: cargo test --example window_app");
}

/// Exemplo de uso
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_creation() {
        let app = Application::new("Test App", 800, 600);
        assert!(app.is_ok());

        let app = app.unwrap();
        assert_eq!(app.window.title(), "Test App");
        assert!(app.running);
    }

    #[test]
    fn test_application_input_handling() {
        let mut app = Application::new("Test", 800, 600).unwrap();

        // Simula pressionar tecla W
        let key_event = KeyEvent::new(
            Key::Code(KeyCode::W),
            kernel_math::window::KeyState::Pressed,
        );
        app.handle_keyboard_event(key_event);

        assert!(app.input_state.is_keycode_pressed(KeyCode::W));
    }
}

#[cfg(not(test))]
fn _example_main() {
    let mut app =
        Application::new("Avila Window Example", 1280, 720).expect("Failed to create application");

    println!("Application started!");
    println!("Controls:");
    println!("  WASD - Movement");
    println!("  Right Mouse - Camera rotation");
    println!("  Shift - Sprint");
    println!("  F11 - Toggle fullscreen");
    println!("  F12 - Print profiler report");
    println!("  Ctrl+F - Print FPS");
    println!("  ESC - Exit");

    app.run();
}
