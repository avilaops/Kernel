use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ptr::NonNull;

/// Arena Allocator - aloca memória sequencialmente de um bloco pré-alocado
/// Ideal para alocações temporárias que são liberadas todas de uma vez
///
/// Características:
/// - Alocação O(1) extremamente rápida (apenas incrementa ponteiro)
/// - Não suporta free individual, apenas reset completo
/// - Excelente localidade de cache
/// - Perfeito para frames em game engines, parsing temporário, etc.
pub struct Arena {
    buffer: NonNull<u8>,
    capacity: usize,
    offset: Cell<usize>,
    layout: Layout,
}

impl Arena {
    /// Cria uma nova arena com a capacidade especificada (em bytes)
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Arena capacity must be greater than 0");

        let layout =
            Layout::from_size_align(capacity, 16).expect("Failed to create layout for arena");

        let buffer = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate arena memory");
            }
            NonNull::new_unchecked(ptr)
        };

        Self {
            buffer,
            capacity,
            offset: Cell::new(0),
            layout,
        }
    }

    /// Cria uma arena com capacidade padrão de 1MB
    pub fn with_default_capacity() -> Self {
        Self::new(1024 * 1024) // 1MB
    }

    /// Aloca um bloco de memória com o tamanho e alinhamento especificados
    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let current_offset = self.offset.get();

        // Calcula o offset alinhado
        let aligned_offset = align_up(current_offset, align);
        let new_offset = aligned_offset.checked_add(size)?;

        if new_offset > self.capacity {
            return None; // Arena cheia
        }

        self.offset.set(new_offset);

        unsafe {
            let ptr = self.buffer.as_ptr().add(aligned_offset);
            Some(NonNull::new_unchecked(ptr))
        }
    }

    /// Aloca memória para um tipo específico
    pub fn alloc_type<T>(&self) -> Option<NonNull<T>> {
        let layout = Layout::new::<T>();
        self.alloc(layout.size(), layout.align())
            .map(|ptr| ptr.cast::<T>())
    }

    /// Aloca um slice de um tipo específico
    pub fn alloc_slice<T>(&self, count: usize) -> Option<NonNull<[T]>> {
        if count == 0 {
            return Some(NonNull::slice_from_raw_parts(NonNull::dangling(), 0));
        }

        let layout = Layout::array::<T>(count).ok()?;
        self.alloc(layout.size(), layout.align())
            .map(|ptr| NonNull::slice_from_raw_parts(ptr.cast::<T>(), count))
    }

    /// Reseta a arena, permitindo reutilização da memória
    /// ATENÇÃO: Não chama destructors! Use apenas com tipos Copy ou que não precisam de cleanup
    pub fn reset(&self) {
        self.offset.set(0);
    }

    /// Retorna a quantidade de memória usada (em bytes)
    pub fn used(&self) -> usize {
        self.offset.get()
    }

    /// Retorna a capacidade total da arena (em bytes)
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Retorna a quantidade de memória disponível (em bytes)
    pub fn available(&self) -> usize {
        self.capacity - self.used()
    }

    /// Retorna a porcentagem de utilização da arena
    pub fn utilization(&self) -> f32 {
        (self.used() as f32 / self.capacity as f32) * 100.0
    }

    /// Cria um checkpoint que pode ser usado para liberar memória até esse ponto
    pub fn checkpoint(&self) -> ArenaCheckpoint {
        ArenaCheckpoint {
            offset: self.offset.get(),
        }
    }

    /// Restaura a arena para um checkpoint anterior
    pub fn restore(&self, checkpoint: ArenaCheckpoint) {
        assert!(
            checkpoint.offset <= self.offset.get(),
            "Cannot restore to a checkpoint beyond current offset"
        );
        self.offset.set(checkpoint.offset);
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.buffer.as_ptr(), self.layout);
        }
    }
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

/// Checkpoint para restaurar a arena a um estado anterior
#[derive(Debug, Clone, Copy)]
pub struct ArenaCheckpoint {
    offset: usize,
}

/// Arena com escopo automático - reseta ao sair do escopo
pub struct ScopedArena<'a> {
    arena: &'a Arena,
    checkpoint: ArenaCheckpoint,
}

impl<'a> ScopedArena<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        let checkpoint = arena.checkpoint();
        Self { arena, checkpoint }
    }

    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        self.arena.alloc(size, align)
    }

    pub fn alloc_type<T>(&self) -> Option<NonNull<T>> {
        self.arena.alloc_type::<T>()
    }

    pub fn alloc_slice<T>(&self, count: usize) -> Option<NonNull<[T]>> {
        self.arena.alloc_slice::<T>(count)
    }
}

impl<'a> Drop for ScopedArena<'a> {
    fn drop(&mut self) {
        self.arena.restore(self.checkpoint);
    }
}

/// Alinha um valor para cima ao múltiplo mais próximo de align
#[inline]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let arena = Arena::new(1024);
        assert_eq!(arena.capacity(), 1024);
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.available(), 1024);
    }

    #[test]
    fn test_arena_alloc() {
        let arena = Arena::new(1024);

        let ptr1 = arena.alloc(16, 4);
        assert!(ptr1.is_some());
        assert_eq!(arena.used(), 16);

        let ptr2 = arena.alloc(32, 8);
        assert!(ptr2.is_some());
        assert!(arena.used() >= 48); // Pode ter padding
    }

    #[test]
    fn test_arena_alloc_type() {
        let arena = Arena::new(1024);

        let ptr: Option<NonNull<u64>> = arena.alloc_type();
        assert!(ptr.is_some());
        assert!(arena.used() >= std::mem::size_of::<u64>());
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new(1024);

        arena.alloc(100, 4);
        assert_eq!(arena.used(), 100);

        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn test_arena_checkpoint() {
        let arena = Arena::new(1024);

        arena.alloc(100, 4);
        let checkpoint = arena.checkpoint();

        arena.alloc(50, 4);
        assert!(arena.used() >= 150);

        arena.restore(checkpoint);
        assert_eq!(arena.used(), 100);
    }

    #[test]
    fn test_scoped_arena() {
        let arena = Arena::new(1024);

        arena.alloc(100, 4);
        let used_before = arena.used();

        {
            let scoped = ScopedArena::new(&arena);
            scoped.alloc(50, 4);
            assert!(arena.used() > used_before);
        }

        // Após o escopo, deve ter restaurado
        assert_eq!(arena.used(), used_before);
    }

    #[test]
    fn test_arena_full() {
        let arena = Arena::new(64);

        let ptr1 = arena.alloc(32, 1);
        assert!(ptr1.is_some());

        let ptr2 = arena.alloc(32, 1);
        assert!(ptr2.is_some());

        let ptr3 = arena.alloc(32, 1);
        assert!(ptr3.is_none()); // Arena cheia
    }
}
