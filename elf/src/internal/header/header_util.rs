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

impl Into<u8> for ElfClass{
    fn into(self) -> u8 {
        match self{
            ElfClass::Elf32 => 1,
            ElfClass::Elf64 => 2,
        }
    }
}


//----------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ElfEndian{
    LittleEndian = 0x01, 
    BigEndian    = 0x02,
}

impl TryFrom<u8> for ElfEndian {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match num_traits::FromPrimitive::from_u8(val){
            Option::Some(val) => Result::Ok(val),
            Option::None => Result::Err(())
        }
    }
}

impl Into<u8> for ElfEndian{
    fn into(self) -> u8 {
        num_traits::ToPrimitive::to_u8(&self).unwrap()
    }
}


//----------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ElfAbi {
    /** UNIX System V ABI */
    ELFOSABI_NONE	      =0,
    /** HP-UX operating system */
    ELFOSABI_HPUX	      =1,
    /** NetBSD */
    ELFOSABI_NETBSD	      =2,
    /** GNU/Linux */
    ELFOSABI_LINUX	      =3,
    /** GNU/Hurd */
    ELFOSABI_HURD	      =4,
    /** Solaris */
    ELFOSABI_SOLARIS      =6,
    /** AIX */
    ELFOSABI_AIX	      =7,
    /** IRIX */
    ELFOSABI_IRIX	      =8,
    /** FreeBSD */
    ELFOSABI_FREEBSD      =9,
    /** TRU64 UNIX */
    ELFOSABI_TRU64	      =10,
    /** Novell Modesto */
    ELFOSABI_MODESTO      =11,
    /** OpenBSD */
    ELFOSABI_OPENBSD      =12,
    /** OpenVMS */
    ELFOSABI_OPENVMS      =13,
    /** Hewlett-Packard Non-Stop Kernel */
    ELFOSABI_NSK	      =14,
    /** AROS */
    ELFOSABI_AROS	      =15,
    /** ARM */
    ELFOSABI_ARM	      =97,
    /** Standalone (embedded) application */     
    ELFOSABI_STANDALONE   =255,
}

impl TryFrom<u8> for ElfAbi {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match num_traits::FromPrimitive::from_u8(val){
            Option::Some(val) => Result::Ok(val),
            Option::None => Result::Err(())
        }
    }
}

impl Into<u8> for ElfAbi{
    fn into(self) -> u8 {
        num_traits::ToPrimitive::to_u8(&self).unwrap()
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

impl Into<u16> for ElfType{
    fn into(self) -> u16 {
        match self{
            ElfType::ET_NONE => 0x00,
            ElfType::ET_REL => 0x01,
            ElfType::ET_EXEC => 0x02,
            ElfType::ET_DYN => 0x03,
            ElfType::ET_CORE => 0x04,
            ElfType::OsSpecific(val) => val,
            ElfType::ProcessorSpecific(val) => val,
        }
    }
}


//----------------------------------------------------------------------


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
    /** Values for e_machine, which identifies the architecture.  These numbers
       are officially assigned by registry@sco.com.  See below for a list of
       ad-hoc numbers used during initial development.  */
pub enum ElfMachine {
    /** No machine */
    EM_NONE     =   0,
    /** AT&T WE 32100 */
    EM_M32     =   1,
    /** SUN SPARC */
    EM_SPARC     =   2,
    /** Intel 80386 */
    EM_386     =   3,
    /** Motorola m68k family */
    EM_68K     =   4,
    /** Motorola m88k family */
    EM_88K     =   5,
    /** Intel 80486 *//* Reserved for future use */
    EM_486     =   6,
    /** Intel 80860 */
    EM_860     =   7,
    /** MIPS R3000 (officially, big-endian only) */
    EM_MIPS     =   8,
    /** IBM System/370 */
    EM_S370     =   9,
    /** MIPS R3000 little-endian (Oct 4 1999 Draft) Deprecated */
    EM_MIPS_RS3_LE     =  10,
    /** Reserved */
    EM_res011     =  11,
    /** Reserved */
    EM_res012     =  12,
    /** Reserved */
    EM_res013     =  13,
    /** Reserved */
    EM_res014     =  14,
    /** HPPA */
    EM_PARISC     =  15,
    /** Reserved */
    EM_res016     =  16,
    /** Fujitsu VPP500 */
    EM_VPP550     =  17,
    /** Sun's "v8plus" */
    EM_SPARC32PLUS     =  18,
    /** Intel 80960 */
    EM_960     =  19,
    /** PowerPC */
    EM_PPC     =  20,
    /** 64-bit PowerPC */
    EM_PPC64     =  21,
    /** IBM S/390 */
    EM_S390     =  22,
    /** Sony/Toshiba/IBM SPU */
    EM_SPU     =  23,
    /** Reserved */
    EM_res024     =  24,
    /** Reserved */
    EM_res025     =  25,
    /** Reserved */
    EM_res026     =  26,
    /** Reserved */
    EM_res027     =  27,
    /** Reserved */
    EM_res028     =  28,
    /** Reserved */
    EM_res029     =  29,
    /** Reserved */
    EM_res030     =  30,
    /** Reserved */
    EM_res031     =  31,
    /** Reserved */
    EM_res032     =  32,
    /** Reserved */
    EM_res033     =  33,
    /** Reserved */
    EM_res034     =  34,
    /** Reserved */
    EM_res035     =  35,
    /** NEC V800 series */
    EM_V800     =  36,
    /** Fujitsu FR20 */
    EM_FR20     =  37,
    /** TRW RH32 */
    EM_RH32     =  38,
    /** Motorola M*Core */ /* May also be taken by Fujitsu MMA */
    EM_MCORE     =  39,
    ///** Old name for MCore */
    //EM_RCE     =  39,
    /** ARM */
    EM_ARM     =  40,
    /** Digital Alpha */
    EM_OLD_ALPHA     =  41,
    /** Renesas (formerly Hitachi) / SuperH SH */
    EM_SH     =  42,
    /** SPARC v9 64-bit */
    EM_SPARCV9     =  43,
    /** Siemens Tricore embedded processor */
    EM_TRICORE     =  44,
    /** ARC Cores */
    EM_ARC     =  45,
    /** Renesas (formerly Hitachi) H8/300 */
    EM_H8_300     =  46,
    /** Renesas (formerly Hitachi) H8/300H */
    EM_H8_300H     =  47,
    /** Renesas (formerly Hitachi) H8S */
    EM_H8S     =  48,
    /** Renesas (formerly Hitachi) H8/500 */
    EM_H8_500     =  49,
    /** Intel IA-64 Processor */
    EM_IA_64     =  50,
    /** Stanford MIPS-X */
    EM_MIPS_X     =  51,
    /** Motorola Coldfire */
    EM_COLDFIRE     =  52,
    /** Motorola M68HC12 */
    EM_68HC12     =  53,
    /** Fujitsu Multimedia Accelerator */
    EM_MMA     =  54,
    /** Siemens PCP */
    EM_PCP     =  55,
    /** Sony nCPU embedded RISC processor */
    EM_NCPU     =  56,
    /** Denso NDR1 microprocesspr */
    EM_NDR1     =  57,
    /** Motorola Star*Core processor */
    EM_STARCORE     =  58,
    /** Toyota ME16 processor */
    EM_ME16     =  59,
    /** STMicroelectronics ST100 processor */
    EM_ST100     =  60,
    /** Advanced Logic Corp. TinyJ embedded processor */
    EM_TINYJ     =  61,
    /** Advanced Micro Devices X86-64 processor */
    EM_X86_64     =  62,
    /** Sony DSP Processor */
    EM_PDSP     =  63,
    /** Digital Equipment Corp. PDP-10 */
    EM_PDP10     =  64,
    /** Digital Equipment Corp. PDP-11 */
    EM_PDP11     =  65,
    /** Siemens FX66 microcontroller */
    EM_FX66     =  66,
    /** STMicroelectronics ST9+ 8/16 bit microcontroller */
    EM_ST9PLUS     =  67,
    /** STMicroelectronics ST7 8-bit microcontroller */
    EM_ST7     =  68,
    /** Motorola MC68HC16 Microcontroller */
    EM_68HC16     =  69,
    /** Motorola MC68HC11 Microcontroller */
    EM_68HC11     =  70,
    /** Motorola MC68HC08 Microcontroller */
    EM_68HC08     =  71,
    /** Motorola MC68HC05 Microcontroller */
    EM_68HC05     =  72,
    /** Silicon Graphics SVx */
    EM_SVX     =  73,
    /** STMicroelectronics ST19 8-bit cpu */
    EM_ST19     =  74,
    /** Digital VAX */
    EM_VAX     =  75,
    /** Axis Communications 32-bit embedded processor */
    EM_CRIS     =  76,
    /** Infineon Technologies 32-bit embedded cpu */
    EM_JAVELIN     =  77,
    /** Element 14 64-bit DSP processor */
    EM_FIREPATH     =  78,
    /** LSI Logic's 16-bit DSP processor */
    EM_ZSP     =  79,
    /** Donald Knuth's educational 64-bit processor */
    EM_MMIX     =  80,
    /** Harvard's machine-independent format */
    EM_HUANY     =  81,
    /** SiTera Prism */
    EM_PRISM     =  82,
    /** Atmel AVR 8-bit microcontroller */
    EM_AVR     =  83,
    /** Fujitsu FR30 */
    EM_FR30     =  84,
    /** Mitsubishi D10V */
    EM_D10V     =  85,
    /** Mitsubishi D30V */
    EM_D30V     =  86,
    /** NEC v850 */
    EM_V850     =  87,
    /** Renesas M32R (formerly Mitsubishi M32R) */
    EM_M32R     =  88,
    /** Matsushita MN10300 */
    EM_MN10300     =  89,
    /** Matsushita MN10200 */
    EM_MN10200     =  90,
    /** picoJava */
    EM_PJ     =  91,
    /** OpenRISC 32-bit embedded processor */
    EM_OPENRISC     =  92,
    /** ARC Cores Tangent-A5 */
    EM_ARC_A5     =  93,
    /** Tensilica Xtensa Architecture */
    EM_XTENSA     =  94,
    /** Alphamosaic VideoCore processor */
    EM_VIDEOCORE     =  95,
    /** Thompson Multimedia General Purpose Processor */
    EM_TMM_GPP     =  96,
    /** National Semiconductor 32000 series */
    EM_NS32K     =  97,
    /** Tenor Network TPC processor */
    EM_TPC     =  98,
    /** Trebia SNP 1000 processor */
    EM_SNP1K     =  99,
    /** STMicroelectronics ST200 microcontroller */
    EM_ST200     = 100,
    /** Ubicom IP2022 micro controller */
    EM_IP2K     = 101,
    /** MAX Processor */
    EM_MAX     = 102,
    /** National Semiconductor CompactRISC */
    EM_CR     = 103,
    /** Fujitsu F2MC16 */
    EM_F2MC16     = 104,
    /** TI msp430 micro controller */
    EM_MSP430     = 105,
    /** ADI Blackfin */
    EM_BLACKFIN     = 106,
    /** S1C33 Family of Seiko Epson processors */
    EM_SE_C33     = 107,
    /** Sharp embedded microprocessor */
    EM_SEP     = 108,
    /** Arca RISC Microprocessor */
    EM_ARCA     = 109,
    /** Microprocessor series from PKU-Unity Ltd. and MPRC of Peking University */
    EM_UNICORE     = 110,
    /** eXcess: 16/32/64-bit configurable embedded CPU */
    EM_EXCESS     = 111,
    /** Icera Semiconductor Inc. Deep Execution Processor */
    EM_DXP     = 112,
    /** Altera Nios II soft-core processor */
    EM_ALTERA_NIOS2     = 113,
    /** National Semiconductor CRX */
    EM_CRX     = 114,
    /** Motorola XGATE embedded processor */
    EM_XGATE     = 115,
    /** Infineon C16x/XC16x processor */
    EM_C166     = 116,
    /** Renesas M16C series microprocessors */
    EM_M16C     = 117,
    /** Microchip Technology dsPIC30F Digital Signal Controller */
    EM_DSPIC30F     = 118,
    /** Freescale Communication Engine RISC core */
    EM_CE     = 119,
    /** Renesas M32C series microprocessors */
    EM_M32C     = 120,
    /** Reserved */
    EM_res121     = 121,
    /** Reserved */
    EM_res122     = 122,
    /** Reserved */
    EM_res123     = 123,
    /** Reserved */
    EM_res124     = 124,
    /** Reserved */
    EM_res125     = 125,
    /** Reserved */
    EM_res126     = 126,
    /** Reserved */
    EM_res127     = 127,
    /** Reserved */
    EM_res128     = 128,
    /** Reserved */
    EM_res129     = 129,
    /** Reserved */
    EM_res130     = 130,
    /** Altium TSK3000 core */
    EM_TSK3000     = 131,
    /** Freescale RS08 embedded processor */
    EM_RS08     = 132,
    /** Reserved */
    EM_res133     = 133,
    /** Cyan Technology eCOG2 microprocessor */
    EM_ECOG2     = 134,
    /** Sunplus Score */
    EM_SCORE     = 135,
    ///** Sunplus S+core7 RISC processor */
    //EM_SCORE7     = 135,
    /** New Japan Radio (NJR) 24-bit DSP Processor */
    EM_DSP24     = 136,
    /** Broadcom VideoCore III processor */
    EM_VIDEOCORE3     = 137,
    /** RISC processor for Lattice FPGA architecture */
    EM_LATTICEMICO32 =138,
    /** Seiko Epson C17 family */
    EM_SE_C17     = 139,
    /** Reserved */
    EM_res140     = 140,
    /** Reserved */
    EM_res141     = 141,
    /** Reserved */
    EM_res142     = 142,
    /** Reserved */
    EM_res143     = 143,
    /** Reserved */
    EM_res144     = 144,
    /** Reserved */
    EM_res145     = 145,
    /** Reserved */
    EM_res146     = 146,
    /** Reserved */
    EM_res147     = 147,
    /** Reserved */
    EM_res148     = 148,
    /** Reserved */
    EM_res149     = 149,
    /** Reserved */
    EM_res150     = 150,
    /** Reserved */
    EM_res151     = 151,
    /** Reserved */
    EM_res152     = 152,
    /** Reserved */
    EM_res153     = 153,
    /** Reserved */
    EM_res154     = 154,
    /** Reserved */
    EM_res155     = 155,
    /** Reserved */
    EM_res156     = 156,
    /** Reserved */
    EM_res157     = 157,
    /** Reserved */
    EM_res158     = 158,
    /** Reserved */
    EM_res159     = 159,
    /** STMicroelectronics 64bit VLIW Data Signal Processor */
    EM_MMDSP_PLUS     = 160,
    /** Cypress M8C microprocessor */
    EM_CYPRESS_M8C     = 161,
    /** Renesas R32C series microprocessors */
    EM_R32C     = 162,
    /** NXP Semiconductors TriMedia architecture family */
    EM_TRIMEDIA     = 163,
    /** QUALCOMM DSP6 Processor */
    EM_QDSP6     = 164,
    /** Intel 8051 and variants */
    EM_8051     = 165,
    /** STMicroelectronics STxP7x family */
    EM_STXP7X     = 166,
    /** Andes Technology compact code size embedded RISC processor family */
    EM_NDS32     = 167,
    /** Cyan Technology eCOG1X family */
    EM_ECOG1     = 168,
    ///** Cyan Technology eCOG1X family */
    //EM_ECOG1X     = 168,
    /** Dallas Semiconductor MAXQ30 Core Micro-controllers */
    EM_MAXQ30     = 169,
    /** New Japan Radio (NJR) 16-bit DSP Processor */
    EM_XIMO16     = 170,
    /** M2000 Reconfigurable RISC Microprocessor */
    EM_MANIK     = 171,
    /** Cray Inc. NV2 vector architecture */
    EM_CRAYNV2     = 172,
    /** Renesas RX family */
    EM_RX     = 173,
    /** Imagination Technologies META processor architecture */
    EM_METAG     = 174,
    /** MCST Elbrus general purpose hardware architecture */
    EM_MCST_ELBRUS     = 175,
    /** Cyan Technology eCOG16 family */
    EM_ECOG16     = 176,
    /** National Semiconductor CompactRISC 16-bit processor */
    EM_CR16     = 177,
    /** Freescale Extended Time Processing Unit */
    EM_ETPU     = 178,
    /** Infineon Technologies SLE9X core */
    EM_SLE9X     = 179,
    /** Intel L1OM */
    EM_L1OM     = 180,
    /** Reserved by Intel */
    EM_INTEL181     = 181,
    /** Reserved by Intel */
    EM_INTEL182     = 182,
    /** Reserved by ARM */
    EM_res183     = 183,
    /** Reserved by ARM */
    EM_res184     = 184,
    /** Atmel Corporation 32-bit microprocessor family */
    EM_AVR32     = 185,
    /** STMicroeletronics STM8 8-bit microcontroller */
    EM_STM8     = 186,
/** Tilera TILE64 multicore architecture family */
    EM_TILE64     = 187,
    /** Tilera TILEPro multicore architecture family */
    EM_TILEPRO     = 188,
    /** Xilinx MicroBlaze 32-bit RISC soft processor core */
    EM_MICROBLAZE     = 189,
}

impl TryFrom<u16> for ElfMachine {
    type Error = ();
    fn try_from(n: u16) -> Result<Self, Self::Error> {
        match num_traits::FromPrimitive::from_u16(n){
            Option::Some(val) => Result::Ok(val),
            Option::None => Result::Err(())
        }
    }
}

impl Into<u16> for ElfMachine{
    fn into(self) -> u16 {
        num_traits::ToPrimitive::to_u16(&self).unwrap()
    }
}
//----------------------------------------------------------------------