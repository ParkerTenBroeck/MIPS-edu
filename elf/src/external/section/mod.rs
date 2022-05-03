use core::ptr::read_unaligned;

use num_traits::{PrimInt, Unsigned};

use super::{header::ExternalElfHeaderTrait, GenericExternalElf, ExternalElfTrait};


#[repr(C)]
#[derive(Debug)]
pub struct ExternalSectionHeader<T: Unsigned + PrimInt> {
    sh_name: u32,
    sh_type: u32,
    sh_flags: T,
    sh_addr: T,
    sh_offset: T,
    sh_size: T,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: T,
    sh_entsize: T,
}

impl<T: PrimInt + Unsigned> ExternalSectionHeaderTrait<T> for ExternalSectionHeader<T> {
    fn name_off(&self) -> u32 {
        unsafe { read_unaligned(&self.sh_name) }
    }

    fn sh_type(&self) -> u32 {
        unsafe { read_unaligned(&self.sh_type) }
    }

    fn flags(&self) -> T {
        unsafe { read_unaligned(&self.sh_flags) }
    }

    fn addr(&self) -> T {
        unsafe { read_unaligned(&self.sh_addr) }
    }

    fn offset(&self) -> T {
        unsafe { read_unaligned(&self.sh_offset) }
    }

    fn size(&self) -> T {
        unsafe { read_unaligned(&self.sh_size) }
    }

    fn link(&self) -> u32 {
        unsafe { read_unaligned(&self.sh_link) }
    }

    fn info(&self) -> u32 {
        unsafe { read_unaligned(&self.sh_info) }
    }

    fn addralign(&self) -> T {
        unsafe { read_unaligned(&self.sh_addralign) }
    }

    fn entsize(&self) -> T {
        unsafe { read_unaligned(&self.sh_entsize) }
    }
}

pub trait ExternalSectionHeaderTrait<T: PrimInt + Unsigned>{
    fn name_off(&self) -> u32;
    fn sh_type(&self) -> u32;
    fn flags(&self) -> T;
    fn addr(&self) -> T;
    fn offset(&self) -> T;
    fn size(&self) -> T;
    fn link(&self) -> u32;
    fn info(&self) -> u32;
    fn addralign(&self) -> T;
    fn entsize(&self) -> T;
}



pub type ExternalSectionHeader32 = ExternalSectionHeader<u32>;
pub type ExternalSectionHeader64 = ExternalSectionHeader<u64>;

//---------------------------------------------------------------------------------------------------------

pub struct ExternalSectionHeaderWrapper<'a, T: ExternalElfTrait>{
    elf_header: &'a dyn ExternalElfHeaderTrait<T::Size>,
    section_header: &'a dyn ExternalSectionHeaderTrait<T::Size>,
    gen_elf: &'a GenericExternalElf<'a, T>
}

impl<'a, T: ExternalElfTrait> ExternalSectionHeaderWrapper<'a, T>{
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

    pub fn new(index: usize, gen_elf: &'a GenericExternalElf<'a, T>) -> Option<Self>{
        unsafe{
            match gen_elf.section_headers_raw().get(index){
                Some(section_header) => {
                    Option::Some(Self{
                        elf_header: gen_elf.elf_header_raw(),
                        section_header,
                        gen_elf,
                    })
                },
                None => Option::None,
            }
        }
    }

    pub fn get_data(&self) -> &'a [u8]{
        let start = self.offset().try_into().unwrap();
        let end = self.offset().try_into().unwrap() + self.size().try_into().unwrap();
        &self.gen_elf.data[start..end]
    }

    pub fn get_name(&self) -> &'a str{
        
        match self.gen_elf.section_header(self.gen_elf.elf_header().shstr_index() as usize){
            Some(val) => {
                
                let start = self.name_off() as usize;
                let mut end = self.name_off() as usize;
                let tmp = &val.get_data()[start..];
                for char in core::str::from_utf8(tmp).unwrap().chars(){
                    if char == '\0'{
                        break;
                    }
                    end += char.len_utf8();
                }

                let tmp = &val.get_data()[start..end];
                core::str::from_utf8(tmp).unwrap()
            },
            None => "",
        }
    }
}

impl<T: ExternalElfTrait> ExternalSectionHeaderTrait<T::Size> for ExternalSectionHeaderWrapper<'_, T>{
    fn name_off(&self) -> u32 {
        self.fix_endian(self.section_header.name_off())
    }

    fn sh_type(&self) -> u32 {
        self.fix_endian(self.section_header.sh_type())
    }

    fn flags(&self) -> T::Size {
        self.fix_endian(self.section_header.flags())
    }

    fn addr(&self) -> T::Size {
        self.fix_endian(self.section_header.addr())
    }

    fn offset(&self) -> T::Size {
        self.fix_endian(self.section_header.offset())
    }

    fn size(&self) -> T::Size {
        self.fix_endian(self.section_header.size())
    }

    fn link(&self) -> u32 {
        self.fix_endian(self.section_header.link())
    }

    fn info(&self) -> u32 {
        self.fix_endian(self.section_header.info())
    }

    fn addralign(&self) -> T::Size {
        self.fix_endian(self.section_header.addralign())
    }

    fn entsize(&self) -> T::Size {
        self.fix_endian(self.section_header.entsize())
    }
}