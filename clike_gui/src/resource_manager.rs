use std::{fs::File, path::Path};

pub fn get_log_file() -> File{
    let path = Path::new("log.txt");
    let mut file = File::open(path);
    match &mut file {
        Ok(file) => {
            let meta = file.metadata().unwrap();
            let created = meta.created().unwrap();
            
            let created = created.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis();
            let _created = format!("{}", created);
        },
        Err(_) => {},
    }
    File::create(path).unwrap()
}