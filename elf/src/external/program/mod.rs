use core::ptr::read_unaligned;

use num_traits::{Unsigned, PrimInt};

use super::{header::ExternalElfHeaderTrait, GenericExternalElf};

#[derive(Debug)]
#[repr(C)]
pub struct ExternalProgramHeader64 {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

impl ExternalProgramHeaderTrait<u64> for ExternalProgramHeader64 {
    fn ph_type(&self) -> u32 {
        unsafe { read_unaligned(&self.p_type) }
    }

    fn flags(&self) -> u32 {
        unsafe { read_unaligned(&self.p_flags) }
    }

    fn offset(&self) -> u64 {
        unsafe { read_unaligned(&self.p_offset) }
    }

    fn vaddr(&self) -> u64 {
        unsafe { read_unaligned(&self.p_vaddr) }
    }

    fn paddr(&self) -> u64 {
        unsafe { read_unaligned(&self.p_paddr) }
    }

    fn filesz(&self) -> u64 {
        unsafe { read_unaligned(&self.p_filesz) }
    }

    fn memsz(&self) -> u64 {
        unsafe { read_unaligned(&self.p_memsz) }
    }

    fn align(&self) -> u64 {
        unsafe { read_unaligned(&self.p_align) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ExternalProgramHeader32 {
    p_type: u32,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    p_flags: u32,
    p_align: u32,
}

impl ExternalProgramHeaderTrait<u32> for ExternalProgramHeader32 {
    fn ph_type(&self) -> u32 {
        unsafe { read_unaligned(&self.p_type) }
    }

    fn flags(&self) -> u32 {
        unsafe { read_unaligned(&self.p_flags) }
    }

    fn offset(&self) -> u32 {
        unsafe { read_unaligned(&self.p_offset) }
    }

    fn vaddr(&self) -> u32 {
        unsafe { read_unaligned(&self.p_vaddr) }
    }

    fn paddr(&self) -> u32 {
        unsafe { read_unaligned(&self.p_paddr) }
    }

    fn filesz(&self) -> u32 {
        unsafe { read_unaligned(&self.p_filesz) }
    }

    fn memsz(&self) -> u32 {
        unsafe { read_unaligned(&self.p_memsz) }
    }

    fn align(&self) -> u32 {
        unsafe { read_unaligned(&self.p_align) }
    }
}


pub trait ExternalProgramHeaderTrait<T: PrimInt + Unsigned> {
    fn ph_type(&self) -> u32;
    fn flags(&self) -> u32;
    fn offset(&self) -> T;
    fn vaddr(&self) -> T;
    fn paddr(&self) -> T;
    fn filesz(&self) -> T;
    fn memsz(&self) -> T;
    fn align(&self) -> T;
}

//---------------------------------------------------------------------------------------------------------

pub struct ExternalProgramHeaderWrapper<'a, T: PrimInt + Unsigned>{
    elf_header: &'a dyn ExternalElfHeaderTrait<T>,
    program_header: &'a dyn ExternalProgramHeaderTrait<T>
}

impl<'a, T: PrimInt + Unsigned> ExternalProgramHeaderWrapper<'a, T>{
    fn fix_endian<I: PrimInt>(&self, val: I) -> I{
        match self.elf_header.endianness(){
            1 => {
                val
            }
            2 => {
                val.to_be()
            }
            _ => {
                panic!();
            }
        }
    }

    pub fn new<R: super::ExternalElfTrait>(index: usize, gen_elf: &'a GenericExternalElf<'a, R>) -> Option<Self> where <R as super::ExternalElfTrait>::ElfHeader: ExternalElfHeaderTrait<T>, <R as super::ExternalElfTrait>::ProgramHeader: ExternalProgramHeaderTrait<T>{
        unsafe{
            match gen_elf.program_headers_raw().get(index){
                Some(program_header) => {
                    Option::Some(Self{
                        elf_header: gen_elf.elf_header_raw(),
                        program_header,
                    })
                },
                None => Option::None,
            }
        }
    }

    pub fn from_components(elf_header: &'a impl ExternalElfHeaderTrait<T>, program_header: &'a impl ExternalProgramHeaderTrait<T>) -> ExternalProgramHeaderWrapper<'a, T>{
        ExternalProgramHeaderWrapper::<T>{
            elf_header,
            program_header,
        }
    }
}

impl<T: PrimInt + Unsigned> ExternalProgramHeaderTrait<T> for ExternalProgramHeaderWrapper<'_, T>{

    fn ph_type(&self) -> u32 {
        self.fix_endian(self.program_header.ph_type())
    }

    fn offset(&self) -> T {
        self.fix_endian(self.program_header.offset())
    }

    fn vaddr(&self) -> T {
        self.fix_endian(self.program_header.vaddr())
    }

    fn paddr(&self) -> T {
        self.fix_endian(self.program_header.paddr())
    }

    fn filesz(&self) -> T {
        self.fix_endian(self.program_header.filesz())
    }

    fn memsz(&self) -> T {
        self.fix_endian(self.program_header.memsz())
    }

    fn align(&self) -> T {
        self.fix_endian(self.program_header.align())
    }

    fn flags(&self) -> u32 {
        self.fix_endian(self.program_header.flags())
    }
}