use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Thread pool para execução paralela de tarefas
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Job>,
    active_jobs: Arc<AtomicUsize>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Cria um novo thread pool com o número especificado de threads
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Thread pool size must be greater than 0");

        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let active_jobs = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&active_jobs),
            ));
        }

        ThreadPool {
            workers,
            sender,
            active_jobs,
        }
    }

    /// Cria um thread pool com número de threads baseado nos CPUs disponíveis
    pub fn new_with_cpus() -> Self {
        let size = num_cpus();
        Self::new(size)
    }

    /// Executa uma tarefa no thread pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .send(job)
            .expect("Failed to send job to thread pool");
    }

    /// Retorna o número de threads no pool
    pub fn size(&self) -> usize {
        self.workers.len()
    }

    /// Retorna o número de jobs ativos
    pub fn active_jobs(&self) -> usize {
        self.active_jobs.load(Ordering::Relaxed)
    }

    /// Aguarda todas as tarefas terminarem
    pub fn join(&self) {
        while self.active_jobs() > 0 {
            thread::yield_now();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Aguarda todos os jobs terminarem
        self.join();
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>,
        active_jobs: Arc<AtomicUsize>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = {
                let receiver = receiver.lock().unwrap();
                receiver.recv()
            };

            match job {
                Ok(job) => {
                    active_jobs.fetch_add(1, Ordering::Relaxed);
                    job();
                    active_jobs.fetch_sub(1, Ordering::Relaxed);
                }
                Err(_) => break,
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// Task scheduler para execução assíncrona
pub struct TaskScheduler {
    pool: ThreadPool,
    tasks: Arc<Mutex<Vec<Task>>>,
}

struct Task {
    name: String,
    job: Job,
    priority: u8,
}

impl TaskScheduler {
    pub fn new(num_threads: usize) -> Self {
        Self {
            pool: ThreadPool::new(num_threads),
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn schedule<F>(&self, name: impl Into<String>, priority: u8, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task {
            name: name.into(),
            job: Box::new(f),
            priority,
        };

        self.tasks.lock().unwrap().push(task);
    }

    pub fn run(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        while let Some(task) = tasks.pop() {
            self.pool.execute(task.job);
        }
    }
}

/// Thread handle com nome e metadata
pub struct ManagedThread {
    handle: Option<JoinHandle<()>>,
    name: String,
    id: usize,
}

impl ManagedThread {
    pub fn spawn<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let name = name.into();
        let builder = thread::Builder::new().name(name.clone());
        let handle = builder.spawn(f).expect("Failed to spawn thread");

        // Hash do ThreadId como alternativa ao as_u64() que está unstable
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        handle.thread().id().hash(&mut hasher);
        let id = hasher.finish() as usize;

        Self {
            handle: Some(handle),
            name,
            id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn join(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
    }
}

/// Sincronização avançada - Semaphore
pub struct Semaphore {
    permits: Arc<Mutex<usize>>,
    condvar: Arc<Condvar>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            permits: Arc::new(Mutex::new(permits)),
            condvar: Arc::new(Condvar::new()),
        }
    }

    pub fn acquire(&self) {
        let mut permits = self.permits.lock().unwrap();
        while *permits == 0 {
            permits = self.condvar.wait(permits).unwrap();
        }
        *permits -= 1;
    }

    pub fn try_acquire(&self) -> bool {
        let mut permits = self.permits.lock().unwrap();
        if *permits > 0 {
            *permits -= 1;
            true
        } else {
            false
        }
    }

    pub fn release(&self) {
        let mut permits = self.permits.lock().unwrap();
        *permits += 1;
        self.condvar.notify_one();
    }

    pub fn available(&self) -> usize {
        *self.permits.lock().unwrap()
    }
}

/// Read-Write Lock com contador de leitores
pub struct RwCounter<T> {
    data: RwLock<T>,
    readers: AtomicUsize,
    writers: AtomicUsize,
}

impl<T> RwCounter<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
            readers: AtomicUsize::new(0),
            writers: AtomicUsize::new(0),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
        self.readers.fetch_add(1, Ordering::Relaxed);
        let guard = self.data.read().unwrap();
        self.readers.fetch_sub(1, Ordering::Relaxed);
        guard
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<T> {
        self.writers.fetch_add(1, Ordering::Relaxed);
        let guard = self.data.write().unwrap();
        self.writers.fetch_sub(1, Ordering::Relaxed);
        guard
    }

    pub fn active_readers(&self) -> usize {
        self.readers.load(Ordering::Relaxed)
    }

    pub fn active_writers(&self) -> usize {
        self.writers.load(Ordering::Relaxed)
    }
}

/// Barreira para sincronização de múltiplas threads
pub struct ThreadBarrier {
    barrier: Arc<Barrier>,
}

impl ThreadBarrier {
    pub fn new(n: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(n)),
        }
    }

    pub fn wait(&self) {
        self.barrier.wait();
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            barrier: Arc::clone(&self.barrier),
        }
    }
}

/// Flag atômica para shutdown
pub struct ShutdownFlag {
    flag: Arc<AtomicBool>,
}

impl ShutdownFlag {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn shutdown(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            flag: Arc::clone(&self.flag),
        }
    }
}

impl Default for ShutdownFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// Retorna o número de CPUs/cores disponíveis
pub fn num_cpus() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Sleep helper
pub fn sleep(duration: Duration) {
    thread::sleep(duration);
}

/// Yield helper
pub fn yield_now() {
    thread::yield_now();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }

        // Aguardar um pouco para garantir que as tasks sejam executadas
        thread::sleep(Duration::from_millis(100));
        pool.join();
        assert_eq!(counter.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_semaphore() {
        let sem = Semaphore::new(2);

        assert_eq!(sem.available(), 2);
        sem.acquire();
        assert_eq!(sem.available(), 1);
        sem.acquire();
        assert_eq!(sem.available(), 0);

        sem.release();
        assert_eq!(sem.available(), 1);
    }

    #[test]
    fn test_managed_thread() {
        let thread = ManagedThread::spawn("test_thread", || {
            sleep(Duration::from_millis(10));
        });

        assert_eq!(thread.name(), "test_thread");
        thread.join();
    }

    #[test]
    fn test_shutdown_flag() {
        let flag = ShutdownFlag::new();
        assert!(!flag.is_shutdown());

        flag.shutdown();
        assert!(flag.is_shutdown());
    }
}
