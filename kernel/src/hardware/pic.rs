use core::arch::asm;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;

const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;

const ICW4_8086: u8 = 0x01;

pub const MASTER_OFFSET: u8 = 32;
pub const SLAVE_OFFSET: u8 = 40;

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags)
        );
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nostack, preserves_flags)
        );
    }

    value
}

fn io_wait() {
    unsafe {
        outb(0x80, 0);
    }
}

pub unsafe fn remap() {
    let master_mask = unsafe {
        inb(PIC1_DATA)
    };

    let slave_mask = unsafe {
        inb(PIC2_DATA)
    };

    unsafe {
        outb(
            PIC1_COMMAND,
            ICW1_INIT | ICW1_ICW4,
        );

        io_wait();

        outb(
            PIC2_COMMAND,
            ICW1_INIT | ICW1_ICW4,
        );

        io_wait();

        outb(
            PIC1_DATA,
            MASTER_OFFSET,
        );

        io_wait();

        outb(
            PIC2_DATA,
            SLAVE_OFFSET,
        );

        io_wait();

        outb(
            PIC1_DATA,
            0x04,
        );

        io_wait();

        outb(
            PIC2_DATA,
            0x02,
        );

        io_wait();

        outb(
            PIC1_DATA,
            ICW4_8086,
        );

        io_wait();

        outb(
            PIC2_DATA,
            ICW4_8086,
        );

        io_wait();

        outb(
            PIC1_DATA,
            master_mask,
        );

        outb(
            PIC2_DATA,
            slave_mask,
        );
    }
}

pub unsafe fn send_eoi(irq: u8) {
    if irq >= 8 {
        unsafe {
            outb(
                PIC2_COMMAND,
                PIC_EOI,
            );
        }
    }

    unsafe {
        outb(
            PIC1_COMMAND,
            PIC_EOI,
        );
    }
}

pub unsafe fn enable_irq(irq: u8) {
    if irq >= 16 {
        return;
    }

    let port = if irq < 8 {
        PIC1_DATA
    } else {
        PIC2_DATA
    };

    let bit = 1u8 << (irq % 8);

    unsafe {
        let mask = inb(port);
        outb(port, mask & !bit);
    }
}