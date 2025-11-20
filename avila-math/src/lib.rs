//! # Kernel - Infrastructure Library
//!
//! Biblioteca de infraestrutura para o Avila Framework, fornecendo:
//!
//! ## Math 3D
//! - **Vec3 & Vec4**: Vetores 3D e 4D com operações completas
//! - **Mat4**: Matrizes 4x4 para transformações (column-major, compatível com OpenGL/Vulkan)
//! - **Quat**: Quaternions para rotações suaves e eficientes
//! - **Aabb**: Axis-Aligned Bounding Boxes para detecção de colisão
//!
//! ## Memory Management
//! - **Arena**: Alocador linear de alta performance para alocações temporárias
//! - **Pool**: Alocador de objetos de tamanho fixo com zero fragmentação
//! - **Stack**: Alocador LIFO para hierarquias
//! - **MemoryManager**: Gerenciador central com estatísticas e profiling
//!
//! ## Operating System Abstraction
//! - **Threading**: Thread pools, task scheduler, sincronização avançada
//! - **FileSystem**: Operações de arquivo e diretório cross-platform
//! - **Clock**: Timers, FPS counter, delta time, profiling
//! - **Network**: TCP/UDP sockets, HTTP client simples
//! - **System**: Informações do sistema, processos, variáveis de ambiente
//!
//! ## Exemplo de Uso - Math
//!
//! ```rust
//! use kernel_math::{Vec3, Mat4, Quat, Aabb};
//!
//! // Vetores
//! let v1 = Vec3::new(1.0, 2.0, 3.0);
//! let v2 = Vec3::new(4.0, 5.0, 6.0);
//! let dot = v1.dot(v2);
//! let cross = v1.cross(v2);
//!
//! // Matrizes
//! let translation = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
//! let rotation = Mat4::from_rotation_y(std::f32::consts::PI / 2.0);
//! let transform = translation * rotation;
//!
//! // Quaternions
//! let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 4.0);
//! let rotated = q.rotate_vec3(Vec3::X);
//!
//! // AABB
//! let aabb = Aabb::from_center_size(Vec3::ZERO, Vec3::ONE);
//! let contains = aabb.contains_point(Vec3::new(0.5, 0.0, 0.0));
//! ```
//!
//! ## Exemplo de Uso - Memory
//!
//! ```rust
//! use kernel_math::memory::{Arena, Pool, StackAllocator};
//!
//! #[derive(Debug, Clone)]
//! struct MyStruct {
//!     value: i32,
//! }
//!
//! // Arena para alocações temporárias
//! let arena = Arena::new(1024 * 1024); // 1MB
//! let ptr = arena.alloc(256, 8);
//! arena.reset(); // Libera tudo de uma vez
//!
//! // Pool para objetos de tamanho fixo
//! let pool = Pool::for_type::<MyStruct>(128);
//! let obj = pool.alloc_type::<MyStruct>().unwrap();
//!
//! // Stack para alocações hierárquicas
//! let stack = StackAllocator::new(512 * 1024);
//! let mark = stack.mark();
//! // ... alocações ...
//! stack.free_to_mark(mark);
//! ```
//!
//! ## Exemplo de Uso - OS Abstraction
//!
//! ```rust,no_run
//! use kernel_math::os::{ThreadPool, FileSystem, Clock, TcpServer, FpsCounter, DeltaTime};
//! use std::time::Duration;
//!
//! // Thread pool para processamento paralelo
//! let pool = ThreadPool::new(4);
//! pool.execute(|| println!("Task executando em background!"));
//!
//! // Operações de arquivo cross-platform
//! let content = FileSystem::read_to_string("config.txt").unwrap();
//! FileSystem::write("output.txt", b"Hello, World!").unwrap();
//!
//! // Timing e profiling
//! let mut fps = FpsCounter::new();
//! let mut dt = DeltaTime::new();
//! fps.tick();
//! let frame_time = dt.update();
//!
//! // Network server TCP
//! let server = TcpServer::bind("127.0.0.1:8080").unwrap();
//! for stream in server.incoming() {
//!     // processar conexão
//!     break; // apenas exemplo
//! }
//! ```

pub mod aabb;
pub mod mat4;
pub mod memory;
pub mod os;
pub mod quat;
pub mod vec3;
pub mod vec4;
pub mod window;

pub use aabb::Aabb;
pub use mat4::Mat4;
pub use quat::Quat;
pub use vec3::Vec3;
pub use vec4::Vec4;

/// Constantes matemáticas úteis
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const E: f32 = std::f32::consts::E;
}

/// Funções utilitárias
pub mod utils {
    /// Converte graus para radianos
    #[inline]
    pub fn deg_to_rad(degrees: f32) -> f32 {
        degrees * (std::f32::consts::PI / 180.0)
    }

    /// Converte radianos para graus
    #[inline]
    pub fn rad_to_deg(radians: f32) -> f32 {
        radians * (180.0 / std::f32::consts::PI)
    }

    /// Clamp de um valor entre min e max
    #[inline]
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.max(min).min(max)
    }

    /// Interpolação linear
    #[inline]
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Verifica se dois floats são aproximadamente iguais
    #[inline]
    pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    /// Smooth step (interpolação suave)
    #[inline]
    pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() {
        // Teste de integração: criar uma transformação completa
        let position = Vec3::new(10.0, 5.0, 0.0);
        let rotation = Quat::from_euler(0.0, utils::deg_to_rad(45.0), 0.0);
        let scale = Vec3::new(2.0, 2.0, 2.0);

        // Criar matrizes
        let translation_mat = Mat4::from_translation(position);
        let rotation_mat = rotation.to_mat4();
        let scale_mat = Mat4::from_scale(scale);

        // Combinar transformações (TRS order)
        let transform = translation_mat * rotation_mat * scale_mat;

        // Transformar um ponto
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point3(point);

        // Verificar que o ponto foi transformado
        assert!(transformed.length() > 0.0);
    }

    #[test]
    fn test_aabb_transform() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::ONE);
        let points = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5)];

        for point in points {
            assert!(aabb.contains_point(point));
        }
    }
}
