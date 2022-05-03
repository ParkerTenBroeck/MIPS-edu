use self::header::ElfHeader;

pub mod header;
pub mod program;
pub mod section;





pub struct InternalElf{
    header: ElfHeader
}