use std::{fs::File, error::Error, collections::{HashMap, LinkedList}, rc::Rc, cell::RefCell, io::Read, ops::{Deref, DerefMut}};


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
#[derive(Debug)]
struct Report{
    level: ReportLevel,
    r#type: ReportType,
    message: String,
    cause_area: Option<TokenData>,
}
#[derive(Debug)]
enum ReportType{
    Tokenizer,
    PreProcessor,
    Assembler,
    Linker,
    File
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
            cause_area: part,
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
    fn preprocessor_error_in_area(message: String, area: TokenData) -> Self{
        Report{
            r#type: ReportType::PreProcessor,
            message,
            cause_area: Option::Some(area),
            level: ReportLevel::Error,
        }
    }

    fn assembler_error_in_area(message: String, area: TokenData) -> Self{
        Report{
            r#type: ReportType::Assembler,
            message,
            cause_area: Option::Some(area),
            level: ReportLevel::Error,
        }
    }

    fn to_string(&self, assembler: &AssemblerState) -> String {
        if let Option::Some(area) = self.cause_area{

            let area_str;
            if let Option::Some(file) = area.file{
                let file = assembler.get_file(file as usize);
                area_str = format!("\n   --> {}:{}:{}\n     |\n{: <5}|\t{}\n     |\n", file.as_ref().file, area.line + 1, area.column + 1, area.line + 1, area.str_from_token(file.as_ref().data.as_str()));
            }else{
                area_str = format!("(line: {}, column: {})", area.line + 1, area.column);
            }
            format!("{:?} {:?}: {} {}",self.r#type, self.level, self.message, area_str)
        }else{
            format!("{:?}: {}",self.r#type, self.message)
        }
    }
}


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


    pub fn report_preprocessor_error(&mut self, error: impl Into<String>, area: TokenData){
        self.errors.push_back(Report::preprocessor_error_in_area(error.into(), area));
    }

    pub fn report_assembler_error(&mut self, error: impl Into<String>, area: TokenData){
        self.errors.push_back(Report::preprocessor_error_in_area(error.into(), area));
    }

    pub fn has_encountered_error(&self) -> bool{
        self.errors.len() > 0
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

use super::{symbol::Symbol, preprocessor::{PreProcessedLine, PPToken, PreProcessor}};


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
    pub fn assemble(&mut self, input: String, output:&mut  File) -> Result<(), Box<dyn Error>>{
        
        //let test = memmap::Mmap::map(output)?;
        let pre_processor = PreProcessor::new(self, input)?;

        for line in pre_processor{
            match line{
                Ok(line) => {
                    println!("{:?}", line);
                    self.assemble_line(line);
                },
                Err(_) => {
                    //the pre processor should have reported the error already
                    //so we can continue to the next line if there is one
                },
            }
        }

        if self.asm_state().has_encountered_error(){
            let state = self.asm_state();
            for error in &state.errors{
                println!("{}", error.to_string(state.deref()));
            }
            Result::Err("Still havent made the assembler error report :)".into())
        }else{
            Result::Ok(())
        }
    }

    fn assemble_line(&mut self, line: PreProcessedLine){
        match line{
            PreProcessedLine::Label(label, token) => {
                let mut state = self.asm_state();
                let sym = Symbol{
                    name: label.clone(),
                    value: state.cur_addr,
                    size: 0,
                    section_index: state.curr_sec,
                    ..Default::default()
                };
                state.symbols.insert(label, sym);
            },
            _ => {
                self.asm_state().report_assembler_error("", line.get_area());
            }
        }
    }

    // fn accept_preprocess_statement(&mut self, line: &mut Vec<Token>) -> Option<Vec<PPToken>>{
    //     let first = line.remove(0);
    //     if let TokenType::PreProcessorStatement(ident) = first.t_type{
    //         match ident.as_str(){
    //             "define" => {
    //                 let second = line.remove(0);
    //                 if let TokenType::Identifier(ident) = second.t_type{
    //                     if line.len() > 0{
    //                         let mut pp_line = Vec::new();
    //                         while line.len() > 0{
    //                             let tok = line.remove(0);
    //                             pp_line.push(
    //                                 PPToken{
    //                                 tok,
    //                                 parent: Option::None,
    //                             });
    //                         }
    //                         self.put_into_scope(ident, Define::Replacement(pp_line));
    //                     }else{
    //                         self.put_into_scope(ident, Define::Nothing);
    //                     }
    //                 }else{
    //                     self.report_preprocessor_error("Only a valid identifier can be used in define", second.t_data);
    //                     return Option::None;
    //                 }
    //             },
    //             "undefine" => {

    //             },
    //             val => {
    //                 self.report_preprocessor_error(format!("Invalid preprocessing statement: {:?}", val), first.t_data);
    //                 return Option::None;
    //             }
    //         }
    //     }
    //     Option::None
    // }
}