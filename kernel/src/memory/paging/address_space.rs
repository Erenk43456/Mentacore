use super::{
    mapper::{map_page, unmap_page},
    registers::{current_pml4, load_cr3},
    PageFlags,
    PageTable,
    ADDRESS_MASK,
    ENTRY_COUNT,
    PRESENT,
    USER,
};
use crate::memory::physical::PhysicalFrameAllocator;

pub struct AddressSpace {
    pml4: *mut PageTable,
}

unsafe impl Send for AddressSpace {}

impl AddressSpace {
    pub unsafe fn new(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let (_, pml4) =
            unsafe { super::table::allocate_pml4(allocator)? };

        Ok(Self { pml4 })
    }

    pub unsafe fn new_user(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let (_, pml4) =
            unsafe { super::table::allocate_pml4(allocator)? };

        let kernel_pml4 =
            unsafe { current_pml4() };

        unsafe {
            for index in 0..ENTRY_COUNT {
                let entry =
                    (*kernel_pml4).entries[index];

                if entry & PRESENT != 0
                    && entry & USER == 0
                {
                    (*pml4).entries[index] =
                        entry;
                }
            }
        }

        Ok(Self { pml4 })
    }

    pub unsafe fn from_pml4(
        pml4: *mut PageTable,
    ) -> Self {
        Self { pml4 }
    }

    pub fn pml4(&self) -> *mut PageTable {
        self.pml4
    }

    pub fn pml4_address(&self) -> u64 {
        self.pml4 as u64 & ADDRESS_MASK
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