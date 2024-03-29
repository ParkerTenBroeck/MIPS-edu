use crate::platform::sync::PlatSpecificLocking;
use std::{collections::LinkedList, io::Write, sync::Mutex};

pub type Record = (log::Level, String);
type Records = LinkedList<Record>;

#[derive(Default)]
struct LogData {
    records: Records,
    log_file: Option<std::fs::File>,
    sequence: usize,
}

fn get_logger_data() -> &'static Mutex<LogData> {
    use std::mem::MaybeUninit;

    use std::sync::Once;
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

pub fn get_last_record(level: log::Level, num: usize) -> LinkedList<Record> {
    let mut list = LinkedList::new();
    let test = get_logger_data().plat_lock().unwrap();
    for (i, record) in test.records.iter().filter(|t1| t1.0.lt(&level)).enumerate() {
        list.push_back(record.clone());
        if i >= num {
            break;
        }
    }
    list
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn info(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn trace(s: &str);
}
struct Logger;

static LOGGER: Logger = Logger;

fn full_msg(record: &log::Record<'_>, data: &LogData) -> String {
    let time_since_epoch = crate::platform::time::duration_since_epoch();

    let mut buf = format!(
        "{{
millis: {},
nanos: {},
level: {},
sequence: {},
message: {:?},
target: {},
",
        time_since_epoch.as_millis(),
        time_since_epoch.as_micros(),
        record.level(),
        data.sequence,
        record.args(),
        record.target()
    );

    if let Option::Some(file) = record.file() {
        buf.push_str("\tfile: ");
        buf.push_str(file);
        buf.push_str(",\n");
    }
    if let Option::Some(file_static) = record.file_static() {
        buf.push_str("\tfile_static: ");
        buf.push_str(file_static);
        buf.push_str(",\n");
    }
    if let Option::Some(module_path) = record.module_path() {
        buf.push_str("\tmodule_path: ");
        buf.push_str(module_path);
        buf.push_str(",\n");
    }
    if let Option::Some(module_path_static) = record.module_path_static() {
        buf.push_str("\tmodule_path_static: ");
        buf.push_str(module_path_static);
        buf.push_str(",\n");
    }
    if let Option::Some(line) = record.line() {
        buf.push_str(format!("\tline: {},\n", line).as_str());
    }
    buf.push_str("}\n");
    buf.to_string()
}

#[cfg(target_arch = "wasm32")]
fn full_msg_no_time(record: &log::Record<'_>, data: &LogData) -> String {
    //record.file()
    let mut buf = format!(
        "{{
level: {},
sequence: {},
message: {:?},
target: {},
",
        record.level(),
        data.sequence,
        record.args(),
        record.target()
    );
    if let Option::Some(file) = record.file() {
        buf.push_str("\tfile: ");
        buf.push_str(file);
        buf.push_str(",\n");
    }
    if let Option::Some(file_static) = record.file_static() {
        buf.push_str("\tfile_static: ");
        buf.push_str(file_static);
        buf.push_str(",\n");
    }
    if let Option::Some(module_path) = record.module_path() {
        buf.push_str("\tmodule_path: ");
        buf.push_str(module_path);
        buf.push_str(",\n");
    }
    if let Option::Some(module_path_static) = record.module_path_static() {
        buf.push_str("\tmodule_path_static: ");
        buf.push_str(module_path_static);
        buf.push_str(",\n");
    }
    if let Option::Some(line) = record.line() {
        buf.push_str(format!("\tline: {},\n", line).as_str());
    }
    buf.push_str("}\n");
    buf.to_string()
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true //metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let mut data = get_logger_data().plat_lock().unwrap();
            //logging to console
            #[cfg(target_arch = "wasm32")]
            match record.level() {
                log::Level::Error => error(full_msg_no_time(record, &*data).as_str()),
                log::Level::Warn => warn(full_msg_no_time(record, &*data).as_str()),
                log::Level::Info => {
                    info(format!("{} - {}", record.level(), record.args()).as_str())
                }
                log::Level::Debug => debug(full_msg_no_time(record, &*data).as_str()),
                log::Level::Trace => trace(full_msg_no_time(record, &*data).as_str()),
            }
            #[cfg(not(target_arch = "wasm32"))]
            match record.level() {
                log::Level::Error => {
                    eprintln!("{} - {}", record.level(), record.args());
                }
                _ => {
                    println!("{} - {}", record.level(), record.args());
                }
            }

            //logging to file
            {
                use std::io::*;
                let mut file = Option::None::<std::fs::File>;
                std::mem::swap(&mut file, &mut data.log_file);
                match &mut file {
                    Some(file) => {
                        let res = file.write_all(full_msg(record, &data).as_bytes());
                        match res {
                            Ok(_) => {}
                            Err(_err) => {}
                        }
                    }
                    None => {}
                }
                std::mem::swap(&mut file, &mut data.log_file);
            }

            //logging to records
            {
                data.records
                    .push_front((record.level(), format!("{}", record.args())));
                if data.records.len() > 100 {
                    data.records.pop_back();
                }
            }
            data.sequence += 1;
        }
    }

    fn flush(&self) {
        let mut data = get_logger_data().plat_lock().unwrap();
        match &mut data.log_file {
            Some(file) => {
                let _ = file.flush();
            }
            None => {}
        }
    }
}

pub fn init_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace))
}

pub fn init() -> bool {
    #[allow(unused_mut)]
    let mut data = get_logger_data().plat_lock().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::create_dir("./log/");
        data.log_file = match std::fs::File::create("./log/log.txt") {
            Ok(val) => Option::Some(val),
            Err(err) => {
                eprintln!("failed to create log file: {}", err);
                return false;
            }
        };
    }

    match init_logger() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("failed to initialize logger: {}", err);
            return false;
        }
    }
    drop(data);
    log::debug!("Initialized log");
    true
}
