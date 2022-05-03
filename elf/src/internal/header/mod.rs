

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


//----------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass{
    Elf32,
    Elf64,
}

impl TryFrom<u8> for ElfClass {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(ElfClass::Elf32),
            2 => Ok(ElfClass::Elf64),
            _ => Result::Err(()),
        }
    }
}


//----------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfEndian{
    LittleEndian, 
    BigEndian,
}

impl TryFrom<u8> for ElfEndian {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(ElfEndian::LittleEndian),
            2 => Ok(ElfEndian::BigEndian),
            _ => Result::Err(()),
        }
    }
}


//----------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfAbi {
    SystemV,       
    HPUX,          
    NetBSD,        
    Linux,         
    Hurd,          
    Solaris,       
    AIX,           
    IRIX,          
    FreeBSD,       
    Tru64,         
    NovellModesto, 
    OpenBSD,       
    OpenVMS,       
    NonStopKernel, 
    AROS,          
    FenixOS,       
    CloudABI,      
}

impl TryFrom<u8> for ElfAbi {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(ElfAbi::SystemV),
            0x01 => Ok(ElfAbi::HPUX),
            0x02 => Ok(ElfAbi::NetBSD),
            0x03 => Ok(ElfAbi::Linux),
            0x04 => Ok(ElfAbi::Hurd),
            0x06 => Ok(ElfAbi::Solaris),
            0x07 => Ok(ElfAbi::AIX),
            0x08 => Ok(ElfAbi::IRIX),
            0x09 => Ok(ElfAbi::FreeBSD),
            0x0A => Ok(ElfAbi::Tru64),
            0x0B => Ok(ElfAbi::NovellModesto),
            0x0C => Ok(ElfAbi::OpenBSD),
            0x0D => Ok(ElfAbi::OpenVMS),
            0x0E => Ok(ElfAbi::NonStopKernel),
            0x0F => Ok(ElfAbi::AROS),
            0x10 => Ok(ElfAbi::FenixOS),
            0x11 => Ok(ElfAbi::CloudABI),
            _ => Result::Err(()),
        }
    }
}


//----------------------------------------------------------------------

const ET_LOOS: u16 = 0xfe00u16;
const ET_HIOS: u16 = 0xfeffu16;
const ET_LOPROC: u16 = 0xff00u16;
const ET_HIPROC: u16 = 0xffffu16;
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfType {
    ET_NONE,
    ET_REL,
    ET_EXEC,
    ET_DYN,
    ET_CORE,
    OsSpecific(u16),
    ProcessorSpecific(u16),
}

impl TryFrom<u16> for ElfType{
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value{
            0x00 => Ok(ElfType::ET_NONE),
            0x01 => Ok(ElfType::ET_REL),
            0x02 => Ok(ElfType::ET_EXEC),
            0x03 => Ok(ElfType::ET_DYN),
            0x04 => Ok(ElfType::ET_CORE),
            val @ ET_LOOS..=ET_HIOS => Ok(ElfType::OsSpecific(val)),
            val @ ET_LOPROC..=ET_HIPROC => Ok(ElfType::ProcessorSpecific(val)),
            _ => Result::Err(())
        }
    }
}


//----------------------------------------------------------------------


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfMachine {
    Unknown, 
    SPARC,   
    x86,     
    MIPS,    
    PowerPC, 
    S390,    
    ARM,     
    SuperH,  
    IA_64,   
    x86_64,  
    AArch64, 
    RISC_V,  
}

impl TryFrom<u16> for ElfMachine {
    type Error = ();
    fn try_from(n: u16) -> Result<Self, Self::Error> {
        match n {
            0x00 => Ok(ElfMachine::Unknown),
            0x02 => Ok(ElfMachine::SPARC),
            0x03 => Ok(ElfMachine::x86),
            0x08 => Ok(ElfMachine::MIPS),
            0x14 => Ok(ElfMachine::PowerPC),
            0x16 => Ok(ElfMachine::S390),
            0x28 => Ok(ElfMachine::ARM),
            0x2A => Ok(ElfMachine::SuperH),
            0x32 => Ok(ElfMachine::IA_64),
            0x3E => Ok(ElfMachine::x86_64),
            0xB7 => Ok(ElfMachine::AArch64),
            0xF3 => Ok(ElfMachine::RISC_V),
            _ => Result::Err(())
        }
    }
}
//----------------------------------------------------------------------