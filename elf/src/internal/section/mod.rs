

pub struct InternalSectionHeader{
    name: String,
    sh_type: SectionType,
    flags: SectionFlags,
    mem_addr: u128,
    mem_addr_align: u128,
    link: (),
}

//-----------------------------------------------------------------

const SHT_LOOS: u32 = 0x60000000;
const SHT_HIOS: u32 = 0x6fffffff;
const SHT_LOPROC: u32 = 0x70000000;
const SHT_HIPROC: u32 = 0x7fffffff;
const SHT_LOUSER: u32 = 0x80000000;
const SHT_HIUSER: u32 = 0xffffffff;
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SectionType {
    SHT_NULL,          
    SHT_PROGBITS,      
    SHT_SYMTAB,        
    SHT_STRTAB,        
    SHT_RELA,          
    SHT_HASH,          
    SHT_DYNAMIC,       
    SHT_NOTE,          
    SHT_NOBITS,        
    SHT_REL,           
    SHT_SHLIB,         
    SHT_DYNSYM,        
    SHT_INIT_ARRAY,    
    SHT_FINI_ARRAY,    
    SHT_PREINIT_ARRAY, 
    SHT_GROUP,         
    SHT_SYMTAB_SHNDX,  
    SHT_NUM,           
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
            0x13 => Ok(SectionType::SHT_NUM),
            val @ SHT_LOOS..=SHT_HIOS => Ok(SectionType::OsSpecific(val)),
            val @ SHT_LOPROC..=SHT_HIPROC => Ok(SectionType::ProcessorSpecific(val)),
            val @ SHT_LOUSER..=SHT_HIUSER => Ok(SectionType::ApplicationSpecific(val)),
            _ => Result::Err(())
        }
    }
}


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