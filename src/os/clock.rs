use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Clock de alta precisão para medição de tempo
pub struct Clock {
    start: Instant,
}

impl Clock {
    /// Cria um novo clock
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Retorna o tempo decorrido desde a criação
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Retorna o tempo em segundos
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    /// Retorna o tempo em milissegundos
    pub fn elapsed_millis(&self) -> u128 {
        self.elapsed().as_millis()
    }

    /// Retorna o tempo em microssegundos
    pub fn elapsed_micros(&self) -> u128 {
        self.elapsed().as_micros()
    }

    /// Retorna o tempo em nanossegundos
    pub fn elapsed_nanos(&self) -> u128 {
        self.elapsed().as_nanos()
    }

    /// Reseta o clock
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Retorna o timestamp atual do sistema
    pub fn now() -> Instant {
        Instant::now()
    }

    /// Retorna o system time atual
    pub fn system_time() -> SystemTime {
        SystemTime::now()
    }

    /// Retorna o timestamp Unix em segundos
    pub fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Retorna o timestamp Unix em milissegundos
    pub fn unix_timestamp_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer para operações temporizadas
pub struct Timer {
    start: Instant,
    duration: Duration,
}

impl Timer {
    /// Cria um timer com duração especificada
    pub fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }

    /// Cria um timer em segundos
    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    /// Cria um timer em milissegundos
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    /// Verifica se o timer expirou
    pub fn expired(&self) -> bool {
        self.start.elapsed() >= self.duration
    }

    /// Retorna o tempo restante
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.start.elapsed())
    }

    /// Retorna o progresso (0.0 a 1.0)
    pub fn progress(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        (elapsed / total).min(1.0)
    }

    /// Reseta o timer
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Aguarda o timer expirar
    pub fn wait(&self) {
        if let Some(remaining) = self.duration.checked_sub(self.start.elapsed()) {
            thread::sleep(remaining);
        }
    }
}

/// Stopwatch para medir intervalos de tempo
pub struct Stopwatch {
    start: Option<Instant>,
    accumulated: Duration,
    running: bool,
}

impl Stopwatch {
    /// Cria um novo stopwatch (parado)
    pub fn new() -> Self {
        Self {
            start: None,
            accumulated: Duration::ZERO,
            running: false,
        }
    }

    /// Inicia o stopwatch
    pub fn start(&mut self) {
        if !self.running {
            self.start = Some(Instant::now());
            self.running = true;
        }
    }

    /// Para o stopwatch
    pub fn stop(&mut self) {
        if self.running {
            if let Some(start) = self.start {
                self.accumulated += start.elapsed();
            }
            self.start = None;
            self.running = false;
        }
    }

    /// Reseta o stopwatch
    pub fn reset(&mut self) {
        self.start = None;
        self.accumulated = Duration::ZERO;
        self.running = false;
    }

    /// Retorna o tempo decorrido
    pub fn elapsed(&self) -> Duration {
        let mut total = self.accumulated;
        if self.running {
            if let Some(start) = self.start {
                total += start.elapsed();
            }
        }
        total
    }

    /// Retorna se está rodando
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Lap - registra um lap e retorna o tempo
    pub fn lap(&mut self) -> Duration {
        let elapsed = self.elapsed();
        self.reset();
        self.start();
        elapsed
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// FPS counter - contador de frames por segundo
pub struct FpsCounter {
    frame_count: u64,
    last_update: Instant,
    current_fps: f64,
    update_interval: Duration,
}

impl FpsCounter {
    /// Cria um novo FPS counter
    pub fn new() -> Self {
        Self::with_interval(Duration::from_secs(1))
    }

    /// Cria com intervalo de atualização customizado
    pub fn with_interval(update_interval: Duration) -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            current_fps: 0.0,
            update_interval,
        }
    }

    /// Registra um frame
    pub fn tick(&mut self) {
        self.frame_count += 1;

        let elapsed = self.last_update.elapsed();
        if elapsed >= self.update_interval {
            self.current_fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_update = Instant::now();
        }
    }

    /// Retorna o FPS atual
    pub fn fps(&self) -> f64 {
        self.current_fps
    }

    /// Reseta o contador
    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.last_update = Instant::now();
        self.current_fps = 0.0;
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta time calculator - calcula tempo entre frames
pub struct DeltaTime {
    last_frame: Instant,
    delta: Duration,
    smoothing_factor: f32,
    smoothed_delta: f32,
}

impl DeltaTime {
    /// Cria um novo delta time calculator
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: Duration::ZERO,
            smoothing_factor: 0.1,
            smoothed_delta: 0.0,
        }
    }

    /// Atualiza e retorna o delta time
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        // Smooth delta
        let current = self.delta.as_secs_f32();
        self.smoothed_delta =
            self.smoothed_delta * (1.0 - self.smoothing_factor) + current * self.smoothing_factor;

        self.delta
    }

    /// Retorna o delta time em segundos
    pub fn as_secs(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Retorna o delta time suavizado em segundos
    pub fn as_secs_smoothed(&self) -> f32 {
        self.smoothed_delta
    }

    /// Retorna o delta time como Duration
    pub fn as_duration(&self) -> Duration {
        self.delta
    }

    /// Define o fator de suavização (0.0 a 1.0)
    pub fn set_smoothing(&mut self, factor: f32) {
        self.smoothing_factor = factor.clamp(0.0, 1.0);
    }
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiler simples para medir performance
pub struct Profiler {
    measurements: std::collections::HashMap<String, Vec<Duration>>,
    current: Option<(String, Instant)>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            measurements: std::collections::HashMap::new(),
            current: None,
        }
    }

    /// Inicia medição de uma seção
    pub fn begin(&mut self, name: impl Into<String>) {
        self.current = Some((name.into(), Instant::now()));
    }

    /// Termina medição e registra
    pub fn end(&mut self) {
        if let Some((name, start)) = self.current.take() {
            let duration = start.elapsed();
            self.measurements
                .entry(name)
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    /// Obtém média de uma medição
    pub fn average(&self, name: &str) -> Option<Duration> {
        self.measurements.get(name).map(|measurements| {
            let sum: Duration = measurements.iter().sum();
            sum / measurements.len() as u32
        })
    }

    /// Obtém todas as médias
    pub fn averages(&self) -> Vec<(String, Duration)> {
        self.measurements
            .iter()
            .map(|(name, measurements)| {
                let sum: Duration = measurements.iter().sum();
                let avg = sum / measurements.len() as u32;
                (name.clone(), avg)
            })
            .collect()
    }

    /// Limpa todas as medições
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.current = None;
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper para sleep
pub fn sleep(duration: Duration) {
    thread::sleep(duration);
}

/// Helper para sleep em milissegundos
pub fn sleep_ms(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        let clock = Clock::new();
        sleep_ms(10);
        assert!(clock.elapsed_millis() >= 10);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::from_millis(50);
        assert!(!timer.expired());
        sleep_ms(60);
        assert!(timer.expired());
    }

    #[test]
    fn test_stopwatch() {
        let mut sw = Stopwatch::new();
        assert!(!sw.is_running());

        sw.start();
        assert!(sw.is_running());
        sleep_ms(10);

        sw.stop();
        assert!(!sw.is_running());
        assert!(sw.elapsed().as_millis() >= 10);
    }

    #[test]
    fn test_fps_counter() {
        let mut fps = FpsCounter::with_interval(Duration::from_millis(100));

        // Fazer mais ticks e aguardar mais tempo para garantir medição
        for _ in 0..20 {
            fps.tick();
            sleep_ms(10);
        }

        // Aguardar um pouco mais para calcular FPS
        sleep_ms(150);
        fps.tick();

        assert!(fps.fps() > 0.0);
    }

    #[test]
    fn test_delta_time() {
        let mut dt = DeltaTime::new();
        sleep_ms(16);
        let delta = dt.update();
        assert!(delta.as_millis() >= 16);
    }
}
