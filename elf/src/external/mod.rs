use core::marker::PhantomData;
use std::{
    fmt::{Display, UpperHex},
    num::TryFromIntError,
};

use num_traits::{PrimInt, Unsigned};

use self::{
    header::{
        ExternalElf32Header, ExternalElf64Header, ExternalElfHeaderTrait, ExternalElfHeaderWrapper,
    },
    program::{
        ExternalProgramHeader32, ExternalProgramHeader64, ExternalProgramHeaderTrait,
        ExternalProgramHeaderWrapper,
    },
    section::{
        ExternalSectionHeader32, ExternalSectionHeader64, ExternalSectionHeaderTrait,
        ExternalSectionHeaderWrapper,
    },
};

pub mod header;
pub mod program;
pub mod section;

pub trait ExternalElfTrait {
    type Size: PrimInt
        + Unsigned
        + Display
        + UpperHex
        + Into<u128>
        + TryInto<usize, Error = TryFromIntError>;
    type ElfHeader: ExternalElfHeaderTrait<Self::Size>;
    type ProgramHeader: ExternalProgramHeaderTrait<Self::Size>;
    type SectionHeader: ExternalSectionHeaderTrait<Self::Size>;
}

pub struct ExternalElf32 {}

impl ExternalElfTrait for ExternalElf32 {
    type Size = u32;
    type ElfHeader = ExternalElf32Header;
    type ProgramHeader = ExternalProgramHeader32;
    type SectionHeader = ExternalSectionHeader32;
}

pub struct ExternalElf64 {}

impl ExternalElfTrait for ExternalElf64 {
    type Size = u64;
    type ElfHeader = ExternalElf64Header;
    type ProgramHeader = ExternalProgramHeader64;
    type SectionHeader = ExternalSectionHeader64;
}

pub enum TernaryResult<T, V, E> {
    Ok1(T),
    Ok2(V),
    Err(E),
}

pub struct GenericExternalElf<'a, ET> {
    data: &'a [u8],
    pha: PhantomData<ET>,
}

impl<'a, T: ExternalElfTrait> GenericExternalElf<'a, T> {
    pub unsafe fn elf_header_raw(&self) -> &T::ElfHeader {
        &*(self.data.as_ptr() as *const T::ElfHeader)
    }

    pub unsafe fn section_headers_raw(&'a self) -> &'a [T::SectionHeader] {
        let sh_off: usize = match self.elf_header().section_header_offset().try_into() {
            Ok(val) => val,
            Err(_err) => {
                panic!()
            }
        };
        let sh_num: usize = match self.elf_header().section_header_entry_num().try_into() {
            Ok(val) => val,
            Err(_err) => {
                panic!()
            }
        };
        let sh_ptr = self.data.as_ptr().add(sh_off);
        core::slice::from_raw_parts(sh_ptr as *const T::SectionHeader, sh_num)
    }

    pub unsafe fn program_headers_raw(&'a self) -> &'a [T::ProgramHeader] {
        let ph_off: usize = match self.elf_header().program_header_offset().try_into() {
            Ok(val) => val,
            Err(_err) => {
                panic!()
            }
        };
        let ph_num: usize = match self.elf_header().program_header_entry_num().try_into() {
            Ok(val) => val,
            Err(_err) => {
                panic!()
            }
        };
        let sh_ptr = self.data.as_ptr().add(ph_off);
        core::slice::from_raw_parts(sh_ptr as *const T::ProgramHeader, ph_num)
    }

    pub fn elf_header(&'a self) -> ExternalElfHeaderWrapper<T::Size> {
        ExternalElfHeaderWrapper::new(self)
    }
    pub fn section_header(&'a self, index: usize) -> Option<ExternalSectionHeaderWrapper<T>> {
        ExternalSectionHeaderWrapper::new(index, self)
    }
    pub fn program_header(&'a self, index: usize) -> Option<ExternalProgramHeaderWrapper<T::Size>> {
        ExternalProgramHeaderWrapper::new(index, self)
    }
}

pub fn from_bytes<'a>(
    buf: &'a [u8],
) -> TernaryResult<GenericExternalElf<ExternalElf32>, GenericExternalElf<ExternalElf64>, ()> {
    match buf.get(0x04) {
        Option::Some(val) => match val {
            0x1 => {
                if buf.len() < core::mem::size_of::<ExternalElf32Header>() {
                    TernaryResult::Err(())
                } else {
                    TernaryResult::Ok1(GenericExternalElf {
                        data: buf,
                        pha: PhantomData,
                    })
                }
            }
            0x2 => {
                if buf.len() < core::mem::size_of::<ExternalElf64Header>() {
                    TernaryResult::Err(())
                } else {
                    TernaryResult::Ok2(GenericExternalElf {
                        data: buf,
                        pha: PhantomData,
                    })
                }
            }
            _ => TernaryResult::Err(()),
        },
        Option::None => TernaryResult::Err(()),
    }
}

#[allow(unused_imports)]
pub mod tests {
    use crate::external::{
        from_bytes, header::ExternalElfHeaderTrait, section::ExternalSectionHeaderTrait,
    };
    use std::{fs::File, io::Read};

    use super::{ExternalElfTrait, GenericExternalElf};

    #[test]
    pub fn test1() {
        println!("current dir: {:?}", std::env::current_dir());
        let mut elf_file = File::open("res/mips_elf_test.o")
            .unwrap_or_else(|_| File::open("./elf/res/mips_elf_test.o").unwrap());
        let mut elf_buf = Vec::<u8>::new();
        elf_file
            .read_to_end(&mut elf_buf)
            .expect("read file failed");
        let buf = elf_buf.as_slice();

        match from_bytes(buf) {
            super::TernaryResult::Ok1(mut e32) => {
                println!("Elf 32");
                print_elf(&mut e32);
            }
            super::TernaryResult::Ok2(mut e64) => {
                println!("Elf 64");
                print_elf(&mut e64);
            }
            super::TernaryResult::Err(_err) => {
                println!("Invalid elf file!");
            }
        }
    }

    pub fn print_elf<T: ExternalElfTrait>(elf: &mut GenericExternalElf<T>) {
        println!("Elf Header");
        println!(
            "\tClass(big or little endian): {}",
            elf.elf_header().class()
        );
        println!("\tABI: 0x{:X}", elf.elf_header().abi());
        println!("\tABI Version: {}", elf.elf_header().abi_version());
        println!("\tType: 0x{:X}", elf.elf_header().elftype());
        println!("\tMachine: 0x{:X}", elf.elf_header().machine());
        println!("\tEntry: 0x{:X}", elf.elf_header().entry_point());
        println!("\tFlags: 0x{:X}", elf.elf_header().flags());
        println!();

        let mut index = 0;
        println!("Section Headers");
        while let Option::Some(section) = elf.section_header(index) {
            let name = section.get_name();
            println!("\tSection header: {} -> {}", index, name);
            println!("\t\tAddress:       {}", section.addr());
            println!("\t\tAddress Align: {}", section.addralign());
            println!("\t\tSize:          {}", section.size());
            println!("\t\ttype:          {:X}", section.sh_type());
            println!("\t\tFlags:         {}", section.flags());
            println!("\t\tLink:          {}\n", section.link());
            index += 1;
        }
        index = 0;
        while let Option::Some(_program) = elf.program_header(index) {}
    }
}
