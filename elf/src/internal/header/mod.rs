

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

bitflags::bitflags! {
    pub struct MipsMachineFlags: u32 {
        ///At least one .noreorder directive appears in the source.  
        const EF_MIPS_NOREORDER     = 0x00000001;
        ///File contains position independent code.  
        const EF_MIPS_PIC           = 0x00000002;
        ///Code in file uses the standard calling sequence for calling position independent code.  
        const EF_MIPS_CPIC          = 0x00000004;
        ///???  Unknown flag, set in IRIX 6's BSDdup2.o in libbsd.a.  
        const EF_MIPS_XGOT          = 0x00000008;
        ///Code in file uses UCODE (obsolete) 
        const EF_MIPS_UCODE         = 0x00000010;
        ///Code in file uses new ABI (-n32 on Irix 6).  
        const EF_MIPS_ABI2          = 0x00000020;
        ///Process the .MIPS.options section first by ld 
        const EF_MIPS_OPTIONS_FIRST = 0x00000080;
        ///Indicates code compiled for a 64-bit machine in 32-bit mode. (regs are 32-bits wide.)
        const EF_MIPS_32BITMODE     = 0x00000100;

        ///Machine variant if we know it.  This field was invented at Cygnus,
        ///but it is hoped that other vendors will adopt it.  If some standard
        ///is developed, this code should be changed to follow it.
        const EF_MIPS_MACH          = 0x00FF0000;

        /* Cygnus is choosing values between 80 and 9F;
        00 - 7F should be left for a future standard;
        the rest are open. */

        const E_MIPS_MACH_3900      = 0x00810000;
        const E_MIPS_MACH_4010      = 0x00820000;
        const E_MIPS_MACH_4100      = 0x00830000;
        const E_MIPS_MACH_4650      = 0x00850000;
        const E_MIPS_MACH_4120      = 0x00870000;
        const E_MIPS_MACH_4111      = 0x00880000;
        const E_MIPS_MACH_SB1       = 0x008a0000;
        const E_MIPS_MACH_OCTEON    = 0x008b0000;
        const E_MIPS_MACH_XLR       = 0x008c0000;
        const E_MIPS_MACH_5400      = 0x00910000;
        const E_MIPS_MACH_5500      = 0x00980000;
        const E_MIPS_MACH_9000      = 0x00990000;
        const E_MIPS_MACH_LS2E      = 0x00A00000;
        const E_MIPS_MACH_LS2F      = 0x00A10000;

        ///The ABI of the file.  Also see EF_MIPS_ABI2 above.
        const EF_MIPS_ABI           = 0x0000F000;
        ///The original o32 abi.
        const E_MIPS_ABI_O32        = 0x00001000;
        ///O32 extended to work on 64 bit architectures
        const E_MIPS_ABI_O64        = 0x00002000;
        ///EABI in 32 bit mode
        const E_MIPS_ABI_EABIO32    = 0x00003000;
        ///EABI in 64 bit mode
        const E_MIPS_ABI_EBAIO64    = 0x00004000;

        ///Architectural Extensions used by this file 
        const EF_MIPS_ARCH_ASE      = 0x0f000000;
        ///Use MDMX multimedia extensions 
        const EF_MIPS_ARCH_ASE_MDMX = 0x08000000;
        ///Use MIPS-16 ISA extensions 
        const EF_MIPS_ARCH_ASE_M16  = 0x04000000;

        ///Four bit MIPS architecture field.  
        const EF_MIPS_ARCH          = 0xf0000000;
        ///-mips1 code.  
        const E_MIPS_ARCH_1         = 0x00000000;
        ///-mips2 code.  
        const E_MIPS_ARCH_2         = 0x10000000;
        ///-mips3 code.  
        const E_MIPS_ARCH_3         = 0x20000000;
        ///-mips4 code.  
        const E_MIPS_ARCH_4         = 0x30000000;
        ///-mips5 code.  
        const E_MIPS_ARCH_5         = 0x40000000;
        ///-mips32 code.  
        const E_MIPS_ARCH_32        = 0x50000000;
        ///-mips64 code.  
        const E_MIPS_ARCH_32R2      = 0x70000000;
        ///-mips32r2 code.  
        const E_MIPS_ARCH_64        = 0x60000000;
        ///-mips64r2 code.  
        const E_MIPS_ARCH_64R2      = 0x80000000;
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