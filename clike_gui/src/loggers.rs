use std::{collections::LinkedList, sync::Mutex, io::{Write}};



pub type Record = (log::Level, String);
type Records = LinkedList<Record>;

#[derive(Default)]
struct LogData{
    records: Records,
    log_file: Option<std::fs::File>,
    sequence: usize,
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
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {

            let start = std::time::SystemTime::now();
            let since_the_epoch = start
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            
            //logging to console
            println!("{} - {}", record.level(), record.args());

            let mut data = get_logger_data().lock().unwrap();
                
            //logging to file
            {
                use std::io::*;
                let mut file = Option::None::<std::fs::File>;
                std::mem::swap(&mut file, &mut data.log_file);
                match &mut file{
                    Some(file) => {

                        //record.file()
                        let mut buf = format!(
"{{
    millis: {},
    nanos: {},
    level: {},
    sequence: {},
    message: {:?},
    target: {},
",
since_the_epoch.as_millis(),
since_the_epoch.as_micros(),
record.level(),
data.sequence,
record.args(),
record.target());
                        if let Option::Some(file) = record.file()
                        {
                            buf.push_str("\tfile: ");
                            buf.push_str(file);
                            buf.push_str(",\n");
                        }
                        if let Option::Some(file_static) = record.file_static()
                        {
                            buf.push_str("\tfile_static: ");
                            buf.push_str(file_static);
                            buf.push_str(",\n");
                        }
                        if let Option::Some(module_path) = record.module_path()
                        {
                            buf.push_str("\tmodule_path: ");
                            buf.push_str(module_path);
                            buf.push_str(",\n");
                        }
                        if let Option::Some(module_path_static) = record.module_path_static()
                        {
                            buf.push_str("\tmodule_path_static: ");
                            buf.push_str(module_path_static);
                            buf.push_str(",\n");
                        }
                        if let Option::Some(line) = record.line()
                        {
                            buf.push_str(format!("\tline: {},\n", line).as_str());
                        }
                        buf.push_str("}\n");
                        let res = file.write_all(buf.as_bytes());
                        match res{
                            Ok(_) => {}
                            Err(_err) => {

                            }
                        }
                    }
                    None => {}
                }
                std::mem::swap(&mut file, &mut data.log_file);
            }

            //logging to records
            {
                data.records.push_front((record.level(), format!("{}", record.args())));
                if data.records.len() > 100{
                    data.records.pop_back();
                }    
            }
            data.sequence += 1;
        }
    }

    fn flush(&self) {

        let mut data = get_logger_data().lock().unwrap();
        match &mut data.log_file{
            Some(file) => {
                let _ = file.flush();
            },
            None => {},
        }
    }
}


pub fn init_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn init() -> bool{
    let mut data = get_logger_data().lock().unwrap();
    
    let _ = std::fs::create_dir("./log/");
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
    drop(data);
    log::info!("Initialized log");
    true
}
