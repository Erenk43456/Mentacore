use crate::debug;
use crate::hardware;
use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

const LAPIC_VIRTUAL_BASE: u64 =
    0xFFFF_A000_0000_0000;

pub unsafe fn initialize(
    allocator: &mut PhysicalFrameAllocator,
) -> hardware::lapic::Lapic {
    let mut lapic =
        match hardware::lapic::Lapic::discover() {
            Some(lapic) => lapic,

            None => {
                debug::write(
                    b"ERROR: LAPIC discovery failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut lapic_mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        match lapic_mapper.map(
            allocator,
            LAPIC_VIRTUAL_BASE,
            lapic.physical_base(),
            memory::paging::PageFlags {
                writable: true,
                cache_disable: true,
                user: false,
            },
        ) {
            Ok(()) => {}

            Err(()) => {
                debug::write(
                    b"ERROR: LAPIC MMIO mapping failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    lapic.set_virtual_base(
        LAPIC_VIRTUAL_BASE
    );

    let lapic_svr =
        unsafe {
            lapic.svr()
        };

    if (lapic_svr
        & hardware::lapic::LAPIC_SVR_ENABLE)
        == 0
    {
        unsafe {
            lapic.set_svr(
                lapic_svr
                    | hardware::lapic::LAPIC_SVR_ENABLE,
            );
        }
    }

    unsafe {
        lapic.set_lvt_timer(
            hardware::lapic::LAPIC_LVT_MASKED
                | crate::interrupts::LAPIC_TIMER_VECTOR
                    as u32,
        );
    }

    debug::write(
        b"LAPIC initialized.\r\n"
    );

    lapic
}