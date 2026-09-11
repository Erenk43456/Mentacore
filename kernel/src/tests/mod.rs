pub mod framework;
mod physical;
mod paging;
mod heap;
mod sync;
mod tsc;
pub mod interrupts;
mod cpu;
mod process;
mod thread;
mod scheduler;

use crate::debug;
use crate::memory::physical::PhysicalFrameAllocator;

pub struct KernelTestRunner {
    runner: framework::TestRunner,
}

impl KernelTestRunner {
    pub const fn new() -> Self {
        Self {
            runner: framework::TestRunner::new(),
        }
    }

    pub fn run_physical(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
    ) {
        physical::run(
            &mut self.runner,
            allocator,
        );
    }

    pub fn run_paging(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
    ) {
        paging::run(
            &mut self.runner,
            allocator,
        );
    }

    pub fn run_heap(&mut self) {
        heap::run(&mut self.runner);
    }

    pub fn run_sync(&mut self) {
        sync::run(&mut self.runner);
    }

    pub fn run_tsc(&mut self) {
        tsc::run(&mut self.runner);
    }

    pub fn run_cpu(&mut self) {
        cpu::run(&mut self.runner);
    }

    pub fn run_process(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
    ) {
        process::run(
            &mut self.runner,
            allocator,
        );
    }

    pub fn run_thread(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
    ) {
        thread::run(
            &mut self.runner,
            allocator,
        );
    }

    pub fn run_scheduler(&mut self) {
        scheduler::run(&mut self.runner);
    }

    pub fn run_interrupts(
        &mut self,
        lapic: &crate::hardware::lapic::Lapic,
    ) {
        interrupts::run(
            &mut self.runner,
            lapic,
        );
    }

    pub fn finish(&self) {
        self.runner.finish();
        debug::write(b"\r\n");
    }
}

pub fn write_header() {
    framework::write_header();
}