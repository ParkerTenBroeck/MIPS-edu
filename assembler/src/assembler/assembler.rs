use std::{fs::File, error::Error, collections::{HashMap, LinkedList}, rc::Rc, cell::RefCell, io::Read, ops::{Deref}};


pub enum Define{
    Replacement(Vec<PPToken>),
    Macro(()),
    Label(String),
    Nothing,
}

#[derive(Default)]
struct Scope{
    values: HashMap<String, Define>,
}

impl Scope {
    fn new() -> Self {
        Scope{
            ..Default::default()
        }
    }
}

//-------------------------------------------------------------------------------------------
#[derive(Debug)]
struct Report{
    level: ReportLevel,
    r#type: ReportType,
    message: String,
    cause_area: Option<PPArea>,
}
#[derive(Debug)]
enum ReportType{
    Tokenizer,
    PreProcessor,
    Assembler,
    Linker,
    OS
}

#[derive(Debug)]
enum ReportLevel{
    Message,
    Warning,
    Error,
}

impl Report{
    fn tokenizer_error(error: TokenizerError) -> Self {
        let TokenizerError{
            error,
            part
        } = error;
        Report{
            r#type: ReportType::Tokenizer,
            message: error,
            cause_area: match part{
                Some(part) => Option::Some(PPArea::from_t_data(part)),
                None => Option::None,
            },
            level: ReportLevel::Error,
        }
    }

    fn preprocessor_error(message: String) -> Self{
        Report{
            r#type: ReportType::PreProcessor,
            message,
            cause_area: Option::None,
            level: ReportLevel::Error,
        }
    }
    fn preprocessor_error_in_area(message: String, area: impl Into<PPArea>) -> Self{
        Report{
            r#type: ReportType::PreProcessor,
            message,
            cause_area: Option::Some(area.into()),
            level: ReportLevel::Error,
        }
    }

    fn assembler_error_in_area(message: String, area: impl Into<PPArea>) -> Self{
        Report{
            r#type: ReportType::Assembler,
            message,
            cause_area: Option::Some(area.into()),
            level: ReportLevel::Error,
        }
    }
    fn os_error(message: String) -> Self{
        Report{
            r#type: ReportType::OS,
            message,
            cause_area: Option::None,
            level: ReportLevel::Error,
        }
    }

    fn to_string(&self, assembler: &AssemblerState) -> String {
        if let Option::Some(area) = &self.cause_area{

            fn generate_message(assembler: &AssemblerState, area: TokenData) -> String{
                let area_str;
                if let Option::Some(file) = area.file{
                    let file = assembler.get_file(file as usize);
                    area_str = format!("\n    --> {}:{}:{}\n     |\n{: <5}|\t{}\n     |", file.as_ref().file, area.line + 1, area.column + 1, area.line + 1, area.str_from_token(file.as_ref().data.as_str()));
                }else{
                    area_str = format!("(line: {}, column: {})", area.line + 1, area.column);
                }
                area_str
            }

            fn generate_message_2(assembler: &AssemblerState, area: TokenData) -> String{
                let area_str;
                if let Option::Some(file) = area.file{
                    let file = assembler.get_file(file as usize);
                    area_str = format!("\n    ::: {}:{}:{}\n     |\n{: <5}|\t{}\n     |", file.as_ref().file, area.line + 1, area.column + 1, area.line + 1, area.str_from_token(file.as_ref().data.as_str()));
                }else{
                    area_str = format!("(line: {}, column: {})", area.line + 1, area.column);
                }
                area_str
            }
            let mut msg = generate_message(assembler, area.area);
            let mut parent = &area.parent;
            while let Option::Some(p) = parent{
                msg = format!("{}{}", msg, generate_message_2(assembler, p.area));
                parent = &p.parent;
            }

            format!("{:?} {:?}: {} {}\n",self.r#type, self.level, self.message, msg)
        }else{
            format!("{:?}: {}",self.r#type, self.message)
        }
    }
}
//-------------------------------------------------------------------------------------------

pub struct AssemblerReport{
    state: Rc<RefCell<AssemblerState>>,
}

impl AssemblerReport{
    fn new(assembler: &mut Assembler) -> Self{
        Self{
            state: assembler.asm_state.clone(),
        }
    }
}

impl std::fmt::Display for AssemblerReport{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        let state = self.state.deref().borrow();
        let assembler = state.deref();
        for report in &assembler.errors{
            string = format!("{}\n{}", string, report.to_string(assembler))
        }
        write!(f, "{}", string)
    }
}

//-------------------------------------------------------------------------------------------

pub struct FileInfo{
    pub file: String,
    pub data: String,
}

//------------------------------------------------------------------------
pub struct AssemblerState{
    cur_addr: u32,
    curr_sec: u16,
    symbols: HashMap<String, Symbol>,
    errors: LinkedList<Report>,
    files: LinkedList<Rc<FileInfo>>,
    scope: Vec<Scope>,
}

impl AssemblerState{
    fn new() -> Self{
        Self{
            scope: Vec::new(),
            cur_addr: 0,
            curr_sec: 0,
            errors: LinkedList::new(),
            files: LinkedList::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn report_tokenizer_error(&mut self, error: TokenizerError){
        self.errors.push_back(Report::tokenizer_error(error));
    }


    pub fn report_preprocessor_error(&mut self, error: impl Into<String>, area: PPArea){
        self.errors.push_back(Report::preprocessor_error_in_area(error.into(), area));
    }

    pub fn report_assembler_error(&mut self, error: impl Into<String>, area: PPArea){
        self.errors.push_back(Report::preprocessor_error_in_area(error.into(), area));
    }

    pub fn report_os_error(&mut self, error: impl Into<String>){
        self.errors.push_back(Report::os_error(error.into()))
    }
    pub fn has_encountered_error(&self) -> bool{
        self.errors.iter().any(|x|{
            match x.level {
                ReportLevel::Error => true,
                _ => false,
            }
        })
    }

    pub fn get_from_scope<'b>(&'b mut self, ident: &String) -> Option<&'b mut Define>{
        

        for scope in self.scope.iter_mut().rev(){
            match scope.values.get_mut(ident){
                Some(val) => {
                    return Option::Some(val);
                },
                None => continue,
            }
        }

        Option::None
    }

    pub fn put_into_scope(&mut self, ident: String, val: Define) {
        let mut len = self.scope.len();
        if len == 0 {
            self.scope.push(Scope::new());
            len = 1;
        }
        self.scope[len - 1].values.insert(ident, val);
    }

    pub fn add_file(&mut self, file: String) -> Result<(usize, Rc<FileInfo>), Box<dyn Error>>{
        
        let mut input_buf = String::new();
        let _size = File::open(&file)?.read_to_string(&mut input_buf)?;
    
        let rc = Rc::new(FileInfo{
            data: input_buf,
            file: file
        });

        self.files.push_back(rc);
        Result::Ok((self.files.len(), self.files.back().unwrap().clone()))
    }

    pub fn get_file(&self, file_id: usize) -> Rc<FileInfo>{
        self.files.iter().nth(file_id - 1).unwrap().clone()
    }
}

//------------------------------------------------------------------------

pub struct Assembler{
    asm_state: Rc<RefCell<AssemblerState>>,
}

//errors
use util::token::{TokenizerError, TokenData};

use super::{symbol::Symbol, preprocessor::{PreProcessedLine, PPToken, PreProcessor, PPArea}};


impl Assembler {

    pub fn new() -> Self{
        Assembler {
            asm_state: Rc::new(RefCell::new(AssemblerState::new())),
        }
    }



    pub fn clone_asm_state(&mut self) -> Rc<RefCell<AssemblerState>>{
        self.asm_state.clone()
    }

    pub fn asm_state(&mut self) -> std::cell::RefMut<AssemblerState>{
        self.asm_state.borrow_mut()
    }

    #[allow(dead_code)]
    #[allow(unused)]
    pub fn assemble(&mut self, input: String, output:&mut  File) -> Result<AssemblerReport, AssemblerReport>{
        
        //let test = memmap::Mmap::map(output)?;
        let pre_processor = match PreProcessor::new(self, input){
            Ok(pp) => pp,
            Err(err) => {
                return Result::Err(AssemblerReport::new(self));
            },
        };

        for line in pre_processor{
            self.assemble_line(line);
        }

        if self.asm_state().has_encountered_error(){
            Result::Err(AssemblerReport::new(self))
        }else{
            Result::Ok(AssemblerReport::new(self))
        }
    }

    fn assemble_line(&mut self, line: PreProcessedLine){
        match line{
            PreProcessedLine::Label(label, _token) => {
                let mut state = self.asm_state();
                let sym = Symbol{
                    name: label.clone(),
                    value: state.cur_addr,
                    size: 0,
                    section_index: state.curr_sec,
                    ..Default::default()
                };
                state.symbols.insert(label, sym);
            }
        }
    }
}