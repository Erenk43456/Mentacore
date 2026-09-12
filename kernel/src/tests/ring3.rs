use core::arch::asm;
use core::sync::atomic::{
    AtomicBool,
    AtomicU64,
    Ordering,
};

use crate::debug;
use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

const USER_CODE_VADDR: u64 =
    0x0000_0100_0000_0000;

const USER_STACK_VADDR: u64 =
    USER_CODE_VADDR + 0x1000;

const USER_STACK_TOP: u64 =
    USER_STACK_VADDR + memory::paging::PAGE_SIZE;

const FIRST_MARKER: u64 =
    0xC0DE_0000_0000_0001;

const SECOND_MARKER: u64 =
    0xC0DE_0000_0000_0002;

const USER_CODE_SELECTOR: u64 = 0x2B;
const USER_DATA_SELECTOR: u64 = 0x33;
const KERNEL_CODE_SELECTOR: u64 = 0x08;

static TEST_PASSED: AtomicBool =
    AtomicBool::new(false);

#[unsafe(no_mangle)]
pub static RING3_KERNEL_RESUME_RSP: AtomicU64 =
    AtomicU64::new(0);

unsafe extern "C" {
    fn user_privilege_test() -> !;

    fn enter_user_mode(
        user_rip: u64,
        user_rsp: u64,
    );

    fn ring3_kernel_resume();
}

pub fn prepare(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let pml4 = unsafe {
        memory::paging::current_pml4()
    };

    let kernel_user_test_address =
        user_privilege_test as usize as u64;

    let user_code_frame =
        kernel_user_test_address
            & !(memory::paging::PAGE_SIZE - 1);

    let user_code_offset =
        kernel_user_test_address
            & (memory::paging::PAGE_SIZE - 1);

    if user_code_offset != 0 {
        debug::write(
            b"ERROR: Ring 3 test code is not page-start aligned.\r\n",
        );

        return false;
    }

    let stack_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                debug::write(
                    b"ERROR: Failed to allocate Ring 3 stack frame.\r\n",
                );

                return false;
            }
        };

    let code_result = unsafe {
        memory::paging::map_page(
            pml4,
            allocator,
            USER_CODE_VADDR,
            user_code_frame,
            memory::paging::PageFlags {
                writable: false,
                cache_disable: false,
                user: true,
            },
        )
    };

    if code_result.is_err() {
        debug::write(
            b"ERROR: Failed to map Ring 3 code page.\r\n",
        );

        return false;
    }

    let stack_result = unsafe {
        memory::paging::map_page(
            pml4,
            allocator,
            USER_STACK_VADDR,
            stack_frame.start_address,
            memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
                user: true,
            },
        )
    };

    if stack_result.is_err() {
        debug::write(
            b"ERROR: Failed to map Ring 3 stack page.\r\n",
        );

        return false;
    }

    let mappings = unsafe {
        memory::paging::test_entry(
            pml4,
            USER_CODE_VADDR,
        )
    };

    let code_user = match mappings {
        Some(entries) => entries
            .iter()
            .all(|entry| entry & (1 << 2) != 0),

        None => false,
    };

    if !code_user {
        debug::write(
            b"ERROR: Ring 3 code mapping lacks USER permission.\r\n",
        );

        return false;
    }

    let mappings = unsafe {
        memory::paging::test_entry(
            pml4,
            USER_STACK_VADDR,
        )
    };

    let stack_user = match mappings {
        Some(entries) => entries
            .iter()
            .all(|entry| entry & (1 << 2) != 0),

        None => false,
    };

    if !stack_user {
        debug::write(
            b"ERROR: Ring 3 stack mapping lacks USER permission.\r\n",
        );

        return false;
    }

    true
}

pub fn run() -> bool {
    TEST_PASSED.store(
        false,
        Ordering::Relaxed,
    );

    let user_rip =
        USER_CODE_VADDR;

    let user_rsp =
        USER_STACK_TOP;

    unsafe {
        enter_user_mode(
            user_rip,
            user_rsp,
        );
    }

    TEST_PASSED.load(
        Ordering::Relaxed,
    )
}

#[unsafe(no_mangle)]
extern "C" fn ring3_interrupt_dispatch(
    saved_registers: *const u64,
    cpu_frame: *mut u64,
    current_rsp: u64,
) {
    let saved_rax = unsafe {
        *saved_registers.add(14)
    };

    let kernel_resume_rsp =
        RING3_KERNEL_RESUME_RSP.load(
            Ordering::Relaxed,
        );

    let user_rip = unsafe {
        *cpu_frame.add(0)
    };

    let user_cs = unsafe {
        *cpu_frame.add(1)
    };

    let user_rflags = unsafe {
        *cpu_frame.add(2)
    };

    let user_rsp = unsafe {
        *cpu_frame.add(3)
    };

    let user_ss = unsafe {
        *cpu_frame.add(4)
    };

    let (
        kernel_stack_start,
        kernel_stack_end,
    ) = crate::cpu::kernel_stack_range();

    let kernel_stack_ok =
        current_rsp >= kernel_stack_start
            && current_rsp <= kernel_stack_end;

    let user_frame_ok =
        user_cs == USER_CODE_SELECTOR
            && user_ss == USER_DATA_SELECTOR
            && user_rsp == USER_STACK_TOP
            && user_rflags & 0x2 != 0
            && user_rip >= USER_CODE_VADDR
            && user_rip < USER_CODE_VADDR
                + memory::paging::PAGE_SIZE;

    if saved_rax == FIRST_MARKER {
        let ok =
            kernel_stack_ok
                && user_frame_ok;

        if !ok {
            debug::write(
                b"Ring 3 first interrupt validation FAILED.\r\n",
            );

            unsafe {
                *cpu_frame.add(0) =
                    ring3_kernel_resume as *const () as usize as u64;
                *cpu_frame.add(1) =
                    KERNEL_CODE_SELECTOR;
                *cpu_frame.add(3) =
                    kernel_resume_rsp;
                *cpu_frame.add(4) =
                    0x10;
            }

            TEST_PASSED.store(false, Ordering::Relaxed);
            return;
        }

        debug::write(
            b"Ring 3 -> Ring 0 -> Ring 3 OK.\r\n",
        );

        return;
    }

    if saved_rax == SECOND_MARKER {
        let ok =
            kernel_stack_ok
                && user_frame_ok
                && kernel_resume_rsp != 0;

        if !ok {
            debug::write(
                b"Ring 3 second interrupt validation FAILED.\r\n",
            );
        } else {
            debug::write(
                b"Ring 3 privilege transition complete.\r\n",
            );
        }

        TEST_PASSED.store(ok, Ordering::Relaxed);

        unsafe {
            *cpu_frame.add(0) =
                ring3_kernel_resume as *const () as usize as u64;
            *cpu_frame.add(1) =
                KERNEL_CODE_SELECTOR;
            *cpu_frame.add(3) =
                kernel_resume_rsp;
            *cpu_frame.add(4) =
                0x10;
        }

        return;
    }

    debug::write(
        b"ERROR: Unexpected Ring 3 interrupt marker.\r\n",
    );

    TEST_PASSED.store(
        false,
        Ordering::Relaxed,
    );

    debug::write(
        b"ERROR: Unexpected Ring 3 interrupt marker.\r\n",
    );

    TEST_PASSED.store(
        false,
        Ordering::Relaxed,
    );

    unsafe {
        *cpu_frame.add(0) =
            ring3_kernel_resume as usize as u64;

        *cpu_frame.add(1) =
            KERNEL_CODE_SELECTOR;

        *cpu_frame.add(3) =
            kernel_resume_rsp;

        *cpu_frame.add(4) =
            0x10;
    }
}