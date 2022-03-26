use std::{collections::LinkedList, sync::Mutex};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static SIMPLE_LOGGER: SimpleLogger = SimpleLogger;

pub fn init_simple_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&SIMPLE_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
}
struct RecordLogger;

use std::{mem::MaybeUninit};


pub type Record = (log::Level, String);
type Records = Mutex<LinkedList<Record>>;
#[inline(never)]
fn record_log_list() -> &'static Records {

    use std::sync::{Once};
    // Create an uninitialized static
    static ONCE: Once = Once::new();
    static mut SINGLETON: MaybeUninit<Records> = MaybeUninit::uninit();

    unsafe {
        ONCE.call_once(|| {
            let singleton = Records::default();
            SINGLETON.write(singleton);
        });
        SINGLETON.assume_init_ref()
    }
}

pub fn get_last_record(level: log::Level, num: u32) -> LinkedList<Record>{
    let mut list = LinkedList::new();
    let mut i = 0;
    let test = record_log_list().lock().unwrap();
    for record in test.iter()
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

impl log::Log for RecordLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let mut test = record_log_list().lock().unwrap();
            test.push_front((record.level(), format!("{}", record.args())));
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static RECORD_LOGGER: RecordLogger = RecordLogger;


pub fn init_record_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&RECORD_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn init(){
    let _ = init_record_logger();
}
