//------------------------------------------------------------------------

pub struct Symbol {
    pub name: String,
    pub value: u32,
    pub size: u32,
    pub section_index: u16,
    pub s_type: SymType,
    pub binding: SymBind,
    pub vis: SymVis,
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            name: Default::default(),
            value: Default::default(),
            size: Default::default(),
            section_index: Default::default(),
            s_type: SymType::STT_NOTYPE,
            binding: SymBind::STB_LOCAL,
            vis: SymVis::STV_DEFUALT,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum SymBind {
    STB_LOCAL,
    STB_GLOBAL,
    STB_WEAK,
    STB_LOOS,
    STB_HIOS,
    STB_LOPROC,
    STB_HIPROC,
}

#[allow(non_camel_case_types)]
pub enum SymType {
    STT_NOTYPE,
    STT_OBJECT,
    STT_FUNC,
    STT_SECTION,
    STT_FILE,
    STT_COMMON,
    STT_TLS,
    STT_LOOS,
    STT_HIOS,
    STT_LOPROC,
    STT_SPARC_REGISTER,
    STT_HIPROC,
}

#[allow(non_camel_case_types)]
pub enum SymVis {
    STV_DEFUALT,
    STV_INTERNAL,
    STV_HIDDNE,
    STV_PROTECTED,
    STV_EXPORTED,
    STV_SINGLETON,
    STV_ELIMINATE,
}
