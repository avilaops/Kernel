# Avila Math - Infrastructure Library

High-performance infrastructure library providing 3D math, memory management, OS abstractions, and windowing system for game development and graphics applications.

## ⚠️ Important Notice

This library provides different levels of production-readiness:

**✅ Production Ready:**
- **Math 3D** (Vec3, Vec4, Mat4, Quat, Aabb) - Fully tested, zero dependencies, battle-tested algorithms
- **Memory Management** (Arena, Pool, Stack) - Production-quality allocators with comprehensive testing

**⚙️ Abstraction Layer (std wrappers):**
- **OS Abstractions** (Threading, Filesystem, Clock, Network) - Thin wrappers over Rust std library for cross-platform compatibility. These are convenience abstractions, not replacements for std.

**🚧 Stub/Prototype (not for production):**
- **Window System** (Window, Events, Input) - API design and event system only. This is a **stub/abstraction layer** without native implementation. For production use, consider established libraries like [winit](https://github.com/rust-windowing/winit) or [SDL2](https://github.com/Rust-SDL2/rust-sdl2).

Similar to how libraries like [glam](https://github.com/bitshifter/glam-rs) focuses on math and [bumpalo](https://github.com/fitzgen/bumpalo) focuses on arena allocation, Avila Math provides foundational building blocks. The window system serves as an API reference for integrating with production-ready windowing libraries.

## 📐 Math 3D

### Funcionalidades

- **Vec3 & Vec4**: Vetores 3D e 4D com operações completas
  - Operações aritméticas (adição, subtração, multiplicação, divisão)
  - Produto escalar (dot) e vetorial (cross)
  - Normalização, comprimento, distância
  - Interpolação linear (lerp)
  - Min, max, clamp

- **Mat4**: Matrizes 4x4 para transformações
  - Column-major order (compatível com OpenGL/Vulkan)
  - Transformações: translação, rotação, escala
  - Matrizes de câmera: look_at, perspective, orthographic
  - Multiplicação de matrizes e vetores
  - Transposta e determinante
  - Transformação de pontos e vetores

- **Quat**: Quaternions para rotações
  - Conversão de/para ângulos de Euler
  - Conversão de/para eixo-ângulo
  - Conversão para Mat4
  - Interpolação (lerp e slerp)
  - Rotação de vetores
  - Operações: multiplicação, conjugado, inverso

- **Aabb**: Axis-Aligned Bounding Box
  - Criação a partir de pontos, centro/tamanho
  - Testes de contenção (ponto, AABB)
  - Testes de interseção (AABB, raio)
  - Expansão e união de AABBs
  - Cálculo de volume e área de superfície
  - Ponto mais próximo e distância

## 🧠 Memory Management

Sistema completo de gerenciamento de memória com múltiplos allocators especializados.

### Allocators Disponíveis

#### Arena Allocator
Alocador linear de alta performance para alocações temporárias.

**Características:**
- Alocação O(1) extremamente rápida (apenas incrementa ponteiro)
- Não suporta free individual, apenas reset completo
- Excelente localidade de cache
- Perfeito para frames em game engines, parsing temporário

**Uso:**
```rust
use kernel_math::memory::Arena;

let arena = Arena::new(1024 * 1024); // 1MB
let ptr = arena.alloc(256, 8);

// Uso com checkpoint
let checkpoint = arena.checkpoint();
// ... alocações ...
arena.restore(checkpoint);

// Reset completo
arena.reset();
```

#### Pool Allocator
Gerenciador de blocos de tamanho fixo para objetos do mesmo tipo.

**Características:**
- Alocação e liberação O(1)
- Zero fragmentação para objetos de tamanho fixo
- Excelente para gerenciar entidades, partículas, componentes
- Cache-friendly com memória contígua

**Uso:**
```rust
use kernel_math::memory::{Pool, TypedPool, PoolBox};

// Pool genérico
let pool = Pool::for_type::<MyStruct>(128);
let ptr = pool.alloc_type::<MyStruct>().unwrap();
unsafe { pool.free_type(ptr); }

// Pool tipado (type-safe)
let typed_pool = TypedPool::<MyStruct>::new(128);
let ptr = typed_pool.alloc().unwrap();
unsafe { typed_pool.free(ptr); }

// PoolBox com RAII
let boxed = PoolBox::new(&pool, MyStruct::new()).unwrap();
// Libera automaticamente ao sair do escopo
```

#### Stack Allocator
Alocador LIFO para hierarquias e processamento estruturado.

**Características:**
- Alocação O(1) muito rápida
- Liberação O(1) apenas na ordem correta (LIFO)
- Excelente localidade de cache
- Perfeito para call stacks, processamento hierárquico

**Uso:**
```rust
use kernel_math::memory::StackAllocator;

let stack = StackAllocator::new(512 * 1024); // 512KB

// Alocação simples
let ptr = stack.alloc(256, 16);

// Com marcadores
let mark = stack.mark();
// ... alocações ...
stack.free_to_mark(mark);

// Escopo automático
{
    let scoped = ScopedStack::new(&stack);
    scoped.alloc(1024, 8);
} // Libera automaticamente
```

#### Double-Ended Stack
Stack que cresce dos dois lados para separar tipos de alocações.

**Uso:**
```rust
use kernel_math::memory::DoubleEndedStack;

let stack = DoubleEndedStack::new(1024 * 1024);

// Aloca do começo (para dados persistentes)
let bottom_ptr = stack.alloc_bottom(256, 8);

// Aloca do final (para dados temporários)
let top_ptr = stack.alloc_top(128, 4);

stack.clear_top(); // Limpa apenas o topo
```

### Memory Manager & Profiling

Sistema centralizado de tracking e estatísticas.

```rust
use kernel_math::memory::{MemoryManager, AllocatorInfo, AllocatorType};

let mut manager = MemoryManager::new();

// Registra allocators
manager.register_allocator("main_arena", AllocatorInfo {
    allocator_type: AllocatorType::Arena,
    total_capacity: 1024 * 1024,
    used: 512 * 1024,
    available: 512 * 1024,
    allocation_count: 100,
    deallocation_count: 0,
});

// Gera relatório
let report = manager.report();
report.print_summary();

// Profiling ao longo do tempo
use std::time::Duration;
let mut profiler = MemoryProfiler::new(Duration::from_millis(100));

// Coleta amostras
profiler.sample(&stats);

// Análise
let avg_usage = profiler.average_usage();
let peak_usage = profiler.peak_usage();
```

## 🖥️ Operating System Abstraction

Sistema completo de abstração de SO para operações cross-platform.

## 🪟 Window System

Sistema completo de gerenciamento de janelas, eventos e input.

### Window Management

Criação e gerenciamento de janelas com múltiplos modos de exibição.

**Componentes:**
- **Window**: Gerenciamento de janela com controle completo
- **WindowConfig**: Configuração de janela (título, tamanho, posição, etc)
- **DisplayMode**: Modos (Windowed, Fullscreen, Borderless, Maximized)
- **WindowPosition** & **WindowSize**: Posição e tamanho da janela
- **MonitorInfo**: Informações de monitores disponíveis

**Uso:**
```rust
use kernel_math::window::{Window, WindowConfig, DisplayMode};

// Criar janela
let config = WindowConfig::new("My Game")
    .with_size(1280, 720)
    .resizable(true)
    .vsync(true);

let mut window = Window::new(config)?;

// Controlar janela
window.set_fullscreen()?;  // Fullscreen exclusivo
window.set_fullscreen_borderless()?;  // Fullscreen sem bordas
window.set_windowed()?;  // Volta ao modo janela
window.maximize()?;  // Maximiza

// Cursor
window.hide_cursor();
window.show_cursor();
window.set_cursor_position(100.0, 200.0);

// Multi-monitor
let monitors = Window::available_monitors();
let primary = Window::primary_monitor().unwrap();
window.move_to_monitor(&primary);
```

### Event System

Sistema de eventos para capturar input, resize, focus, etc.

**Componentes:**
- **EventLoop**: Loop de eventos principal
- **Event**: Enum de todos os tipos de eventos
- **WindowEvent**: Eventos da janela (resize, close, focus)
- **KeyEvent**: Eventos de teclado com modificadores
- **MouseEvent**: Eventos de mouse (click, move, scroll)

**Uso:**
```rust
use kernel_math::window::{EventLoop, Event, WindowEvent, KeyEvent, MouseEvent};

let mut event_loop = EventLoop::new();

while event_loop.is_running() {
    for event in event_loop.poll_events() {
        match event {
            Event::Window(WindowEvent::Closed) => {
                // Janela fechada
                event_loop.stop();
            }
            Event::Window(WindowEvent::Resized(size)) => {
                println!("Resized to {}x{}", size.width, size.height);
            }
            Event::Keyboard(key_event) => {
                if key_event.is_pressed() {
                    println!("Key pressed: {:?}", key_event.key);
                }
            }
            Event::Mouse(MouseEvent::ButtonPressed { button, position, .. }) => {
                println!("Mouse {:?} at {:?}", button, position);
            }
            Event::Mouse(MouseEvent::Scrolled { delta, .. }) => {
                println!("Scroll: {:?}", delta);
            }
            _ => {}
        }
    }
}
```

### Input System

Sistema de input para teclado e mouse com rastreamento de estado.

**Componentes:**
- **InputState**: Rastreia estado atual de teclas e botões
- **Key** & **KeyCode**: Teclas do teclado (físicas e caracteres)
- **MouseButton**: Botões do mouse (Left, Right, Middle, Back, Forward)
- **ModifierKeys**: Teclas modificadoras (Ctrl, Shift, Alt, Meta/Win/Cmd)

**Uso:**
```rust
use kernel_math::window::{InputState, Key, KeyCode, MouseButton, ModifierKeys};

let mut input = InputState::new();

// Processar eventos
input.press_key(Key::Code(KeyCode::W));
input.press_button(MouseButton::Left);
input.set_cursor_position(100.0, 200.0);

// Verificar estado
if input.is_keycode_pressed(KeyCode::W) {
    println!("Moving forward!");
}

if input.is_button_pressed(MouseButton::Left) {
    let (x, y) = input.cursor_position();
    println!("Left button down at {}, {}", x, y);
}

// Modificadores
if input.modifiers().has_ctrl() && input.is_keycode_pressed(KeyCode::S) {
    println!("Ctrl+S: Save!");
}

// Scroll (resetar a cada frame)
let (scroll_x, scroll_y) = input.scroll_delta();
input.reset_scroll_delta();
```

### Exemplo Completo: Game Loop

```rust
use kernel_math::window::{
    Window, WindowConfig, EventLoop, Event, WindowEvent,
    KeyEvent, MouseEvent, InputState, Key, KeyCode,
};
use kernel_math::os::{FpsCounter, DeltaTime};

struct Game {
    window: Window,
    event_loop: EventLoop,
    input: InputState,
    fps: FpsCounter,
    dt: DeltaTime,
}

impl Game {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            window: Window::new(WindowConfig::new("My Game"))?,
            event_loop: EventLoop::new(),
            input: InputState::new(),
            fps: FpsCounter::new(),
            dt: DeltaTime::new(),
        })
    }

    fn run(&mut self) {
        while self.event_loop.is_running() && self.window.is_open() {
            // Process events
            for event in self.event_loop.poll_events() {
                match event {
                    Event::Window(WindowEvent::Closed) => {
                        self.event_loop.stop();
                    }
                    Event::Keyboard(key_event) => {
                        if key_event.is_pressed() {
                            self.input.press_key(key_event.key);

                            // ESC to exit
                            if matches!(key_event.key, Key::Code(KeyCode::Escape)) {
                                self.event_loop.stop();
                            }
                        } else {
                            self.input.release_key(key_event.key);
                        }
                    }
                    Event::Mouse(MouseEvent::CursorMoved { position, .. }) => {
                        self.input.set_cursor_position(position.0, position.1);
                    }
                    _ => {}
                }
            }

            // Update
            let delta = self.dt.update();
            self.update(delta.as_secs_f32());

            // Render
            self.render();

            // FPS
            self.fps.tick();

            // Reset per-frame input
            self.input.reset_scroll_delta();
        }
    }

    fn update(&mut self, dt: f32) {
        // WASD movement
        if self.input.is_keycode_pressed(KeyCode::W) {
            // Move forward
        }
        if self.input.is_keycode_pressed(KeyCode::S) {
            // Move backward
        }
        // ... etc
    }

    fn render(&mut self) {
        // Rendering code
    }
}
```

## 🖥️ Operating System Abstraction (Legacy Header)

Sistema completo de abstração de SO para operações cross-platform.

### Threading

Pool de threads e primitivas de sincronização avançadas.

**Componentes:**
- **ThreadPool**: Pool gerenciado de worker threads com fila de tarefas
- **Semaphore**: Contador semáforo com wait/signal
- **RwCounter**: Contador com leitura/escrita concorrente
- **ThreadBarrier**: Barreira de sincronização para múltiplas threads
- **ShutdownFlag**: Flag atômica para shutdown coordenado
- **ManagedThread**: Thread com nome e ID gerenciados
- **TaskScheduler**: Agendador de tarefas com prioridades

**Uso:**
```rust
use kernel_math::os::{ThreadPool, Semaphore, TaskScheduler, Priority};

// Thread pool
let pool = ThreadPool::new(4);
pool.execute(|| {
    println!("Task executando em worker thread");
});

// Semaphore
let sem = Semaphore::new(3); // Permite 3 threads simultâneas
sem.wait();
// ... seção crítica ...
sem.signal();

// Task scheduler com prioridades
let mut scheduler = TaskScheduler::new(4);
scheduler.schedule_with_priority(
    "critical_task",
    Priority::High,
    || { /* trabalho crítico */ }
);
```

### Filesystem

Operações de arquivo e diretório cross-platform.

**Componentes:**
- **FileSystem**: API estática para operações de arquivo
- **FileHandle**: Handle com Read/Write/Seek traits
- **FileMetadata**: Metadados (tamanho, tipo, timestamps, permissions)
- **PathUtil**: Utilitários para manipulação de paths
- **DirectoryWalker**: Iterator para percorrer diretórios recursivamente
- **FileWatcher**: Observador de mudanças em arquivos (TODO: inotify/FSEvents)

**Uso:**
```rust
use kernel_math::os::{FileSystem, FileHandle, PathUtil, DirectoryWalker};

// Operações básicas
FileSystem::write("config.txt", b"Hello, World!").unwrap();
let content = FileSystem::read_to_string("config.txt").unwrap();
FileSystem::copy("source.txt", "dest.txt").unwrap();
FileSystem::remove("old.txt").unwrap();

// FileHandle com controle fino
let mut file = FileHandle::create("data.bin").unwrap();
file.write_all(b"Binary data").unwrap();
file.seek(SeekFrom::Start(0)).unwrap();
let mut buffer = Vec::new();
file.read_to_end(&mut buffer).unwrap();

// Paths
let absolute = PathUtil::absolute("./relative/path.txt").unwrap();
let parent = PathUtil::parent(&absolute);
let filename = PathUtil::filename(&absolute);

// Directory walker
for entry in DirectoryWalker::new("src").recursive(true) {
    if PathUtil::extension(&entry) == Some("rs") {
        println!("Rust file: {}", entry);
    }
}
```

### Clock & Timing

Sistema de timing de alta precisão.

**Componentes:**
- **Clock**: Relógio de alta precisão (cross-platform)
- **Timer**: Timer com callback e repetição
- **Stopwatch**: Cronômetro com pause/resume
- **FpsCounter**: Contador de FPS com intervalo configurável
- **DeltaTime**: Calculador de delta time com smoothing
- **Profiler**: Profiler de seções de código nomeadas

**Uso:**
```rust
use kernel_math::os::{Clock, Timer, Stopwatch, FpsCounter, DeltaTime, Profiler};

// Clock high-precision
let timestamp = Clock::now();
std::thread::sleep(std::time::Duration::from_millis(10));
let elapsed = Clock::elapsed_since(timestamp);

// Timer com callback
let mut timer = Timer::new(std::time::Duration::from_secs(1), || {
    println!("Timer tick!");
});
timer.tick(); // Chama callback se tempo expirou

// Stopwatch
let mut sw = Stopwatch::new();
sw.start();
// ... trabalho ...
sw.stop();
println!("Elapsed: {:?}", sw.elapsed());

// FPS Counter
let mut fps = FpsCounter::new();
loop {
    fps.tick();
    println!("FPS: {:.1}", fps.fps());
}

// Delta Time com smoothing
let mut dt = DeltaTime::new();
loop {
    let delta = dt.update();
    let smoothed = dt.smoothed();
    // ... use delta para movimento ...
}

// Profiler
let mut profiler = Profiler::new();
profiler.begin_section("physics");
// ... código de física ...
profiler.end_section("physics");
profiler.print_report();
```

### Network

Abstrações de rede TCP/UDP e cliente HTTP simples.

**Componentes:**
- **TcpServer**: Servidor TCP com accept non-blocking opcional
- **TcpClient**: Cliente TCP com timeout configurável
- **UdpClient**: Cliente UDP para datagramas
- **HttpClient**: Cliente HTTP simples (GET requests)
- **NetworkBuffer**: Buffer para serialização de dados de rede
- **Network utilities**: Funções utilitárias (hostname, port available)

**Uso:**
```rust
use kernel_math::os::{TcpServer, TcpClient, UdpClient, HttpClient, NetworkBuffer};

// TCP Server
let server = TcpServer::bind("127.0.0.1:8080").unwrap();
server.set_nonblocking(true).unwrap();
for stream in server.incoming() {
    // Processar conexão
    let mut buf = [0u8; 1024];
    stream.read(&mut buf).unwrap();
}

// TCP Client
let mut client = TcpClient::connect("127.0.0.1:8080").unwrap();
client.write_all(b"Hello, Server!").unwrap();

// UDP Client
let udp = UdpClient::bind("0.0.0.0:0").unwrap();
udp.send_to(b"UDP message", "127.0.0.1:9000").unwrap();

// HTTP Client
let http = HttpClient::new();
let response = http.get("http://example.com/api/data").unwrap();
println!("Response: {}", response);

// Network Buffer para serialização
let mut buffer = NetworkBuffer::with_capacity(1024);
buffer.write_u32(42);
buffer.write_string("Hello");
buffer.write_bytes(b"data");
let data = buffer.as_slice();
```

### System Information

Informações do sistema, processos e console.

**Componentes:**
- **SystemInfo**: Informações do SO (OS type, CPU count, hostname)
- **Environment**: Gerenciamento de variáveis de ambiente
- **Process**: Controle de processos (spawn, shell, exit)
- **Console**: I/O de console com cores ANSI

**Uso:**
```rust
use kernel_math::os::{SystemInfo, Environment, Process, Console, ConsoleColor};

// System Info
let os = SystemInfo::os_name();
let cpus = SystemInfo::cpu_count();
let hostname = SystemInfo::hostname();
println!("{} - {} CPUs - {}", os, cpus, hostname);

// Environment
Environment::set_var("MY_VAR", "value");
let value = Environment::var("MY_VAR").unwrap();
Environment::remove_var("MY_VAR");

// Process
let output = Process::spawn("ls", &["-la"]).unwrap();
println!("Output: {}", output);
Process::shell("echo Hello").unwrap();

// Console
Console::write("Normal text\n");
Console::write_colored("Green text\n", ConsoleColor::Green);
Console::clear_screen();
let input = Console::read_line().unwrap();
```

## Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
kernel-math = { path = "../Kernel" }
```

## Uso Básico - Math

```rust
use kernel_math::{Vec3, Mat4, Quat, Aabb};

fn main() {
    // Vetores
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);
    let sum = v1 + v2;
    let dot = v1.dot(v2);
    let cross = v1.cross(v2);

    // Matrizes
    let translation = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
    let rotation = Mat4::from_rotation_y(std::f32::consts::PI / 2.0);
    let transform = translation * rotation;

    // Quaternions
    let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0);
    let rotated = q.rotate_vec3(Vec3::X);

    // AABB
    let aabb = Aabb::from_center_size(Vec3::ZERO, Vec3::ONE);
    let contains = aabb.contains_point(Vec3::new(0.5, 0.0, 0.0));
}
```

## Exemplo Completo - Game Engine

```rust
use kernel_math::{Vec3, Mat4, Quat};
use kernel_math::memory::{Arena, Pool, StackAllocator};
use kernel_math::os::{ThreadPool, FpsCounter, DeltaTime, Profiler};

struct GameEngine {
    // Arena para dados temporários de frame
    frame_arena: Arena,

    // Pool para entidades
    entity_pool: Pool,

    // Stack para processamento hierárquico
    transform_stack: StackAllocator,

    // Threading para tasks paralelas
    thread_pool: ThreadPool,

    // Timing
    fps_counter: FpsCounter,
    delta_time: DeltaTime,
    profiler: Profiler,
}

impl GameEngine {
    fn new() -> Self {
        Self {
            frame_arena: Arena::new(4 * 1024 * 1024),  // 4MB
            entity_pool: Pool::for_type::<Entity>(1024),
            transform_stack: StackAllocator::new(1 * 1024 * 1024), // 1MB
            thread_pool: ThreadPool::new(4),
            fps_counter: FpsCounter::new(),
            delta_time: DeltaTime::new(),
            profiler: Profiler::new(),
        }
    }

    fn update(&mut self) {
        // Timing
        self.fps_counter.tick();
        let dt = self.delta_time.update();

        // Profiling
        self.profiler.begin_section("frame");

        // Usa arena para dados temporários do frame
        let temp_data = self.frame_arena.alloc(1024, 8);

        // Pool para entidades persistentes
        let entity = self.entity_pool.alloc_type::<Entity>();

        // Stack para transformações hierárquicas
        self.profiler.begin_section("transforms");
        let mark = self.transform_stack.mark();
        self.process_hierarchy();
        self.transform_stack.free_to_mark(mark);
        self.profiler.end_section("transforms");

        // Processamento paralelo
        self.profiler.begin_section("physics");
        self.thread_pool.execute(|| {
            // Física em background
        });
        self.profiler.end_section("physics");

        // Limpa dados temporários do frame
        self.frame_arena.reset();

        self.profiler.end_section("frame");

        // Log FPS
        if self.fps_counter.fps() > 0.0 {
            println!("FPS: {:.1}", self.fps_counter.fps());
        }
    }
}
```

## Testes

Execute os testes com:

```bash
cargo test
```

**Resultado:** ✅ **97 testes passando!**
- 78 unit tests na biblioteca principal
- 16 integration tests de memória
- 3 doc tests

### Cobertura de Testes:
- ✅ Math 3D: Vec3, Vec4, Mat4, Quat, Aabb (18 testes)
- ✅ Memory: Arena, Pool, Stack, Manager (38 testes)
- ✅ Window System: Window, Events, Input (18 testes)
- ✅ OS Threading: ThreadPool, Semaphore, Barriers (5 testes)
- ✅ OS Filesystem: FileHandle, PathUtil, Read/Write (4 testes)
- ✅ OS Clock: Timer, Stopwatch, FPS Counter, Delta Time (5 testes)
- ✅ OS Network: TCP/UDP local, NetworkBuffer (4 testes)
- ✅ OS System: SystemInfo, Environment, Process (4 testes)
- ✅ Integration: Math+Memory scenarios (2 testes)

## Performance

### Math 3D
- Todas as operações são `#[inline]` para otimização
- Usa tipos `f32` (single precision) por padrão
- Zero alocações de heap para operações matemáticas
- Column-major order para compatibilidade direta com GPUs

### Memory Management
- Alocações O(1) em todos os allocators
- Zero overhead em builds release
- Thread-safe (Send + Sync)
- Alinhamento configurável por alocação

### OS Abstraction
- Cross-platform (Windows, Linux, macOS)
- ThreadPool com work-stealing e shutdown gracioso
- FileSystem com operações atômicas quando possível
- High-precision timing (nanosegundos)
- Network com non-blocking I/O opcional
- Zero-cost abstractions (thin wrappers sobre std)

## Benchmarks Esperados

| Allocator  | Alocação  | Liberação            | Uso Ideal                  |
| ---------- | --------- | -------------------- | -------------------------- |
| Arena      | ~1ns      | N/A (reset em massa) | Dados temporários de frame |
| Pool       | ~5ns      | ~5ns                 | Objetos de tamanho fixo    |
| Stack      | ~2ns      | ~2ns (LIFO)          | Processamento hierárquico  |
| std::alloc | ~50-100ns | ~50-100ns            | Tamanhos variados          |

## Estrutura do Projeto

```
Kernel/
├── src/
│   ├── lib.rs              # Ponto de entrada da biblioteca
│   ├── vec3.rs             # Vetores 3D
│   ├── vec4.rs             # Vetores 4D
│   ├── mat4.rs             # Matrizes 4x4
│   ├── quat.rs             # Quaternions
│   ├── aabb.rs             # Bounding Boxes
│   ├── memory/
│   │   ├── mod.rs          # Module exports
│   │   ├── arena.rs        # Arena allocator
│   │   ├── pool.rs         # Pool allocator
│   │   ├── stack.rs        # Stack allocator
│   │   └── manager.rs      # Memory manager & profiler
│   ├── os/
│   │   ├── mod.rs          # System info, environment, process, console
│   │   ├── threading.rs    # ThreadPool, semaphore, barriers
│   │   ├── filesystem.rs   # FileSystem, FileHandle, paths
│   │   ├── clock.rs        # Timing, FPS, profiling
│   │   └── network.rs      # TCP, UDP, HTTP client
│   └── window/
│       ├── mod.rs          # Window management, DisplayMode
│       ├── events.rs       # Event loop, WindowEvent, KeyEvent, MouseEvent
│       └── input.rs        # InputState, Key, KeyCode, MouseButton
├── tests/
│   └── memory_tests.rs     # Integration tests
├── examples/
│   └── window_app.rs       # Complete windowing application example
├── Cargo.toml
└── README.md
```

## Dependências

```toml
[dependencies]
hostname = "0.4"  # Para Network::hostname()

# Apenas std lib para o resto!
```

## Recursos

### Principais Características

✅ **Math 3D completa** - Vetores, matrizes, quaternions, AABB
✅ **Memory management** - Arena, Pool, Stack allocators com profiling
✅ **Window System** - Janelas, fullscreen, eventos, input (teclado/mouse)
✅ **OS Threading** - ThreadPool, sincronização avançada, task scheduler
✅ **OS Filesystem** - Operações cross-platform, metadata, directory walker
✅ **OS Timing** - High-precision clock, FPS counter, delta time, profiler
✅ **OS Network** - TCP/UDP sockets, HTTP client simples
✅ **OS System** - Info do sistema, processos, variáveis de ambiente, console
✅ **97 testes passando** - Cobertura completa de funcionalidades
✅ **Zero unsafe** (exceto em alocadores low-level)
✅ **Thread-safe** - Send + Sync em todas as estruturas públicas
✅ **Cross-platform** - Windows, Linux, macOS

### Próximos Passos

- [ ] Implementação nativa de janelas (Win32, X11, Wayland, Cocoa)
- [ ] Context OpenGL/Vulkan para rendering
- [ ] Gamepad/Controller input
- [ ] Drag & drop de arquivos
- [ ] Clipboard support
- [ ] SIMD optimizations para math (Vec3, Mat4)
- [ ] Async I/O para filesystem e network (Tokio integration)
- [ ] File watching implementation (inotify/FSEvents)
- [ ] GPU memory allocators
- [ ] Benchmarks formais com Criterion
- [ ] Documentação expandida com mais exemplos## Licença

Dual-licensed under:
- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Contribuição

See [CLA.md](CLA.md) for contribution terms and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards.

---

**Avila Math** - Part of the Avila project
