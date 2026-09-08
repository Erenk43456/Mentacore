use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU64, Ordering};

pub const HEAP_START: u64 = 0xFFFF_8000_0000_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024 * 1024;

struct BumpAllocator {
    next: AtomicU64,
}

impl BumpAllocator {
    const fn new() -> Self {
        Self {
            next: AtomicU64::new(HEAP_START),
        }
    }

    unsafe fn init(&self) {
        self.next.store(HEAP_START, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        let size = layout.size() as u64;
        let align = layout.align() as u64;

        loop {
            let current =
                self.next.load(Ordering::Relaxed);

            let aligned = match current.checked_add(align - 1) {
                Some(value) => value & !(align - 1),
                None => return null_mut(),
            };

            let new_next =
                match aligned.checked_add(size) {
                    Some(value) => value,
                    None => return null_mut(),
                };

            if new_next > HEAP_START + HEAP_SIZE {
                return null_mut();
            }

            if self
                .next
                .compare_exchange(
                    current,
                    new_next,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return aligned as *mut u8;
            }
        }
    }

    unsafe fn dealloc(
        &self,
        _ptr: *mut u8,
        _layout: Layout,
    ) {
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator =
    BumpAllocator::new();

pub unsafe fn init() {
    unsafe {
        ALLOCATOR.init();
    }
}