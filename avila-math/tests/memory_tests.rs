use avila_math::memory::*;

#[test]
fn test_arena_basic_usage() {
    let arena = Arena::new(1024);

    // Aloca alguns bytes
    let ptr1 = arena.alloc(64, 8).expect("Failed to allocate");
    assert!(arena.used() >= 64);

    let ptr2 = arena.alloc(128, 16).expect("Failed to allocate");
    assert!(arena.used() >= 64 + 128);

    // Reset libera tudo
    arena.reset();
    assert_eq!(arena.used(), 0);
}

#[test]
fn test_arena_checkpoint_restore() {
    let arena = Arena::new(2048);

    arena.alloc(100, 4);
    let checkpoint = arena.checkpoint();
    let used_at_checkpoint = arena.used();

    arena.alloc(200, 4);
    arena.alloc(300, 4);
    assert!(arena.used() > used_at_checkpoint);

    arena.restore(checkpoint);
    assert_eq!(arena.used(), used_at_checkpoint);
}

#[test]
fn test_scoped_arena() {
    let arena = Arena::new(2048);

    arena.alloc(100, 4);
    let used_before_scope = arena.used();

    {
        let _scoped = ScopedArena::new(&arena);
        arena.alloc(200, 4);
        assert!(arena.used() > used_before_scope);
    } // Escopo termina, restaura automaticamente

    assert_eq!(arena.used(), used_before_scope);
}

#[test]
fn test_pool_alloc_free() {
    let pool = Pool::for_type::<u64>(32);

    let mut ptrs = Vec::new();

    // Aloca vários objetos
    for _ in 0..10 {
        if let Some(ptr) = pool.alloc_type::<u64>() {
            ptrs.push(ptr);
        }
    }

    let stats = pool.stats();
    assert_eq!(stats.chunks_in_use, 10);

    // Libera todos
    unsafe {
        for ptr in ptrs {
            pool.free_type(ptr);
        }
    }

    let stats = pool.stats();
    assert_eq!(stats.chunks_in_use, 0);
}

#[test]
fn test_typed_pool() {
    #[derive(Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        data: f32,
    }

    let pool = TypedPool::<TestStruct>::new(16);

    let ptr = pool.alloc().expect("Failed to allocate");

    // Escreve dados
    unsafe {
        ptr.as_ptr().write(TestStruct {
            value: 42,
            data: 3.14,
        });
    }

    // Lê dados
    unsafe {
        let obj = ptr.as_ref();
        assert_eq!(obj.value, 42);
        assert_eq!(obj.data, 3.14);
    }

    unsafe {
        pool.free(ptr);
    }
}

#[test]
fn test_pool_box() {
    let pool = Pool::for_type::<i32>(16);

    {
        let mut boxed = PoolBox::new(&pool, 100).expect("Failed to allocate");
        assert_eq!(*boxed, 100);

        *boxed = 200;
        assert_eq!(*boxed, 200);
    } // PoolBox libera automaticamente

    let stats = pool.stats();
    assert_eq!(stats.chunks_in_use, 0);
}

#[test]
fn test_stack_allocator() {
    let stack = StackAllocator::new(2048);

    let ptr1 = stack.alloc(64, 8).expect("Failed to allocate");
    let ptr2 = stack.alloc(128, 16).expect("Failed to allocate");

    assert!(stack.used() >= 64 + 128);

    // Libera em ordem LIFO
    unsafe {
        stack.free(ptr2);
        stack.free(ptr1);
    }
}

#[test]
fn test_stack_mark() {
    let stack = StackAllocator::new(2048);

    stack.alloc(100, 4);
    let mark = stack.mark();
    let used_at_mark = stack.used();

    stack.alloc(200, 4);
    stack.alloc(300, 4);

    stack.free_to_mark(mark);
    assert!(stack.used() <= used_at_mark + 32); // Pequena margem para headers
}

#[test]
fn test_scoped_stack() {
    let stack = StackAllocator::new(2048);

    stack.alloc(100, 4);
    let used_before = stack.used();

    {
        let _scoped = ScopedStack::new(&stack);
        stack.alloc(200, 4);
    }

    assert_eq!(stack.used(), used_before);
}

#[test]
fn test_double_ended_stack() {
    let stack = DoubleEndedStack::new(2048);

    let bottom1 = stack
        .alloc_bottom(64, 8)
        .expect("Failed to allocate bottom");
    let top1 = stack.alloc_top(64, 8).expect("Failed to allocate top");

    assert!(stack.used() >= 128);
    assert!(stack.available() <= 2048 - 128);

    stack.clear();
    assert_eq!(stack.used(), 0);
}

#[test]
fn test_memory_stats() {
    let stats = MemoryStats::new();

    stats.record_allocation(1024);
    stats.record_allocation(2048);

    assert_eq!(stats.total_allocations(), 2);
    assert_eq!(stats.current_memory_usage(), 3072);
    assert_eq!(stats.peak_memory_usage(), 3072);

    stats.record_deallocation(1024);
    assert_eq!(stats.current_memory_usage(), 2048);
    assert_eq!(stats.peak_memory_usage(), 3072); // Peak não muda
}

#[test]
fn test_memory_manager() {
    let mut manager = MemoryManager::new();

    manager.register_allocator(
        "arena_main",
        AllocatorInfo {
            allocator_type: AllocatorType::Arena,
            total_capacity: 1024 * 1024,
            used: 512 * 1024,
            available: 512 * 1024,
            allocation_count: 100,
            deallocation_count: 0,
        },
    );

    manager.register_allocator(
        "pool_entities",
        AllocatorInfo {
            allocator_type: AllocatorType::Pool,
            total_capacity: 256 * 1024,
            used: 128 * 1024,
            available: 128 * 1024,
            allocation_count: 500,
            deallocation_count: 250,
        },
    );

    let report = manager.report();
    assert_eq!(report.allocator_count, 2);
    assert_eq!(report.total_allocated, 1024 * 1024 + 256 * 1024);
    assert!(report.utilization() > 0.0);
}

#[test]
fn test_allocator_info() {
    let info = AllocatorInfo {
        allocator_type: AllocatorType::Pool,
        total_capacity: 1000,
        used: 600,
        available: 400,
        allocation_count: 100,
        deallocation_count: 40,
    };

    assert!((info.utilization() - 60.0).abs() < 0.01);
    assert_eq!(info.active_allocations(), 60);
}

#[test]
fn test_memory_profiler() {
    use std::time::Duration;

    let mut profiler = MemoryProfiler::new(Duration::from_millis(10));
    let stats = MemoryStats::new();

    stats.record_allocation(1000);
    profiler.sample(&stats);

    stats.record_allocation(2000);
    std::thread::sleep(Duration::from_millis(15));
    profiler.sample(&stats);

    assert!(profiler.samples().len() >= 1);
    assert!(profiler.average_usage().unwrap() > 0);
}

#[test]
fn test_integration_scenario() {
    // Cenário real: sistema de entidades com diferentes allocators

    // Arena para data temporária de frame
    let frame_arena = Arena::new(1024 * 1024); // 1MB

    // Pool para entidades
    let entity_pool = Pool::for_type::<Entity>(256);

    // Stack para processamento hierárquico
    let process_stack = StackAllocator::new(512 * 1024);

    // Manager para tracking
    let mut manager = MemoryManager::new();

    // Simula alocações
    {
        let _temp_data = frame_arena.alloc(1024, 16);

        let entity = entity_pool.alloc_type::<Entity>();
        assert!(entity.is_some());

        let _process_data = process_stack.alloc(512, 8);
    }

    // Registra estatísticas
    manager.register_allocator(
        "frame_arena",
        AllocatorInfo {
            allocator_type: AllocatorType::Arena,
            total_capacity: frame_arena.capacity(),
            used: frame_arena.used(),
            available: frame_arena.available(),
            allocation_count: 1,
            deallocation_count: 0,
        },
    );

    let report = manager.report();
    assert!(report.total_allocated > 0);
}

#[derive(Debug)]
struct Entity {
    id: u64,
    position: [f32; 3],
    velocity: [f32; 3],
}

#[test]
fn test_format_bytes() {
    assert!(format::bytes(512).contains("bytes"));
    assert!(format::bytes(2048).contains("KB"));
    assert!(format::bytes(2 * 1024 * 1024).contains("MB"));
    assert!(format::bytes(2 * 1024 * 1024 * 1024).contains("GB"));
}
