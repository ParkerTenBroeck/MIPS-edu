use self::header::ElfHeader;

pub mod header;
pub mod program;
pub mod section;

pub struct InternalElf {
    pub(crate) _header: ElfHeader,
}

// pub fn ELF_TBSS_SPECIAL(sectionHeader: InternalSectionHeader, segment: InternalProgramHeader) -> bool{
//     ((sectionHeader.flags() & SectionFlags::SHF_TLS).bits() != 0)
//     && sectionHeader.sh_type() == SectionType::SHT_NOBITS
//     && segment.p_type != ProgramHeaderType::PT_TLS
// }

// pub fn ELF_SECTION_IN_SEGMENT_STRICT(section: InternalSectionHeader, segment: InternalProgramHeader) -> bool{
//     return ELF_SECTION_IN_SEGMENT(section, segment, true, true);
// }

// pub fn ELF_SECTION_IN_SEGMENT_CHECK_VMA(section: InternalSectionHeader, segment: InternalProgramHeader) -> bool{
//     ELF_SECTION_IN_SEGMENT(section, segment, true, false)
// }

// pub fn ELF_SECTION_SIZE(section: InternalSectionHeader, segment: InternalProgramHeader) -> u128{
//     return if ELF_TBSS_SPECIAL(section, segment) {0} else {section.sh_size} ;
// }

// pub fn ELF_SECTION_IN_SEGMENT(section: InternalSectionHeader, segment: InternalProgramHeader, check_vma: bool, strict: bool) -> bool{
//     return ((
//             /* Only PT_LOAD, PT_GNU_RELRO and PT_TLS segments can contain	\
//             SHF_TLS sections.  */
//             (((section.flags().bits() & SectionFlags::SHF_TLS.bits()) != 0)
//             && (segment.p_type == ProgramHeaderType::PT_TLS
//             || segment.p_type == ProgramHeaderType::PT_GNU_RELRO
//             || segment.p_type == ProgramHeaderType::PT_GNU_RELRO))
//             /* PT_TLS segment contains only SHF_TLS sections, PT_PHDR no	\
//             sections at all.  */
//             || ((section.flags() & SectionFlags::SHF_TLS).bits() == 0
//             && segment.p_type != ProgramHeaderType::PT_TLS
//             && segment.p_type != ProgramHeaderType::PT_PHDR))
//             /* PT_LOAD and similar segments only have SHF_ALLOC sections.  */
//             && !((section.flags() & SectionFlags::SHF_ALLOC).bits() == 0
//             && (segment.p_type == ProgramHeaderType::PT_LOAD
//                 || segment.p_type == ProgramHeaderType::PT_DYNAMIC
//                 || segment.p_type == ProgramHeaderType::PT_GNU_EH_FRAME
//                 || segment.p_type == ProgramHeaderType::PT_GNU_STACK
//                 || segment.p_type == ProgramHeaderType::PT_GNU_RELRO
//                 || (segment.p_type.to_u32() >= ProgramHeaderType::PT_GNU_MBIND_LO.to_u32()
//                 && segment.p_type.to_u32() <= ProgramHeaderType::PT_GNU_MBIND_HI.to_u32())))
//            /* Any section besides one of type SHT_NOBITS must have file		\
//               offsets within the segment.  */
//            && (section.sh_type() == SectionType::SHT_NOBITS
//                || ( section.sh_offset >= segment.p_offset
//                && (!(strict)
//                    || (section.sh_offset - segment.p_offset
//                    <= segment.p_filesz - 1))
//                && ((section.sh_offset - segment.p_offset
//                 + ELF_SECTION_SIZE(section, segment))
//                    <= segment.p_filesz)))
//                 /* SHF_ALLOC sections must have VMAs within the segment.  */
//            && (!(check_vma)
//                || (section.flags() & SectionFlags::SHF_ALLOC).bits() == 0
//                || (section.sh_addr >= segment.p_vaddr
//                && (!(strict)
//                    || (section.sh_addr - segment.p_vaddr
//                    <= segment.memory_size - 1))
//                && ((section.sh_addr - segment.p_vaddr
//                 + ELF_SECTION_SIZE(section, segment))
//                    <= segment.p_memsz)))
//            /* No zero size sections at start or end of PT_DYNAMIC nor		\
//               PT_NOTE.  */
//            && ((segment.p_type != ProgramHeaderType::PT_DYNAMIC
//             && segment.p_type != ProgramHeaderType::PT_NOTE)
//                || section.sh_size != 0
//                || segment.p_memsz == 0
//                || ((section.sh_type() == SectionType::SHT_NOBITS
//                 || ( section.sh_offset > segment.p_offset
//                     && (section.sh_offset - segment.p_offset
//                     < segment.p_filesz)))
//                && ((section.flags() & SectionType::SHF_ALLOC).bits() == 0
//                    || (section.sh_addr > segment.p_vaddr
//                    && (section.sh_addr - segment.p_vaddr
//                        < segment.p_memsz))))));
// }
