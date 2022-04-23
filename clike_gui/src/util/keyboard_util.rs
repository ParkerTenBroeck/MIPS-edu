use eframe::egui;

// -------------------------------------------------------------------

pub struct KeyboardMemory{
    pressed: Vec<Key>
}

#[derive(Debug)]
struct Key{
    key_code: char,
    consumed: bool,
    shift: bool,
    alt: bool,
    command: bool,
}

impl Key{
    fn new(char: char, mods: &egui::Modifiers) -> Self{
        Self{
            key_code: char,
            consumed: false,
            shift: mods.shift,
            alt: mods.alt,
            command: mods.command,
        }
    }
}

impl PartialEq for Key{
    fn eq(&self, other: &Self) -> bool {
        self.key_code == other.key_code && self.consumed == other.consumed && self.shift == other.shift && self.alt == other.alt && self.command == other.command
    }
}

impl KeyboardMemory{
    pub fn new() -> Self{
        Self{
            pressed: Vec::new()
        }
    }

    pub fn is_pressed(&self, key_code: char) -> bool{
        self.pressed.iter().any(|x|{
            x.key_code == key_code
        })
    }

    pub fn consume(&mut self, key_code: char) -> bool{
        self.pressed.iter_mut().any(|x|{
            if x.key_code == key_code{
                if !x.consumed{
                    x.consumed = true;
                    return true;
                }
            }
            false
        })
    }

    pub fn update(&mut self, context: &egui::Context){
        let input = context.input();
        for event in input.events.iter(){
            match event{
                egui::Event::Key { key, pressed, modifiers } => {
                    let key = match key{
                        egui::Key::ArrowDown => 38u8 as char,
                        egui::Key::ArrowLeft => 37u8 as char,
                        egui::Key::ArrowRight => 39u8 as char,
                        egui::Key::ArrowUp => 38u8 as char,
                        egui::Key::Escape => 8u8 as char,
                        egui::Key::Tab => '\t',
                        egui::Key::Backspace => '\x08',
                        egui::Key::Enter => '\n',
                        egui::Key::Insert => 45u8 as char,
                        egui::Key::Delete => 46u8 as char,
                        egui::Key::Home => 36u8 as char,
                        egui::Key::End => 35u8 as char,
                        egui::Key::PageUp => 33u8 as char,
                        egui::Key::PageDown => 34u8 as char,
                        egui::Key::Space => ' ',
                        egui::Key::Num0 => '0',
                        egui::Key::Num1 => '1',
                        egui::Key::Num2 => '2',
                        egui::Key::Num3 => '3',
                        egui::Key::Num4 => '4',
                        egui::Key::Num5 => '5',
                        egui::Key::Num6 => '6',
                        egui::Key::Num7 => '7',
                        egui::Key::Num8 => '8',
                        egui::Key::Num9 => '9',
                        egui::Key::A => 'A',
                        egui::Key::B => 'B',
                        egui::Key::C => 'C',
                        egui::Key::D => 'D',
                        egui::Key::E => 'E',
                        egui::Key::F => 'F',
                        egui::Key::G => 'G',
                        egui::Key::H => 'H',
                        egui::Key::I => 'I',
                        egui::Key::J => 'J',
                        egui::Key::K => 'K',
                        egui::Key::L => 'L',
                        egui::Key::M => 'M',
                        egui::Key::N => 'N',
                        egui::Key::O => 'O',
                        egui::Key::P => 'P',
                        egui::Key::Q => 'Q',
                        egui::Key::R => 'R',
                        egui::Key::S => 'S',
                        egui::Key::T => 'T',
                        egui::Key::U => 'U',
                        egui::Key::V => 'V',
                        egui::Key::W => 'W',
                        egui::Key::X => 'X',
                        egui::Key::Y => 'Y',
                        egui::Key::Z => 'Z',
                    };
                    if *pressed{
                        self.pressed.retain(|x| {x.key_code != key});
                        self.pressed.push(Key::new(key, modifiers));
                    }else{
                        self.pressed.retain(|x| {x.key_code != key});
                    }
                },
                _ => {}
            }
        }
    }
}


// -------------------------------------------------------------------
