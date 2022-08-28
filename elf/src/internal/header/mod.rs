use self::header_util::*;

pub mod header_util;

#[derive(Debug)]
pub struct ElfHeader {
    pub(crate) _class: ElfClass,
    pub(crate) _endianness: ElfEndian,
    pub(crate) _abi: ElfAbi,
    pub(crate) _elftype: ElfType,
    pub(crate) _machine: ElfMachine,
    pub(crate) _entry: u128,
    pub(crate) _flags: u32,
}

impl ElfHeader {
    pub fn from_external_header<T: num_traits::PrimInt + num_traits::Unsigned + Into<u128>>(
        header: &impl crate::external::header::ExternalElfHeaderTrait<T>,
    ) -> Self {
        Self {
            _class: ElfClass::try_from(header.class()).unwrap(),
            _endianness: ElfEndian::try_from(header.endianness()).unwrap(),
            _abi: ElfAbi::try_from(header.abi()).unwrap(),
            _elftype: ElfType::try_from(header.elftype()).unwrap(),
            _machine: ElfMachine::try_from(header.machine()).unwrap(),
            _entry: header.entry_point().into(),
            _flags: header.flags(),
        }
    }
}
