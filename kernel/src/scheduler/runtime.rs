use crate::memory::physical::PhysicalFrameAllocator;
use crate::memory::paging::{
    PageFlags,
    PAGE_SIZE,
};
use crate::process::{
    ProcessId,
    ProcessManager,
};
use crate::thread::{
    KernelInterruptContext,
    ThreadManager,
    ThreadState,
    ThreadId,
};
use crate::sync::Spinlock;

use super::Scheduler;

pub const IDLE_THREAD_ID: ThreadId = 0;
pub const KERNEL_PROCESS_ID: ProcessId = 0;

extern "C" fn idle_thread() -> ! {
    loop {
        crate::cpu::halt();
    }
}

pub struct SchedulerRuntime {
    scheduler: Scheduler,
    manager: ThreadManager,
    processes: ProcessManager,
}

pub static SCHEDULER_RUNTIME:
    Spinlock<Option<SchedulerRuntime>> =
    Spinlock::new(None);

impl SchedulerRuntime {
    pub fn initialize(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<(), ()> {
        let runtime =
            SchedulerRuntime::new(allocator)?;

        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        if guard.is_some() {
            return Err(());
        }

        *guard = Some(runtime);

        Ok(())
    }

    pub fn new(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let mut manager = ThreadManager::new();
        let mut scheduler = Scheduler::new();
        let processes = ProcessManager::new();

        manager.create(
            IDLE_THREAD_ID,
            KERNEL_PROCESS_ID,
            allocator,
            idle_thread,
        )?;

        scheduler.start(&mut manager);

        Ok(Self {
            scheduler,
            manager,
            processes,
        })
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn thread_manager(&self) -> &ThreadManager {
        &self.manager
    }

    pub fn thread_manager_mut(
        &mut self,
    ) -> &mut ThreadManager {
        &mut self.manager
    }

    pub fn current(&self) -> Option<ThreadId> {
        self.scheduler.current()
    }

    pub fn current_thread_id() -> Option<ThreadId> {
        let guard = SCHEDULER_RUNTIME.lock_irqsave();
        guard.as_ref().and_then(|runtime| runtime.current())
    }

    pub fn current_state(&self) -> Option<ThreadState> {
        self.current()
            .and_then(|thread_id|
                self.manager
                    .get(thread_id)
                    .map(|thread| thread.state())
            )
    }

    pub fn create_thread(
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: crate::thread::ThreadEntry,
    ) -> Result<(), ()> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut().ok_or(())?;

        runtime.manager.create(
            thread_id,
            process_id,
            allocator,
            entry,
        )?;

        if runtime.scheduler
            .add_thread(
                &runtime.manager,
                thread_id,
            )
            .is_err()
        {
            let _ =
                runtime.manager.remove(thread_id);

            return Err(());
        }

        Ok(())
    }

    pub fn prepare_thread(
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: crate::thread::ThreadEntry,
    ) -> Result<(), ()> {
        let mut guard = SCHEDULER_RUNTIME.lock_irqsave();
        let runtime = guard.as_mut().ok_or(())?;

        runtime
            .manager
            .create(
                thread_id,
                process_id,
                allocator,
                entry,
            )?;

        Ok(())
    }

    pub fn activate_thread(
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let mut guard = SCHEDULER_RUNTIME.lock_irqsave();
        let runtime = guard.as_mut().ok_or(())?;

        runtime
            .scheduler
            .add_thread(
                &runtime.manager,
                thread_id,
            )?;

        Ok(())
    }

    pub fn start_thread(
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut().ok_or(())?;

        runtime.scheduler.start_thread(
            &mut runtime.manager,
            thread_id,
        )
    }

    pub fn preempt(
        current_rsp: u64,
    ) -> Option<(u64, u64)> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut()?;

        let next_thread_id =
            runtime.scheduler.peek_next()?;

        let next_thread =
            runtime.manager.get(next_thread_id)?;

        let process_id =
            next_thread.process_id();

        let next_stack_base =
            next_thread.kernel_stack().base();

        let next_stack_top =
            next_thread.kernel_stack().top();

        let next_pml4 =
            if process_id != KERNEL_PROCESS_ID {
                let process =
                    runtime.processes.get(process_id)?;

                process
                    .address_space()
                    .pml4_address()
            } else {
                crate::memory::paging::kernel_pml4_address()
            };

        let next_rsp =
            runtime.scheduler.preempt(
                &mut runtime.manager,
                current_rsp,
            )?;

        crate::debug::write(b"SCHED SWITCH\r\n");

        crate::debug::write(b"current_rsp: ");
        crate::debug::write_hex(current_rsp);
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_rsp: ");
        crate::debug::write_hex(next_rsp);
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_stack_base: ");
        crate::debug::write_hex(next_stack_base);
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_stack_top: ");
        crate::debug::write_hex(next_stack_top);
        crate::debug::write(b"\r\n");

        crate::debug::write(b"iret_rsp: ");
        crate::debug::write_hex(
            next_rsp
                + core::mem::size_of::<KernelInterruptContext>() as u64
        );
        crate::debug::write(b"\r\n");
        crate::debug::write(b"push_rsp: ");
        crate::debug::write_hex(
            next_rsp
                + core::mem::size_of::<KernelInterruptContext>() as u64
                - 8
        );
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_pml4: ");
        crate::debug::write_hex(next_pml4);
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_frame_rip: ");
        crate::debug::write_hex(
            unsafe {
                *((next_rsp + 15 * 8) as *const u64)
            }
        );
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_frame_cs: ");
        crate::debug::write_hex(
            unsafe {
                *((next_rsp + 16 * 8) as *const u64)
            }
        );
        crate::debug::write(b"\r\n");

        crate::debug::write(b"next_frame_rflags: ");
        crate::debug::write_hex(
            unsafe {
                *((next_rsp + 17 * 8) as *const u64)
            }
        );
        crate::debug::write(b"\r\n");

        drop(guard);

        Some((next_rsp, next_pml4))
    }

    pub fn create_userspace_process(
        process_id: ProcessId,
        thread_id: ThreadId,
        loaded_elf: crate::memory::LoadedElf,
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<(), ()> {
        let mut loaded_elf = loaded_elf;

        let user_stack_top =
            unsafe {
                loaded_elf.map_user_stack(allocator)?
            };

        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut().ok_or(())?;

        runtime.processes.create_userspace(
            process_id,
            loaded_elf,
            user_stack_top,
        )?;

        let process =
            runtime.processes
                .get(process_id)
                .ok_or(())?;

        runtime.manager.create_user(
            thread_id,
            process_id,
            allocator,
            process.entry(),
            process.user_stack_top(),
        )?;

        runtime.scheduler.add_thread(
            &runtime.manager,
            thread_id,
        )?;

        Ok(())
    }

    pub fn launch_userspace(
        process_id: ProcessId,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut().ok_or(())?;

        runtime.scheduler
            .start_thread(
                &mut runtime.manager,
                thread_id,
            )?;

        let next_pml4 =
            runtime.processes
                .get(process_id)
                .ok_or(())?
                .address_space()
                .pml4_address();

        let thread =
            runtime.manager
                .get(thread_id)
                .ok_or(())?;

        let next_rsp =
            thread.interrupt_rsp();

        drop(guard);

        let mut current_rsp = 0u64;

        unsafe {
            crate::thread::interrupt_context_switch_to_address_space(
                &raw mut current_rsp,
                next_rsp as *const u64,
                next_pml4,
            );
        }

        let mut current_rsp = 0u64;

        unsafe {
            crate::thread::interrupt_context_switch_to_address_space(
                &raw mut current_rsp,
                next_rsp as *const u64,
                next_pml4,
            );
        }
    }

    pub fn terminate_current_user_thread(
        current_rsp: u64,
    ) -> Option<u64> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut()?;

        let current =
            runtime.scheduler.current()?;

        let current_thread =
            runtime.manager.get(current)?;

        if current_thread.process_id()
            == KERNEL_PROCESS_ID
        {
            return None;
        }

        let next_process_id;

        let next_rsp;

        runtime
            .scheduler
            .remove(
                &mut runtime.manager,
                current,
            )
            .ok()?;

        runtime.manager.remove(current).ok()?;

        let next =
            runtime.scheduler.current()?;

        let next_thread =
            runtime.manager.get(next)?;

        next_process_id =
            next_thread.process_id();

        next_rsp =
            next_thread.interrupt_rsp();

        let next_pml4 =
            if next_process_id != KERNEL_PROCESS_ID {
                let process =
                    runtime.processes.get(
                        next_process_id,
                    )?;

                process
                    .address_space()
                    .pml4_address()
            } else {
                crate::memory::paging::kernel_pml4_address()
            };

        drop(guard);

        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) next_pml4,
                options(nostack, preserves_flags)
            );
        }

        crate::debug::write(b"POST-CR3 STACK TEST\r\n");

        unsafe {
            let stack_ptr =
                0x0000_0000_02beff0u64 as *mut u64;

            core::ptr::write_volatile(
                stack_ptr,
                0x1122334455667788,
            );

            let value =
                core::ptr::read_volatile(stack_ptr);

            crate::debug::write(b"stack_write_value: ");
            crate::debug::write_hex(value);
            crate::debug::write(b"\r\n");
        }

        let mut saved_rsp = current_rsp;

        unsafe {
            crate::thread::interrupt_context_switch(
                &raw mut saved_rsp,
                next_rsp as *const u64,
            );
        }
    }
}