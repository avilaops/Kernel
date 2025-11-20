use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Memory Manager - gerenciador central de memória com estatísticas
/// Coordena múltiplos allocators e fornece visibilidade sobre uso de memória
pub struct MemoryManager {
    stats: MemoryStats,
    allocators: HashMap<String, AllocatorInfo>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            stats: MemoryStats::new(),
            allocators: HashMap::new(),
        }
    }

    /// Registra um allocator para tracking
    pub fn register_allocator(&mut self, name: impl Into<String>, info: AllocatorInfo) {
        self.allocators.insert(name.into(), info);
    }

    /// Obtém estatísticas globais de memória
    pub fn global_stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Obtém estatísticas de um allocator específico
    pub fn allocator_stats(&self, name: &str) -> Option<&AllocatorInfo> {
        self.allocators.get(name)
    }

    /// Obtém todas as estatísticas de allocators
    pub fn all_allocator_stats(&self) -> &HashMap<String, AllocatorInfo> {
        &self.allocators
    }

    /// Gera um relatório de memória
    pub fn report(&self) -> MemoryReport {
        let mut total_allocated = 0;
        let mut total_used = 0;
        let mut total_free = 0;

        for info in self.allocators.values() {
            total_allocated += info.total_capacity;
            total_used += info.used;
            total_free += info.available;
        }

        MemoryReport {
            timestamp: Instant::now(),
            total_allocated,
            total_used,
            total_free,
            allocator_count: self.allocators.len(),
            allocators: self.allocators.clone(),
        }
    }

    /// Limpa estatísticas
    pub fn reset_stats(&mut self) {
        self.stats.reset();
        self.allocators.clear();
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estatísticas globais de memória
pub struct MemoryStats {
    total_allocations: AtomicUsize,
    total_deallocations: AtomicUsize,
    total_bytes_allocated: AtomicUsize,
    total_bytes_deallocated: AtomicUsize,
    peak_memory_usage: AtomicUsize,
    current_memory_usage: AtomicUsize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            total_allocations: AtomicUsize::new(0),
            total_deallocations: AtomicUsize::new(0),
            total_bytes_allocated: AtomicUsize::new(0),
            total_bytes_deallocated: AtomicUsize::new(0),
            peak_memory_usage: AtomicUsize::new(0),
            current_memory_usage: AtomicUsize::new(0),
        }
    }

    pub fn record_allocation(&self, size: usize) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_bytes_allocated.fetch_add(size, Ordering::Relaxed);

        let current = self.current_memory_usage.fetch_add(size, Ordering::Relaxed) + size;

        // Atualiza o pico se necessário
        let mut peak = self.peak_memory_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_memory_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_bytes_deallocated.fetch_add(size, Ordering::Relaxed);
        self.current_memory_usage.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn total_allocations(&self) -> usize {
        self.total_allocations.load(Ordering::Relaxed)
    }

    pub fn total_deallocations(&self) -> usize {
        self.total_deallocations.load(Ordering::Relaxed)
    }

    pub fn total_bytes_allocated(&self) -> usize {
        self.total_bytes_allocated.load(Ordering::Relaxed)
    }

    pub fn total_bytes_deallocated(&self) -> usize {
        self.total_bytes_deallocated.load(Ordering::Relaxed)
    }

    pub fn current_memory_usage(&self) -> usize {
        self.current_memory_usage.load(Ordering::Relaxed)
    }

    pub fn peak_memory_usage(&self) -> usize {
        self.peak_memory_usage.load(Ordering::Relaxed)
    }

    pub fn active_allocations(&self) -> usize {
        self.total_allocations() - self.total_deallocations()
    }

    pub fn reset(&self) {
        self.total_allocations.store(0, Ordering::Relaxed);
        self.total_deallocations.store(0, Ordering::Relaxed);
        self.total_bytes_allocated.store(0, Ordering::Relaxed);
        self.total_bytes_deallocated.store(0, Ordering::Relaxed);
        self.peak_memory_usage.store(0, Ordering::Relaxed);
        self.current_memory_usage.store(0, Ordering::Relaxed);
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Informações sobre um allocator
#[derive(Debug, Clone)]
pub struct AllocatorInfo {
    pub allocator_type: AllocatorType,
    pub total_capacity: usize,
    pub used: usize,
    pub available: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

impl AllocatorInfo {
    pub fn utilization(&self) -> f32 {
        if self.total_capacity == 0 {
            return 0.0;
        }
        (self.used as f32 / self.total_capacity as f32) * 100.0
    }

    pub fn active_allocations(&self) -> usize {
        self.allocation_count.saturating_sub(self.deallocation_count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocatorType {
    Arena,
    Pool,
    Stack,
    DoubleEndedStack,
    Custom,
}

/// Relatório de memória em um ponto no tempo
#[derive(Debug, Clone)]
pub struct MemoryReport {
    pub timestamp: Instant,
    pub total_allocated: usize,
    pub total_used: usize,
    pub total_free: usize,
    pub allocator_count: usize,
    pub allocators: HashMap<String, AllocatorInfo>,
}

impl MemoryReport {
    pub fn utilization(&self) -> f32 {
        if self.total_allocated == 0 {
            return 0.0;
        }
        (self.total_used as f32 / self.total_allocated as f32) * 100.0
    }

    pub fn print_summary(&self) {
        println!("=== Memory Report ===");
        println!("Total Allocated: {} bytes ({:.2} MB)",
            self.total_allocated,
            self.total_allocated as f64 / (1024.0 * 1024.0)
        );
        println!("Total Used: {} bytes ({:.2} MB)",
            self.total_used,
            self.total_used as f64 / (1024.0 * 1024.0)
        );
        println!("Total Free: {} bytes ({:.2} MB)",
            self.total_free,
            self.total_free as f64 / (1024.0 * 1024.0)
        );
        println!("Utilization: {:.2}%", self.utilization());
        println!("Allocators: {}", self.allocator_count);
        println!();

        for (name, info) in &self.allocators {
            println!("  {}: {:?}", name, info.allocator_type);
            println!("    Capacity: {} bytes", info.total_capacity);
            println!("    Used: {} bytes ({:.2}%)", info.used, info.utilization());
            println!("    Available: {} bytes", info.available);
            println!("    Active Allocations: {}", info.active_allocations());
            println!();
        }
    }

    pub fn to_json(&self) -> String {
        // Implementação simples - em produção usaria serde
        format!(
            r#"{{
  "timestamp": "{:?}",
  "total_allocated": {},
  "total_used": {},
  "total_free": {},
  "utilization": {:.2},
  "allocator_count": {}
}}"#,
            self.timestamp,
            self.total_allocated,
            self.total_used,
            self.total_free,
            self.utilization(),
            self.allocator_count
        )
    }
}

/// Profiler de memória - registra operações ao longo do tempo
pub struct MemoryProfiler {
    samples: Vec<MemorySample>,
    sample_interval: Duration,
    last_sample: Option<Instant>,
}

impl MemoryProfiler {
    pub fn new(sample_interval: Duration) -> Self {
        Self {
            samples: Vec::new(),
            sample_interval,
            last_sample: None,
        }
    }

    pub fn sample(&mut self, stats: &MemoryStats) {
        let now = Instant::now();

        if let Some(last) = self.last_sample {
            if now.duration_since(last) < self.sample_interval {
                return;
            }
        }

        self.samples.push(MemorySample {
            timestamp: now,
            current_usage: stats.current_memory_usage(),
            peak_usage: stats.peak_memory_usage(),
            total_allocated: stats.total_bytes_allocated(),
            total_deallocated: stats.total_bytes_deallocated(),
            active_allocations: stats.active_allocations(),
        });

        self.last_sample = Some(now);
    }

    pub fn samples(&self) -> &[MemorySample] {
        &self.samples
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.last_sample = None;
    }

    pub fn average_usage(&self) -> Option<usize> {
        if self.samples.is_empty() {
            return None;
        }

        let sum: usize = self.samples.iter().map(|s| s.current_usage).sum();
        Some(sum / self.samples.len())
    }

    pub fn peak_usage(&self) -> Option<usize> {
        self.samples.iter().map(|s| s.peak_usage).max()
    }
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub active_allocations: usize,
}

/// Utilitários para formatação de tamanhos
pub mod format {
    pub fn bytes(bytes: usize) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        let bytes_f = bytes as f64;

        if bytes_f >= GB {
            format!("{:.2} GB", bytes_f / GB)
        } else if bytes_f >= MB {
            format!("{:.2} MB", bytes_f / MB)
        } else if bytes_f >= KB {
            format!("{:.2} KB", bytes_f / KB)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::new();

        stats.record_allocation(1024);
        assert_eq!(stats.total_allocations(), 1);
        assert_eq!(stats.current_memory_usage(), 1024);

        stats.record_deallocation(512);
        assert_eq!(stats.total_deallocations(), 1);
        assert_eq!(stats.current_memory_usage(), 512);
    }

    #[test]
    fn test_peak_memory() {
        let stats = MemoryStats::new();

        stats.record_allocation(1000);
        stats.record_allocation(500);
        assert_eq!(stats.peak_memory_usage(), 1500);

        stats.record_deallocation(1000);
        assert_eq!(stats.current_memory_usage(), 500);
        assert_eq!(stats.peak_memory_usage(), 1500); // Peak não muda
    }

    #[test]
    fn test_memory_manager() {
        let mut manager = MemoryManager::new();

        manager.register_allocator("arena1", AllocatorInfo {
            allocator_type: AllocatorType::Arena,
            total_capacity: 1024,
            used: 512,
            available: 512,
            allocation_count: 10,
            deallocation_count: 5,
        });

        let report = manager.report();
        assert_eq!(report.allocator_count, 1);
        assert_eq!(report.total_allocated, 1024);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format::bytes(512), "512 bytes");
        assert_eq!(format::bytes(2048), "2.00 KB");
        assert_eq!(format::bytes(2 * 1024 * 1024), "2.00 MB");
    }
}
