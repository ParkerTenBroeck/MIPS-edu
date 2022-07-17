use self::header_util::*;

pub mod header_util;

#[derive(Debug)]
pub struct ElfHeader {
    pub(crate) class: ElfClass,
    pub(crate) endianness: ElfEndian,
    pub(crate) abi: ElfAbi,
    pub(crate) elftype: ElfType,
    pub(crate) machine: ElfMachine,
    pub(crate) entry: u128,
    pub(crate) flags: u32,
}

impl ElfHeader{
    pub fn from_external_header<T: num_traits::PrimInt + num_traits::Unsigned + Into<u128>>(header: &impl crate::external::header::ExternalElfHeaderTrait<T>) -> Self{
        Self{
            class: ElfClass::try_from(header.class()).unwrap(),
            endianness: ElfEndian::try_from(header.endianness()).unwrap(),
            abi: ElfAbi::try_from(header.abi()).unwrap(),
            elftype: ElfType::try_from(header.elftype()).unwrap(),
            machine: ElfMachine::try_from(header.machine()).unwrap(),
            entry: header.entry_point().into(),
            flags: header.flags(),
        }
    }
}
