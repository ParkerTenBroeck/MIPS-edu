pub struct InternalProgramHeader {
    pub(crate) p_type: ProgramHeaderType,
    pub(crate) flags: ProgramHeaderFlags,
    pub(crate) virtual_offset: u128,
    pub(crate) physical_address_padding: u128,
    pub(crate) idk_yet: u128,
    pub(crate) memory_size: u128,
}

//----------------------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ProgramHeaderType {
    PT_NULL = 0,            /* Program header table entry unused */
    PT_LOAD = 1,            /* Loadable program segment */
    PT_DYNAMIC = 2,         /* Dynamic linking information */
    PT_INTERP = 3,          /* Program interpreter */
    PT_NOTE = 4,            /* Auxiliary information */
    PT_SHLIB = 5,           /* Reserved, unspecified semantics */
    PT_PHDR = 6,            /* Entry for header table itself */
    PT_TLS = 7,             /* Thread local storage segment */
    PT_LOOS = 0x60000000,   /* OS-specific */
    PT_HIOS = 0x6fffffff,   /* OS-specific */
    PT_LOPROC = 0x70000000, /* Processor-specific */
    PT_HIPROC = 0x7FFFFFFF, /* Processor-specific */

    PT_GNU_EH_FRAME = (0x60000000 + 0x474e550), /* Frame unwind information */
    PT_GNU_STACK = (0x60000000 + 0x474e551),    /* Stack flags */
    PT_GNU_RELRO = (0x60000000 + 0x474e552),    /* Read-only after relocation */
    PT_GNU_PROPERTY = (0x60000000 + 0x474e553), /* GNU property */

    /* OpenBSD segment types.  */
    PT_OPENBSD_RANDOMIZE = (0x60000000 + 0x5a3dbe6), /* Fill with random data.  */
    PT_OPENBSD_WXNEEDED = (0x60000000 + 0x5a3dbe7),  /* Program does W^X violations.  */
    PT_OPENBSD_BOOTDATA = (0x60000000 + 0x5a41be6),  /* Section for boot arguments.  */

    /* Mbind segments */
    PT_GNU_MBIND_NUM = 4096,
    PT_GNU_MBIND_LO = (0x60000000 + 0x474e555),
    PT_GNU_MBIND_HI = (0x60000000 + 0x474e555 + 4096 - 1),
}

impl TryFrom<u32> for ProgramHeaderType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match num_traits::FromPrimitive::from_u32(value) {
            Some(val) => Result::Ok(val),
            None => Result::Err(()),
        }
    }
}

impl ProgramHeaderType {
    pub fn to_u32(&self) -> u32 {
        num_traits::ToPrimitive::to_u32(self).unwrap()
    }
}

impl Into<u32> for ProgramHeaderType {
    fn into(self) -> u32 {
        num_traits::ToPrimitive::to_u32(&self).unwrap()
    }
}

//----------------------------------------------------------------------------------

bitflags::bitflags! {
    pub struct ProgramHeaderFlags: u64 {
        const PF_X = 0x01;
        const PF_W = 0x02;
        const PF_R = 0x04;
        const PF_MASKOS = 0x0FF00000;
        const PF_MASKPROC = 0xF0000000;
    }
}
