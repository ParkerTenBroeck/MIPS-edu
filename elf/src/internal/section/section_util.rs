//----------------------------------------------------------------------------

const SHT_LOOS: u32 = 0x60000000;
const SHT_HIOS: u32 = 0x6fffffff;
const SHT_LOPROC: u32 = 0x70000000;
const SHT_HIPROC: u32 = 0x7fffffff;
const SHT_LOUSER: u32 = 0x80000000;
const SHT_HIUSER: u32 = 0xffffffff;
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SectionType {
    /** Section header table entry unused */
    SHT_NULL,
    /** Program specific (private) data */
    SHT_PROGBITS,
    /** Link editing symbol table */
    SHT_SYMTAB,
    /** A string table */
    SHT_STRTAB,
    /** Relocation entries with addends */
    SHT_RELA,
    /** A symbol hash table */
    SHT_HASH,
    /** Information for dynamic linking */
    SHT_DYNAMIC,
    /** Information that marks file */
    SHT_NOTE,
    /** Section occupies no space in file */
    SHT_NOBITS,
    /** Relocation entries, no addends */
    SHT_REL,
    /** Reserved, unspecified semantics */
    SHT_SHLIB,
    /** Dynamic linking symbol table */
    SHT_DYNSYM,
    /** Array of ptrs to init functions */
    SHT_INIT_ARRAY,
    /** Array of ptrs to finish functions */
    SHT_FINI_ARRAY,
    /** Array of ptrs to pre-init funcs */
    SHT_PREINIT_ARRAY,
    /** Section contains a section group */
    SHT_GROUP,
    /** Indicies for SHN_XINDEX entries */
    SHT_SYMTAB_SHNDX,
    OsSpecific(u32),
    ProcessorSpecific(u32),
    ApplicationSpecific(u32),
}

impl TryFrom<u32> for SectionType {
    type Error = ();
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            0x0 => Ok(SectionType::SHT_NULL),
            0x1 => Ok(SectionType::SHT_PROGBITS),
            0x2 => Ok(SectionType::SHT_SYMTAB),
            0x3 => Ok(SectionType::SHT_STRTAB),
            0x4 => Ok(SectionType::SHT_RELA),
            0x5 => Ok(SectionType::SHT_HASH),
            0x6 => Ok(SectionType::SHT_DYNAMIC),
            0x7 => Ok(SectionType::SHT_NOTE),
            0x8 => Ok(SectionType::SHT_NOBITS),
            0x9 => Ok(SectionType::SHT_REL),
            0x0A => Ok(SectionType::SHT_SHLIB),
            0x0B => Ok(SectionType::SHT_DYNSYM),
            0x0E => Ok(SectionType::SHT_INIT_ARRAY),
            0x0F => Ok(SectionType::SHT_FINI_ARRAY),
            0x10 => Ok(SectionType::SHT_PREINIT_ARRAY),
            0x11 => Ok(SectionType::SHT_GROUP),
            0x12 => Ok(SectionType::SHT_SYMTAB_SHNDX),
            val @ SHT_LOOS..=SHT_HIOS => Ok(SectionType::OsSpecific(val)),
            val @ SHT_LOPROC..=SHT_HIPROC => Ok(SectionType::ProcessorSpecific(val)),
            val @ SHT_LOUSER..=SHT_HIUSER => Ok(SectionType::ApplicationSpecific(val)),
            _ => Result::Err(()),
        }
    }
}

impl Into<u32> for SectionType {
    fn into(self) -> u32 {
        match self {
            SectionType::SHT_NULL => 0x0,
            SectionType::SHT_PROGBITS => 0x01,
            SectionType::SHT_SYMTAB => 0x02,
            SectionType::SHT_STRTAB => 0x03,
            SectionType::SHT_RELA => 0x04,
            SectionType::SHT_HASH => 0x05,
            SectionType::SHT_DYNAMIC => 0x06,
            SectionType::SHT_NOTE => 0x07,
            SectionType::SHT_NOBITS => 0x08,
            SectionType::SHT_REL => 0x09,
            SectionType::SHT_SHLIB => 0x0A,
            SectionType::SHT_DYNSYM => 0x0B,
            SectionType::SHT_INIT_ARRAY => 0x0E,
            SectionType::SHT_FINI_ARRAY => 0x0F,
            SectionType::SHT_PREINIT_ARRAY => 0x10,
            SectionType::SHT_GROUP => 0x11,
            SectionType::SHT_SYMTAB_SHNDX => 0x12,
            SectionType::OsSpecific(val) => val,
            SectionType::ProcessorSpecific(val) => val,
            SectionType::ApplicationSpecific(val) => val,
        }
    }
}

//----------------------------------------------------------------------------
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum MipsProcessorSpesificSectionType {
    /** Section contains the set of dynamic shared objects used when
    statically linking.  */
    SHT_MIPS_LIBLIST = 0x70000000,
    /** I'm not sure what this is, but it's used on Irix 5.  */
    SHT_MIPS_MSYM = 0x70000001,
    /** Section contains list of symbols whose definitions conflict with
    symbols defined in shared objects.  */
    SHT_MIPS_CONFLICT = 0x70000002,
    /** Section contains the global pointer table.  */
    SHT_MIPS_GPTAB = 0x70000003,
    /** Section contains microcode information.  The exact format is
    unspecified.  */
    SHT_MIPS_UCODE = 0x70000004,
    /** Section contains some sort of debugging information.  The exact
    format is unspecified.  It's probably ECOFF symbols.  */
    SHT_MIPS_DEBUG = 0x70000005,
    /** Section contains register usage information.  */
    SHT_MIPS_REGINFO = 0x70000006,
    /** ??? */
    SHT_MIPS_PACKAGE = 0x70000007,
    /** ??? */
    SHT_MIPS_PACKSYM = 0x70000008,
    /** ??? */
    SHT_MIPS_RELD = 0x70000009,
    /** Section contains interface information.  */
    SHT_MIPS_IFACE = 0x7000000b,
    /** Section contains description of contents of another section.  */
    SHT_MIPS_CONTENT = 0x7000000c,
    /** Section contains miscellaneous options.  */
    SHT_MIPS_OPTIONS = 0x7000000d,
    /** ??? */
    SHT_MIPS_SHDR = 0x70000010,
    /** ??? */
    SHT_MIPS_FDESC = 0x70000011,
    /** ??? */
    SHT_MIPS_EXTSYM = 0x70000012,
    /** ??? */
    SHT_MIPS_DENSE = 0x70000013,
    /** ??? */
    SHT_MIPS_PDESC = 0x70000014,
    /** ??? */
    SHT_MIPS_LOCSYM = 0x70000015,
    /** ??? */
    SHT_MIPS_AUXSYM = 0x70000016,
    /** ??? */
    SHT_MIPS_OPTSYM = 0x70000017,
    /** ??? */
    SHT_MIPS_LOCSTR = 0x70000018,
    /** ??? */
    SHT_MIPS_LINE = 0x70000019,
    /** ??? */
    SHT_MIPS_RFDESC = 0x7000001a,
    /** Delta C++: symbol table */
    SHT_MIPS_DELTASYM = 0x7000001b,
    /** Delta C++: instance table */
    SHT_MIPS_DELTAINST = 0x7000001c,
    /** Delta C++: class table */
    SHT_MIPS_DELTACLASS = 0x7000001d,
    /** DWARF debugging section.  */
    SHT_MIPS_DWARF = 0x7000001e,
    /** Delta C++: declarations */
    SHT_MIPS_DELTADECL = 0x7000001f,
    /** List of libraries the binary depends on.  Includes a time stamp, version
    number.  */
    SHT_MIPS_SYMBOL_LIB = 0x70000020,
    /** Events section.  */
    SHT_MIPS_EVENTS = 0x70000021,
    /** ??? */
    SHT_MIPS_TRANSLATE = 0x70000022,
    /** Special pixie sections */
    SHT_MIPS_PIXIE = 0x70000023,
    /** Address translation table (for debug info) */
    SHT_MIPS_XLATE = 0x70000024,
    /** SGI internal address translation table (for debug info) */
    SHT_MIPS_XLATE_DEBUG = 0x70000025,
    /** Intermediate code */
    SHT_MIPS_WHIRL = 0x70000026,
    /** C++ exception handling region info */
    SHT_MIPS_EH_REGION = 0x70000027,
    /** Obsolete address translation table (for debug info) */
    SHT_MIPS_XLATE_OLD = 0x70000028,
    /** Runtime procedure descriptor table exception information (ucode) ??? */
    SHT_MIPS_PDR_EXCEPTION = 0x70000029,
}

impl TryFrom<u32> for MipsProcessorSpesificSectionType {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match num_traits::FromPrimitive::from_u32(value) {
            Some(val) => Result::Ok(val),
            None => Result::Err(()),
        }
    }
}

impl TryInto<u32> for MipsProcessorSpesificSectionType {
    type Error = ();
    fn try_into(self) -> Result<u32, Self::Error> {
        match num_traits::ToPrimitive::to_u32(&self) {
            Some(val) => Result::Ok(val),
            None => Result::Err(()),
        }
    }
}

//----------------------------------------------------------------------------

bitflags::bitflags! {
    pub struct SectionFlags: u64 {
        const SHF_WRITE             = 0x1;
        const SHF_ALLOC             = 0x2;
        const SHF_EXECINSTR         = 0x4;
        const SHF_MERGE             = 0x10;
        const SHF_STRINGS           = 0x20;
        const SHF_INFO_LINK         = 0x40;
        const SHF_LINK_ORDER        = 0x80;
        const SHF_OS_NONCONFORMING  = 0x100;
        const SHF_GROUP             = 0x200;
        const SHF_TLS	            = 0x400;
        const SHF_MASKOS            = 0x0ff00000;
        const SHF_MASKPROC          = 0xf0000000;
        const SHF_ORDERED           = 0x40000000;
        const SHF_EXCLUDE           = 0x80000000;
    }
}
