use super::{
    mapper::{map_page, unmap_page},
    PageFlags,
    PageTable,
};
use crate::memory::physical::PhysicalFrameAllocator;

pub struct AddressSpace {
    pml4: *mut PageTable,
}

impl AddressSpace {
    pub unsafe fn new(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let (_, pml4) =
            unsafe { super::table::allocate_pml4(allocator)? };

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

        unsafe { mapper.unmap(virtual_address) }
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