use std::{collections::LinkedList, sync::Mutex, io::Read, borrow::BorrowMut};



pub type Record = (log::Level, String);
type Records = LinkedList<Record>;

#[derive(Default)]
struct LogData{
    records: Records,
    log_file: Option<std::fs::File>,
}

fn get_logger_data() -> &'static Mutex<LogData> {

    use std::{mem::MaybeUninit};

    use std::sync::{Once};
    // Create an uninitialized static
    static ONCE: Once = Once::new();
    static mut SINGLETON: MaybeUninit<Mutex<LogData>> = MaybeUninit::uninit();

    unsafe {
        ONCE.call_once(|| {
            let singleton = Mutex::new(LogData::default());
            SINGLETON.write(singleton);
        });
        SINGLETON.assume_init_ref()
    }
}

pub fn get_last_record(level: log::Level, num: u32) -> LinkedList<Record>{
    let mut list = LinkedList::new();
    let mut i = 0;
    let test = get_logger_data().lock().unwrap();
    for record in test.records.iter()
    .filter(|t1| {
        t1.0.lt(&level)
    })
    {
        list.push_back(record.clone());
        if i >= num{
            break;
        }
        i += 1;
    }
    list
}


struct Logger;

static LOGGER: Logger = Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            
            //logging to console
            println!("{} - {}", record.level(), record.args());

            let mut data = get_logger_data().lock().unwrap();
                
            //logging to file
            {
                use std::io::*;
                match &mut data.log_file{
                    Some(file) => {

                        file.write_all(b"");
                        
                    }
                    None => {}
                }
            }

            //logging to records
            {
                data.records.push_front((record.level(), format!("{}", record.args())));
                if data.records.len() > 100{
                    data.records.pop_back();
                }    
            }
        }
    }

    fn flush(&self) {

    }
}


pub fn init_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn init() -> bool{
    let mut data = get_logger_data().lock().unwrap();
    
    data.log_file = match std::fs::File::create("./log/log.txt"){
        Ok(val) => Option::Some(val),
        Err(err) => {
            eprintln!("failed to create log file: {}", err);  
            return false;
        },
    };
    match init_logger(){
        Ok(_) => {},
        Err(err) => {
          eprintln!("failed to initialize logger: {}", err);  
          return false;
        },
    }
    log::info!("Initialized log");
    true
}
