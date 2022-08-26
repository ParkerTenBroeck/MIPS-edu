use core::ptr::read_unaligned;
use num_traits::{PrimInt, Unsigned};

use super::GenericExternalElf;

#[repr(C)]
#[derive(Debug)]
pub struct ExternalElfHeader<T: PrimInt + Unsigned> {
    magic: [u8; 4],
    class: u8,
    endianness: u8,
    header_version: u8,
    abi: u8,
    abi_version: u8,
    unused: [u8; 7],
    elftype: u16,
    machine: u16,
    elf_version: u32,
    entry: T,
    phoff: T,
    shoff: T,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

impl<T: PrimInt + Unsigned> ExternalElfHeader<T> {
    pub fn read_int_type<N: PrimInt>(&self, val: *const N) -> N {
        unsafe { read_unaligned(val) }
    }
}

impl<T: PrimInt + Unsigned> ExternalElfHeaderTrait<T> for ExternalElfHeader<T> {
    fn class(&self) -> u8 {
        self.class
    }

    fn endianness(&self) -> u8 {
        self.endianness
    }

    fn header_version(&self) -> u8 {
        self.header_version
    }

    fn abi(&self) -> u8 {
        self.abi
    }

    fn abi_version(&self) -> u8 {
        self.abi_version
    }

    fn elftype(&self) -> u16 {
        self.read_int_type(&self.elftype)
    }

    fn machine(&self) -> u16 {
        self.read_int_type(&self.machine)
    }

    fn elf_version(&self) -> u32 {
        self.read_int_type(&self.elf_version)
    }

    fn entry_point(&self) -> T {
        self.read_int_type(&self.entry)
    }

    fn program_header_offset(&self) -> T {
        self.read_int_type(&self.phoff)
    }

    fn section_header_offset(&self) -> T {
        self.read_int_type(&self.shoff)
    }

    fn flags(&self) -> u32 {
        self.read_int_type(&self.flags)
    }

    fn elf_header_size(&self) -> u16 {
        self.read_int_type(&self.ehsize)
    }

    fn program_header_entry_size(&self) -> u16 {
        self.read_int_type(&self.phentsize)
    }

    fn program_header_entry_num(&self) -> u16 {
        self.read_int_type(&self.phnum)
    }

    fn section_header_entry_size(&self) -> u16 {
        self.read_int_type(&self.shentsize)
    }

    fn section_header_entry_num(&self) -> u16 {
        self.read_int_type(&self.shnum)
    }

    fn shstr_index(&self) -> u16 {
        self.read_int_type(&self.shstrndx)
    }
}

pub trait ExternalElfHeaderTrait<T: PrimInt + Unsigned> {
    fn class(&self) -> u8;
    fn endianness(&self) -> u8;
    fn header_version(&self) -> u8;
    fn abi(&self) -> u8;
    fn abi_version(&self) -> u8;
    fn elftype(&self) -> u16;
    fn machine(&self) -> u16;
    fn elf_version(&self) -> u32;
    fn entry_point(&self) -> T;
    fn program_header_offset(&self) -> T;
    fn section_header_offset(&self) -> T;
    fn flags(&self) -> u32;
    fn elf_header_size(&self) -> u16;
    fn program_header_entry_size(&self) -> u16;
    fn program_header_entry_num(&self) -> u16;
    fn section_header_entry_size(&self) -> u16;
    fn section_header_entry_num(&self) -> u16;
    fn shstr_index(&self) -> u16;
}

pub type ExternalElf32Header = ExternalElfHeader<u32>;
pub type ExternalElf64Header = ExternalElfHeader<u64>;

//---------------------------------------------------------------------------------------------------------

pub struct ExternalElfHeaderWrapper<'a, T: PrimInt + Unsigned> {
    header: &'a dyn ExternalElfHeaderTrait<T>,
}

impl<'a, T: PrimInt + Unsigned> ExternalElfHeaderWrapper<'a, T> {
    fn fix_endian<I: PrimInt>(&self, val: I) -> I {
        match self.header.endianness() {
            1 => val,
            2 => val.to_be(),
            _ => {
                panic!();
            }
        }
    }

    pub fn new<R: super::ExternalElfTrait>(gen_elf: &'a GenericExternalElf<'a, R>) -> Self
    where
        <R as super::ExternalElfTrait>::ElfHeader: ExternalElfHeaderTrait<T>,
    {
        unsafe {
            Self {
                header: gen_elf.elf_header_raw(),
            }
        }
    }

    pub fn from_raw_parts(header: &impl ExternalElfHeaderTrait<T>) -> ExternalElfHeaderWrapper<T> {
        ExternalElfHeaderWrapper::<T> { header }
    }
}

impl<T: PrimInt + Unsigned> ExternalElfHeaderTrait<T> for ExternalElfHeaderWrapper<'_, T> {
    fn class(&self) -> u8 {
        self.header.class()
    }

    fn endianness(&self) -> u8 {
        self.header.endianness()
    }

    fn header_version(&self) -> u8 {
        self.header.header_version()
    }

    fn abi(&self) -> u8 {
        self.header.abi()
    }

    fn abi_version(&self) -> u8 {
        self.header.abi_version()
    }

    fn elftype(&self) -> u16 {
        self.fix_endian(self.header.elftype())
    }

    fn machine(&self) -> u16 {
        self.fix_endian(self.header.machine())
    }

    fn elf_version(&self) -> u32 {
        self.fix_endian(self.header.elf_version())
    }

    fn entry_point(&self) -> T {
        self.fix_endian(self.header.entry_point())
    }

    fn program_header_offset(&self) -> T {
        self.fix_endian(self.header.program_header_offset())
    }

    fn section_header_offset(&self) -> T {
        self.fix_endian(self.header.section_header_offset())
    }

    fn flags(&self) -> u32 {
        self.fix_endian(self.header.flags())
    }

    fn elf_header_size(&self) -> u16 {
        self.fix_endian(self.header.elf_header_size())
    }

    fn program_header_entry_size(&self) -> u16 {
        self.fix_endian(self.header.program_header_entry_size())
    }

    fn program_header_entry_num(&self) -> u16 {
        self.fix_endian(self.header.program_header_entry_num())
    }

    fn section_header_entry_size(&self) -> u16 {
        self.fix_endian(self.header.section_header_entry_size())
    }

    fn section_header_entry_num(&self) -> u16 {
        self.fix_endian(self.header.section_header_entry_num())
    }

    fn shstr_index(&self) -> u16 {
        self.fix_endian(self.header.shstr_index())
    }
}
