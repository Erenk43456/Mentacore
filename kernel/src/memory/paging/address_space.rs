use super::{
    mapper::{map_page, unmap_page},
    registers::{current_pml4, load_cr3},
    PageFlags,
    PageTable,
    ADDRESS_MASK,
    ENTRY_COUNT,
    PRESENT,
    USER,
    OWNED,
};
use crate::memory::physical::{
    Frame,
    PhysicalFrameAllocator,
};

const USER_PML4_END: usize = ENTRY_COUNT / 2;

pub struct AddressSpace {
    pml4: *mut PageTable,
    pml4_address: u64,
}

unsafe impl Send for AddressSpace {}

impl AddressSpace {
    pub unsafe fn new(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let (pml4_address, pml4) =
            unsafe { super::table::allocate_pml4(allocator)? };

        Ok(Self {
            pml4,
            pml4_address,
        })
    }

    pub unsafe fn new_user(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let (pml4_address, pml4) =
            unsafe { super::table::allocate_pml4(allocator)? };

        let kernel_pml4 =
            unsafe { current_pml4() };

        unsafe {
            for index in 0..ENTRY_COUNT {
                let entry = (*kernel_pml4).entries[index];

                if entry & PRESENT != 0 && entry & USER == 0 {
                    (*pml4).entries[index] = entry;
                }
            }
        }

        Ok(Self {
            pml4,
            pml4_address,
        })
    }

    pub unsafe fn from_pml4(
        pml4: *mut PageTable,
        pml4_address: u64,
    ) -> Self {
        Self {
            pml4,
            pml4_address,
        }
    }

    pub fn pml4(&self) -> *mut PageTable {
        self.pml4
    }

    pub fn pml4_address(&self) -> u64 {
        self.pml4_address
    }

    pub unsafe fn activate(&self) {
        unsafe {
            load_cr3(self.pml4_address());
        }
    }

    pub unsafe fn mapper(&self) -> Mapper {
        unsafe { Mapper::new(self.pml4) }
    }

    pub unsafe fn map(
        &self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        let mut mapper =
            unsafe { Mapper::new(self.pml4) };

        unsafe {
            mapper.map(
                allocator,
                virtual_address,
                physical_address,
                flags,
            )
        }
    }

    pub unsafe fn map_owned(
        &self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        unsafe {
            self.map(
                allocator,
                virtual_address,
                physical_address,
                flags,
            )?;

            let mut mapper =
                Mapper::new(self.pml4);

            mapper.mark_owned(virtual_address)
        }
    }

    pub unsafe fn unmap(
        &self,
        virtual_address: u64,
    ) -> Result<u64, ()> {
        let mapper =
            unsafe { Mapper::new(self.pml4) };

        unsafe {
            mapper.unmap(virtual_address)
        }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let mut guard =
            crate::memory::physical::frame_allocator()
                .lock();

        let allocator =
            match guard.as_mut() {
                Some(allocator) => allocator,
                None => return,
            };

        unsafe {
            free_user_page_tables(
                self.pml4,
                allocator,
            );
        }

        let pml4_frame =
            match Frame::new(self.pml4_address) {
                Some(frame) => frame,
                None => return,
            };

        let _ =
            allocator.free_frame(pml4_frame);
    }
}

unsafe fn free_user_page_tables(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
) {
    for pml4_index in 0..USER_PML4_END {
        let pml4_entry =
            unsafe {
                (*pml4).entries[pml4_index]
            };

        if pml4_entry & PRESENT == 0
            || pml4_entry & USER == 0
        {
            continue;
        }

        let pdpt_address =
            pml4_entry & ADDRESS_MASK;

        let pdpt =
            match unsafe {
                super::table::physical_table_pointer(
                    pdpt_address,
                )
            } {
                Ok(table) => table,
                Err(_) => continue,
            };

        for pdpt_index in 0..ENTRY_COUNT {
            let pdpt_entry =
                unsafe {
                    (*pdpt).entries[pdpt_index]
                };

            if pdpt_entry & PRESENT == 0 {
                continue;
            }

            let pd_address =
                pdpt_entry & ADDRESS_MASK;

            let pd =
                match unsafe {
                    super::table::physical_table_pointer(
                        pd_address,
                    )
                } {
                    Ok(table) => table,
                    Err(_) => continue,
                };

            for pd_index in 0..ENTRY_COUNT {
                let pd_entry =
                    unsafe {
                        (*pd).entries[pd_index]
                    };

                if pd_entry & PRESENT == 0 {
                    continue;
                }

                let pt_address =
                    pd_entry & ADDRESS_MASK;

                let pt =
                    match unsafe {
                        super::table::physical_table_pointer(
                            pt_address,
                        )
                    } {
                        Ok(table) => table,
                        Err(_) => continue,
                    };

                for pt_index in 0..ENTRY_COUNT {
                    let pte =
                        unsafe {
                            (*pt).entries[pt_index]
                        };

                    if pte & PRESENT == 0 {
                        continue;
                    }

                    if pte & OWNED != 0 {
                        let physical_address =
                            pte & ADDRESS_MASK;

                        if let Some(frame) =
                            Frame::new(physical_address)
                        {
                            let _ =
                                allocator.free_frame(frame);
                        }
                    }
                }

                let pt_frame =
                    match Frame::new(pt_address) {
                        Some(frame) => frame,
                        None => continue,
                    };

                let _ =
                    allocator.free_frame(pt_frame);
            }

            let pd_frame =
                match Frame::new(pd_address) {
                    Some(frame) => frame,
                    None => continue,
                };

            let _ =
                allocator.free_frame(pd_frame);
        }

        let pdpt_frame =
            match Frame::new(pdpt_address) {
                Some(frame) => frame,
                None => continue,
            };

            let _ =
                allocator.free_frame(pdpt_frame);
    }
}

pub struct Mapper {
    pml4: *mut PageTable,
}

impl Mapper {
    pub unsafe fn new(
        pml4: *mut PageTable,
    ) -> Self {
        Self { pml4 }
    }

    pub unsafe fn map(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        unsafe {
            map_page(
                self.pml4,
                allocator,
                virtual_address,
                physical_address,
                flags,
            )
        }
    }

    pub unsafe fn mark_owned(
        &mut self,
        virtual_address: u64,
    ) -> Result<(), ()> {
        let (pt, indices) = unsafe {
            super::table::find_page_table(
                self.pml4,
                virtual_address,
            )?
        };

        unsafe {
            (*pt).entries[indices.pt] |= OWNED;
        }

        Ok(())
    }

    pub unsafe fn unmap(
        &self,
        virtual_address: u64,
    ) -> Result<u64, ()> {
        unsafe {
            unmap_page(
                self.pml4,
                virtual_address,
            )
        }
    }
}