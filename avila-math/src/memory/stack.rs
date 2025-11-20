use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::cell::Cell;

/// Stack Allocator - aloca memória em estilo LIFO (Last In First Out)
/// Ideal para alocações hierárquicas onde a ordem de liberação é previsível
///
/// Características:
/// - Alocação O(1) muito rápida
/// - Liberação O(1) apenas na ordem correta (LIFO)
/// - Excelente localidade de cache
/// - Perfeito para call stacks, processamento hierárquico, etc.
pub struct StackAllocator {
    buffer: NonNull<u8>,
    capacity: usize,
    offset: Cell<usize>,
    layout: Layout,
    markers: Cell<Vec<StackMarker>>,
}

#[derive(Debug, Clone, Copy)]
struct StackMarker {
    offset: usize,
}

impl StackAllocator {
    /// Cria um novo stack allocator com a capacidade especificada
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Stack capacity must be greater than 0");

        let layout = Layout::from_size_align(capacity, 16)
            .expect("Failed to create layout for stack");

        let buffer = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate stack memory");
            }
            NonNull::new_unchecked(ptr)
        };

        Self {
            buffer,
            capacity,
            offset: Cell::new(0),
            layout,
            markers: Cell::new(Vec::new()),
        }
    }

    /// Cria um stack com capacidade padrão de 512KB
    pub fn with_default_capacity() -> Self {
        Self::new(512 * 1024) // 512KB
    }

    /// Aloca memória na stack
    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let current_offset = self.offset.get();

        // Calcula o offset alinhado
        let aligned_offset = align_up(current_offset, align);

        // Adiciona header para guardar informações da alocação
        let header_size = std::mem::size_of::<AllocationHeader>();
        let header_offset = aligned_offset;
        let data_offset = header_offset + header_size;

        let new_offset = data_offset.checked_add(size)?;

        if new_offset > self.capacity {
            return None; // Stack overflow
        }

        // Escreve o header
        unsafe {
            let header_ptr = self.buffer.as_ptr().add(header_offset) as *mut AllocationHeader;
            header_ptr.write(AllocationHeader {
                size,
                prev_offset: current_offset,
            });
        }

        self.offset.set(new_offset);

        unsafe {
            let ptr = self.buffer.as_ptr().add(data_offset);
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

    /// Libera a última alocação (LIFO)
    ///
    /// # Safety
    /// Deve ser chamado na ordem inversa das alocações (LIFO)
    pub unsafe fn free(&self, ptr: NonNull<u8>) {
        let data_offset = ptr.as_ptr() as usize - self.buffer.as_ptr() as usize;
        let header_size = std::mem::size_of::<AllocationHeader>();
        let header_offset = data_offset - header_size;

        let header_ptr = self.buffer.as_ptr().add(header_offset) as *const AllocationHeader;
        let header = header_ptr.read();

        // Verifica se é a alocação no topo da stack
        debug_assert!(
            data_offset + header.size == self.offset.get(),
            "Attempted to free allocation that is not at the top of the stack"
        );

        self.offset.set(header.prev_offset);
    }

    /// Cria um marcador para a posição atual da stack
    pub fn mark(&self) -> StackMark {
        StackMark {
            offset: self.offset.get(),
        }
    }

    /// Libera tudo até o marcador especificado
    pub fn free_to_mark(&self, mark: StackMark) {
        assert!(
            mark.offset <= self.offset.get(),
            "Cannot free to a mark beyond current offset"
        );
        self.offset.set(mark.offset);
    }

    /// Limpa toda a stack
    pub fn clear(&self) {
        self.offset.set(0);
    }

    /// Retorna a quantidade de memória usada (em bytes)
    pub fn used(&self) -> usize {
        self.offset.get()
    }

    /// Retorna a capacidade total da stack (em bytes)
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Retorna a quantidade de memória disponível (em bytes)
    pub fn available(&self) -> usize {
        self.capacity - self.used()
    }

    /// Retorna a porcentagem de utilização
    pub fn utilization(&self) -> f32 {
        (self.used() as f32 / self.capacity as f32) * 100.0
    }
}

impl Drop for StackAllocator {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.buffer.as_ptr(), self.layout);
        }
    }
}

unsafe impl Send for StackAllocator {}
unsafe impl Sync for StackAllocator {}

/// Header armazenado antes de cada alocação
#[repr(C)]
struct AllocationHeader {
    size: usize,
    prev_offset: usize,
}

/// Marcador para uma posição na stack
#[derive(Debug, Clone, Copy)]
pub struct StackMark {
    offset: usize,
}

/// Stack com escopo automático - libera ao sair do escopo
pub struct ScopedStack<'a> {
    stack: &'a StackAllocator,
    mark: StackMark,
}

impl<'a> ScopedStack<'a> {
    pub fn new(stack: &'a StackAllocator) -> Self {
        let mark = stack.mark();
        Self { stack, mark }
    }

    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        self.stack.alloc(size, align)
    }

    pub fn alloc_type<T>(&self) -> Option<NonNull<T>> {
        self.stack.alloc_type::<T>()
    }

    pub fn alloc_slice<T>(&self, count: usize) -> Option<NonNull<[T]>> {
        self.stack.alloc_slice::<T>(count)
    }
}

impl<'a> Drop for ScopedStack<'a> {
    fn drop(&mut self) {
        self.stack.free_to_mark(self.mark);
    }
}

/// Alinha um valor para cima ao múltiplo mais próximo de align
#[inline]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Double-ended stack - cresce dos dois lados
/// Útil para separar alocações temporárias de diferentes tipos
pub struct DoubleEndedStack {
    buffer: NonNull<u8>,
    capacity: usize,
    bottom_offset: Cell<usize>,
    top_offset: Cell<usize>,
    layout: Layout,
}

impl DoubleEndedStack {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Stack capacity must be greater than 0");

        let layout = Layout::from_size_align(capacity, 16)
            .expect("Failed to create layout for double-ended stack");

        let buffer = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate double-ended stack memory");
            }
            NonNull::new_unchecked(ptr)
        };

        Self {
            buffer,
            capacity,
            bottom_offset: Cell::new(0),
            top_offset: Cell::new(capacity),
            layout,
        }
    }

    /// Aloca do começo (bottom)
    pub fn alloc_bottom(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let current = self.bottom_offset.get();
        let aligned = align_up(current, align);
        let new_offset = aligned.checked_add(size)?;

        if new_offset >= self.top_offset.get() {
            return None; // Colidiu com o topo
        }

        self.bottom_offset.set(new_offset);

        unsafe {
            let ptr = self.buffer.as_ptr().add(aligned);
            Some(NonNull::new_unchecked(ptr))
        }
    }

    /// Aloca do final (top)
    pub fn alloc_top(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let current = self.top_offset.get();
        let new_offset = current.checked_sub(size)?;
        let aligned = new_offset & !(align - 1);

        if aligned <= self.bottom_offset.get() {
            return None; // Colidiu com o bottom
        }

        self.top_offset.set(aligned);

        unsafe {
            let ptr = self.buffer.as_ptr().add(aligned);
            Some(NonNull::new_unchecked(ptr))
        }
    }

    pub fn clear_bottom(&self) {
        self.bottom_offset.set(0);
    }

    pub fn clear_top(&self) {
        self.top_offset.set(self.capacity);
    }

    pub fn clear(&self) {
        self.clear_bottom();
        self.clear_top();
    }

    pub fn used(&self) -> usize {
        self.bottom_offset.get() + (self.capacity - self.top_offset.get())
    }

    pub fn available(&self) -> usize {
        self.capacity - self.used()
    }
}

impl Drop for DoubleEndedStack {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.buffer.as_ptr(), self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_creation() {
        let stack = StackAllocator::new(1024);
        assert_eq!(stack.capacity(), 1024);
        assert_eq!(stack.used(), 0);
    }

    #[test]
    fn test_stack_alloc() {
        let stack = StackAllocator::new(1024);

        let ptr1 = stack.alloc(16, 4);
        assert!(ptr1.is_some());

        let ptr2 = stack.alloc(32, 8);
        assert!(ptr2.is_some());

        assert!(stack.used() > 0);
    }

    #[test]
    fn test_stack_mark() {
        let stack = StackAllocator::new(1024);

        stack.alloc(16, 4);
        let mark = stack.mark();

        stack.alloc(32, 4);
        assert!(stack.used() > 16);

        stack.free_to_mark(mark);
        assert!(stack.used() <= 32); // Pode ter headers
    }

    #[test]
    fn test_stack_clear() {
        let stack = StackAllocator::new(1024);

        stack.alloc(100, 4);
        assert!(stack.used() > 0);

        stack.clear();
        assert_eq!(stack.used(), 0);
    }

    #[test]
    fn test_scoped_stack() {
        let stack = StackAllocator::new(1024);

        stack.alloc(50, 4);
        let used_before = stack.used();

        {
            let scoped = ScopedStack::new(&stack);
            scoped.alloc(100, 4);
            assert!(stack.used() > used_before);
        }

        assert_eq!(stack.used(), used_before);
    }

    #[test]
    fn test_double_ended_stack() {
        let stack = DoubleEndedStack::new(1024);

        let bottom = stack.alloc_bottom(64, 8);
        assert!(bottom.is_some());

        let top = stack.alloc_top(64, 8);
        assert!(top.is_some());

        assert!(stack.used() >= 128);
    }
}
