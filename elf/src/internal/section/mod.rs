pub mod section_util;

use self::section_util::*;
use crate::external::{section::*, *};

pub struct InternalSectionHeader {
    _name: String,
    sh_type: SectionType,
    flags: SectionFlags,
    _mem_addr: u128,
    _mem_addr_align: u128,
    _link: (),
}

impl InternalSectionHeader {
    pub fn flags(&self) -> SectionFlags {
        self.flags
    }
    pub fn sh_type(&self) -> SectionType {
        self.sh_type
    }
}

impl InternalSectionHeader {
    pub fn from_external<T: ExternalElfTrait>(external: &ExternalSectionHeaderWrapper<T>) -> Self {
        Self {
            _name: external.get_name().into(),
            sh_type: SectionType::try_from(external.sh_type()).unwrap(),
            flags: unsafe { SectionFlags::from_bits_unchecked(external.flags().into() as u64) },
            _mem_addr: external.addr().into(),
            _mem_addr_align: external.addralign().into(),
            _link: (),
        }
    }
}
