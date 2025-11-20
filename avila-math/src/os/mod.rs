pub mod clock;
pub mod filesystem;
pub mod network;
pub mod threading;

pub use clock::{sleep, sleep_ms, Clock, DeltaTime, FpsCounter, Profiler, Stopwatch, Timer};
pub use filesystem::{
    DirectoryWalker, FileHandle, FileMetadata, FileSystem, FileWatcher, PathUtil,
};
pub use network::{HttpClient, IpAddress, Network, NetworkBuffer, TcpClient, TcpServer, UdpClient};
pub use threading::{
    num_cpus, yield_now, ManagedThread, RwCounter, Semaphore, ShutdownFlag, TaskScheduler,
    ThreadBarrier, ThreadPool,
};

/// Informações sobre o sistema operacional
pub struct SystemInfo;

impl SystemInfo {
    /// Retorna o nome do sistema operacional
    pub fn os_name() -> &'static str {
        std::env::consts::OS
    }

    /// Retorna a arquitetura
    pub fn arch() -> &'static str {
        std::env::consts::ARCH
    }

    /// Retorna a família do OS
    pub fn family() -> &'static str {
        std::env::consts::FAMILY
    }

    /// Verifica se é Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Verifica se é Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    /// Verifica se é macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Verifica se é Unix-like
    pub fn is_unix() -> bool {
        cfg!(unix)
    }

    /// Retorna o número de CPUs
    pub fn num_cpus() -> usize {
        num_cpus()
    }

    /// Retorna o hostname
    pub fn hostname() -> Option<String> {
        Network::hostname()
    }

    /// Retorna o diretório home do usuário
    pub fn home_dir() -> Option<std::path::PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .map(std::path::PathBuf::from)
    }

    /// Retorna o diretório temporário
    pub fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir()
    }

    /// Retorna o diretório de trabalho atual
    pub fn current_dir() -> std::io::Result<std::path::PathBuf> {
        std::env::current_dir()
    }

    /// Retorna o executável atual
    pub fn current_exe() -> std::io::Result<std::path::PathBuf> {
        std::env::current_exe()
    }
}

/// Variáveis de ambiente
pub struct Environment;

impl Environment {
    /// Obtém uma variável de ambiente
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Define uma variável de ambiente
    pub fn set(key: &str, value: &str) {
        std::env::set_var(key, value);
    }

    /// Remove uma variável de ambiente
    pub fn remove(key: &str) {
        std::env::remove_var(key);
    }

    /// Lista todas as variáveis de ambiente
    pub fn all() -> Vec<(String, String)> {
        std::env::vars().collect()
    }

    /// Verifica se uma variável existe
    pub fn exists(key: &str) -> bool {
        std::env::var(key).is_ok()
    }
}

/// Processo
pub struct Process;

impl Process {
    /// Retorna o ID do processo atual
    pub fn id() -> u32 {
        std::process::id()
    }

    /// Termina o processo com código
    pub fn exit(code: i32) -> ! {
        std::process::exit(code)
    }

    /// Aborta o processo
    pub fn abort() -> ! {
        std::process::abort()
    }

    /// Executa um comando
    pub fn spawn(command: &str, args: &[&str]) -> std::io::Result<std::process::Child> {
        std::process::Command::new(command).args(args).spawn()
    }

    /// Executa um comando e aguarda
    pub fn run(command: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
        std::process::Command::new(command).args(args).output()
    }

    /// Executa um comando shell
    #[cfg(unix)]
    pub fn shell(command: &str) -> std::io::Result<std::process::Output> {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
    }

    #[cfg(windows)]
    pub fn shell(command: &str) -> std::io::Result<std::process::Output> {
        std::process::Command::new("cmd")
            .args(&["/C", command])
            .output()
    }
}

/// Console utilities
pub struct Console;

impl Console {
    /// Lê uma linha do stdin
    pub fn read_line() -> std::io::Result<String> {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        Ok(buffer.trim().to_string())
    }

    /// Imprime linha
    pub fn println(text: &str) {
        println!("{}", text);
    }

    /// Imprime sem newline
    pub fn print(text: &str) {
        print!("{}", text);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    /// Limpa a tela (cross-platform)
    pub fn clear() {
        if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(&["/C", "cls"])
                .status()
                .ok();
        } else {
            std::process::Command::new("clear").status().ok();
        }
    }

    /// Define cor do terminal (ANSI - funciona em Unix e Windows 10+)
    pub fn set_color(color: ConsoleColor) {
        print!("{}", color.ansi_code());
    }

    /// Reseta cor
    pub fn reset_color() {
        print!("\x1b[0m");
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl ConsoleColor {
    fn ansi_code(&self) -> &'static str {
        match self {
            ConsoleColor::Black => "\x1b[30m",
            ConsoleColor::Red => "\x1b[31m",
            ConsoleColor::Green => "\x1b[32m",
            ConsoleColor::Yellow => "\x1b[33m",
            ConsoleColor::Blue => "\x1b[34m",
            ConsoleColor::Magenta => "\x1b[35m",
            ConsoleColor::Cyan => "\x1b[36m",
            ConsoleColor::White => "\x1b[37m",
            ConsoleColor::BrightBlack => "\x1b[90m",
            ConsoleColor::BrightRed => "\x1b[91m",
            ConsoleColor::BrightGreen => "\x1b[92m",
            ConsoleColor::BrightYellow => "\x1b[93m",
            ConsoleColor::BrightBlue => "\x1b[94m",
            ConsoleColor::BrightMagenta => "\x1b[95m",
            ConsoleColor::BrightCyan => "\x1b[96m",
            ConsoleColor::BrightWhite => "\x1b[97m",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info() {
        assert!(!SystemInfo::os_name().is_empty());
        assert!(!SystemInfo::arch().is_empty());
        assert!(SystemInfo::num_cpus() > 0);
    }

    #[test]
    fn test_environment() {
        Environment::set("TEST_VAR", "test_value");
        assert_eq!(Environment::get("TEST_VAR"), Some("test_value".to_string()));
        assert!(Environment::exists("TEST_VAR"));

        Environment::remove("TEST_VAR");
        assert!(!Environment::exists("TEST_VAR"));
    }

    #[test]
    fn test_process() {
        assert!(Process::id() > 0);
    }

    #[test]
    fn test_temp_dir() {
        let temp = SystemInfo::temp_dir();
        assert!(temp.exists());
    }
}
