use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Pool Allocator - gerencia blocos de tamanho fixo
/// Ideal para alocações frequentes de objetos do mesmo tamanho
///
/// Características:
/// - Alocação e liberação O(1)
/// - Zero fragmentação para objetos de tamanho fixo
/// - Excelente para gerenciar entidades, partículas, etc.
/// - Cache-friendly com memória contígua
pub struct Pool {
    chunk_size: usize,
    chunk_align: usize,
    chunks_per_block: usize,
    blocks: RefCell<Vec<PoolBlock>>,
    free_list: RefCell<Vec<NonNull<u8>>>,
    total_allocated: AtomicUsize,
    total_freed: AtomicUsize,
}

struct PoolBlock {
    memory: NonNull<u8>,
    layout: Layout,
}

impl Pool {
    /// Cria um novo pool para objetos de tamanho fixo
    ///
    /// # Argumentos
    /// * `chunk_size` - Tamanho de cada objeto
    /// * `chunk_align` - Alinhamento de cada objeto
    /// * `chunks_per_block` - Quantos objetos por bloco alocado
    pub fn new(chunk_size: usize, chunk_align: usize, chunks_per_block: usize) -> Self {
        assert!(chunk_size > 0, "Chunk size must be greater than 0");
        assert!(
            chunk_align.is_power_of_two(),
            "Alignment must be power of 2"
        );
        assert!(
            chunks_per_block > 0,
            "Chunks per block must be greater than 0"
        );

        Self {
            chunk_size,
            chunk_align,
            chunks_per_block,
            blocks: RefCell::new(Vec::new()),
            free_list: RefCell::new(Vec::new()),
            total_allocated: AtomicUsize::new(0),
            total_freed: AtomicUsize::new(0),
        }
    }

    /// Cria um pool para um tipo específico
    pub fn for_type<T>(chunks_per_block: usize) -> Self {
        Self::new(
            std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
            chunks_per_block,
        )
    }

    /// Aloca um chunk do pool
    pub fn alloc(&self) -> Option<NonNull<u8>> {
        // Tenta pegar da free list
        if let Some(ptr) = self.free_list.borrow_mut().pop() {
            self.total_allocated.fetch_add(1, Ordering::Relaxed);
            return Some(ptr);
        }

        // Se não tem na free list, aloca um novo bloco
        self.allocate_new_block();

        // Tenta novamente
        if let Some(ptr) = self.free_list.borrow_mut().pop() {
            self.total_allocated.fetch_add(1, Ordering::Relaxed);
            return Some(ptr);
        }

        None
    }

    /// Aloca um chunk do tipo específico
    pub fn alloc_type<T>(&self) -> Option<NonNull<T>> {
        assert_eq!(std::mem::size_of::<T>(), self.chunk_size);
        assert_eq!(std::mem::align_of::<T>(), self.chunk_align);

        self.alloc().map(|ptr| ptr.cast::<T>())
    }

    /// Libera um chunk de volta para o pool
    ///
    /// # Safety
    /// O ponteiro deve ter sido alocado por este pool
    pub unsafe fn free(&self, ptr: NonNull<u8>) {
        self.free_list.borrow_mut().push(ptr);
        self.total_freed.fetch_add(1, Ordering::Relaxed);
    }

    /// Libera um chunk de tipo específico
    ///
    /// # Safety
    /// O ponteiro deve ter sido alocado por este pool
    pub unsafe fn free_type<T>(&self, ptr: NonNull<T>) {
        self.free(ptr.cast::<u8>());
    }

    /// Aloca um novo bloco de memória e adiciona chunks à free list
    fn allocate_new_block(&self) {
        let block_size = self.chunk_size * self.chunks_per_block;
        let layout =
            Layout::from_size_align(block_size, self.chunk_align).expect("Failed to create layout");

        unsafe {
            let memory = alloc(layout);
            if memory.is_null() {
                panic!("Failed to allocate pool block");
            }

            let memory_ptr = NonNull::new_unchecked(memory);

            // Adiciona todos os chunks deste bloco à free list
            let mut free_list = self.free_list.borrow_mut();
            for i in 0..self.chunks_per_block {
                let chunk_ptr = memory.add(i * self.chunk_size);
                free_list.push(NonNull::new_unchecked(chunk_ptr));
            }

            // Guarda o bloco para fazer cleanup depois
            self.blocks.borrow_mut().push(PoolBlock {
                memory: memory_ptr,
                layout,
            });
        }
    }

    /// Retorna estatísticas do pool
    pub fn stats(&self) -> PoolStats {
        let allocated = self.total_allocated.load(Ordering::Relaxed);
        let freed = self.total_freed.load(Ordering::Relaxed);
        let in_use = allocated - freed;
        let free_chunks = self.free_list.borrow().len();
        let total_chunks = self.blocks.borrow().len() * self.chunks_per_block;

        PoolStats {
            chunk_size: self.chunk_size,
            chunks_per_block: self.chunks_per_block,
            total_blocks: self.blocks.borrow().len(),
            total_chunks,
            chunks_in_use: in_use,
            chunks_free: free_chunks,
            total_allocated: allocated,
            total_freed: freed,
            memory_used: in_use * self.chunk_size,
            memory_reserved: total_chunks * self.chunk_size,
        }
    }

    /// Limpa todos os blocos vazios (mantém pelo menos um)
    pub fn shrink_to_fit(&self) {
        // Implementação simplificada - em produção seria mais sofisticado
        // mantendo track de quais blocos estão completamente vazios
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            for block in self.blocks.borrow_mut().drain(..) {
                dealloc(block.memory.as_ptr(), block.layout);
            }
        }
    }
}

unsafe impl Send for Pool {}
unsafe impl Sync for Pool {}

/// Estatísticas de um pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub chunk_size: usize,
    pub chunks_per_block: usize,
    pub total_blocks: usize,
    pub total_chunks: usize,
    pub chunks_in_use: usize,
    pub chunks_free: usize,
    pub total_allocated: usize,
    pub total_freed: usize,
    pub memory_used: usize,
    pub memory_reserved: usize,
}

impl PoolStats {
    pub fn utilization(&self) -> f32 {
        if self.total_chunks == 0 {
            return 0.0;
        }
        (self.chunks_in_use as f32 / self.total_chunks as f32) * 100.0
    }

    pub fn fragmentation(&self) -> f32 {
        if self.total_chunks == 0 {
            return 0.0;
        }
        (self.chunks_free as f32 / self.total_chunks as f32) * 100.0
    }
}

/// Pool com tipo específico - wrapper type-safe sobre Pool
pub struct TypedPool<T> {
    pool: Pool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TypedPool<T> {
    pub fn new(chunks_per_block: usize) -> Self {
        Self {
            pool: Pool::for_type::<T>(chunks_per_block),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn alloc(&self) -> Option<NonNull<T>> {
        self.pool.alloc_type::<T>()
    }

    /// # Safety
    /// O ponteiro deve ter sido alocado por este pool
    pub unsafe fn free(&self, ptr: NonNull<T>) {
        self.pool.free_type(ptr);
    }

    pub fn stats(&self) -> PoolStats {
        self.pool.stats()
    }
}

/// Helper para criar e destruir objetos no pool
pub struct PoolBox<'a, T> {
    ptr: NonNull<T>,
    pool: &'a Pool,
}

impl<'a, T> PoolBox<'a, T> {
    pub fn new(pool: &'a Pool, value: T) -> Option<Self> {
        let ptr = pool.alloc_type::<T>()?;
        unsafe {
            ptr.as_ptr().write(value);
        }
        Some(Self { ptr, pool })
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<'a, T> std::ops::Deref for PoolBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T> std::ops::DerefMut for PoolBox<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, T> Drop for PoolBox<'a, T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.ptr.as_ptr());
            self.pool.free_type(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = Pool::new(32, 8, 16);
        let stats = pool.stats();
        assert_eq!(stats.chunk_size, 32);
        assert_eq!(stats.chunks_per_block, 16);
    }

    #[test]
    fn test_pool_alloc_free() {
        let pool = Pool::for_type::<u64>(16);

        let ptr1 = pool.alloc_type::<u64>();
        assert!(ptr1.is_some());

        let ptr2 = pool.alloc_type::<u64>();
        assert!(ptr2.is_some());

        unsafe {
            pool.free_type(ptr1.unwrap());
            pool.free_type(ptr2.unwrap());
        }

        let stats = pool.stats();
        assert_eq!(stats.chunks_in_use, 0);
    }

    #[test]
    fn test_typed_pool() {
        let pool = TypedPool::<i32>::new(16);

        let ptr = pool.alloc();
        assert!(ptr.is_some());

        unsafe {
            pool.free(ptr.unwrap());
        }
    }

    #[test]
    fn test_pool_box() {
        let pool = Pool::for_type::<i32>(16);

        {
            let boxed = PoolBox::new(&pool, 42);
            assert!(boxed.is_some());
            let boxed = boxed.unwrap();
            assert_eq!(*boxed, 42);
        }

        let stats = pool.stats();
        assert_eq!(stats.chunks_in_use, 0);
    }

    #[test]
    fn test_pool_stats() {
        let pool = Pool::for_type::<u64>(10);

        let mut ptrs = Vec::new();
        for _ in 0..5 {
            ptrs.push(pool.alloc_type::<u64>().unwrap());
        }

        let stats = pool.stats();
        assert_eq!(stats.chunks_in_use, 5);
        assert!(stats.utilization() > 0.0);

        unsafe {
            for ptr in ptrs {
                pool.free_type(ptr);
            }
        }
    }
}
