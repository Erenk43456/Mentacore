use crate::debug;
use crate::hardware;
use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

const LAPIC_VIRTUAL_BASE: u64 =
    0xFFFF_A000_0000_0000;

pub unsafe fn initialize(
    allocator: &mut PhysicalFrameAllocator,
) -> hardware::lapic::Lapic {
    debug::write(b"Initializing LAPIC...\r\n");

    let mut lapic =
        match hardware::lapic::Lapic::discover() {
            Some(lapic) => lapic,

            None => {
                debug::write(
                    b"LAPIC DISCOVERY FAILED\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    debug::write(b"LAPIC MSR: ");
    debug::write_hex(lapic.msr_value());
    debug::write(b"\r\n");

    debug::write(b"LAPIC physical base: ");
    debug::write_hex(lapic.physical_base());
    debug::write(b"\r\n");

    debug::write(b"LAPIC enabled: YES\r\n");

    debug::write(b"Mapping LAPIC MMIO...\r\n");

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

    lapic.set_virtual_base(LAPIC_VIRTUAL_BASE);

    debug::write(b"LAPIC virtual base: ");
    debug::write_hex(lapic.virtual_base());
    debug::write(b"\r\n");

    debug::write(b"LAPIC MMIO mapping OK\r\n");

    let lapic_id =
        unsafe {
            lapic.id()
        };

    debug::write(b"LAPIC ID register: ");
    debug::write_hex(lapic_id as u64);
    debug::write(b"\r\n");

    let lapic_version =
        unsafe {
            lapic.version()
        };

    debug::write(b"LAPIC VERSION register: ");
    debug::write_hex(lapic_version as u64);
    debug::write(b"\r\n");

    let mut lapic_svr =
        unsafe {
            lapic.svr()
        };

    debug::write(b"LAPIC SVR register: ");
    debug::write_hex(lapic_svr as u64);
    debug::write(b"\r\n");

    if (lapic_svr & hardware::lapic::LAPIC_SVR_ENABLE) == 0 {
        debug::write(
            b"Enabling LAPIC software...\r\n"
        );

        unsafe {
            lapic.set_svr(
                lapic_svr
                    | hardware::lapic::LAPIC_SVR_ENABLE,
            );
        }

        lapic_svr =
            unsafe {
                lapic.svr()
            };

        debug::write(
            b"LAPIC SVR after enable: "
        );
        debug::write_hex(lapic_svr as u64);
        debug::write(b"\r\n");
    }

    let lapic_lvt_timer =
        unsafe {
            lapic.lvt_timer()
        };

    debug::write(
        b"LAPIC LVT TIMER register: "
    );
    debug::write_hex(lapic_lvt_timer as u64);
    debug::write(b"\r\n");

    let lapic_lvt_error =
        unsafe {
            lapic.lvt_error()
        };

    debug::write(
        b"LAPIC LVT ERROR register: "
    );
    debug::write_hex(lapic_lvt_error as u64);
    debug::write(b"\r\n");

    debug::write(
        b"LAPIC SOFTWARE ENABLED\r\n"
    );
    debug::write(
        b"LAPIC REGISTER ACCESS OK\r\n"
    );

    let lapic_timer_divide =
        unsafe {
            lapic.timer_divide()
        };

    debug::write(
        b"LAPIC TIMER DIVIDE register: "
    );
    debug::write_hex(lapic_timer_divide as u64);
    debug::write(b"\r\n");

    let lapic_timer_initial =
        unsafe {
            lapic.timer_initial_count()
        };

    debug::write(
        b"LAPIC TIMER INITIAL COUNT register: "
    );
    debug::write_hex(lapic_timer_initial as u64);
    debug::write(b"\r\n");

    let lapic_timer_current =
        unsafe {
            lapic.timer_current_count()
        };

    debug::write(
        b"LAPIC TIMER CURRENT COUNT register: "
    );
    debug::write_hex(lapic_timer_current as u64);
    debug::write(b"\r\n");

    let lapic_timer_lvt =
        unsafe {
            lapic.lvt_timer()
        };

    debug::write(
        b"LAPIC TIMER LVT verification: "
    );
    debug::write_hex(lapic_timer_lvt as u64);
    debug::write(b"\r\n");

    if (lapic_timer_lvt
        & hardware::lapic::LAPIC_LVT_MASKED)
        != 0
    {
        debug::write(
            b"LAPIC TIMER MASKED OK\r\n"
        );
    } else {
        debug::write(
            b"LAPIC TIMER MASKED UNEXPECTED\r\n"
        );
    }

    lapic
}