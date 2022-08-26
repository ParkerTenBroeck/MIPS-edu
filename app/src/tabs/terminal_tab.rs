use std::str::Chars;

use eframe::{
    egui::{Sense, TextFormat, WidgetInfo, WidgetType},
    epaint::{
        self,
        text::{LayoutJob, TextWrapping},
        Color32, FontFamily, FontId, Rounding, Stroke,
    },
};
use egui_dock::Tab;

use crate::platform::time;

pub enum TerminalMode {
    Basic,
}
pub struct TerminalTab {
    data: String,
    #[allow(unused)]
    mode: TerminalMode,
    #[allow(unused)]
    cursor: Option<usize>,
}

const TEST_TEXT: &str =  "                 40m     41m     42m     43m     44m     45m     46m     47m
\tm \x1b[m  gYw   \x1b[m\x1b[40m  gYw  \x1b[0m \x1b[m\x1b[41m  gYw  \x1b[0m \x1b[m\x1b[42m  gYw  \x1b[0m \x1b[m\x1b[43m  gYw  \x1b[0m \x1b[m\x1b[44m  gYw  \x1b[0m \x1b[m\x1b[45m  gYw  \x1b[0m \x1b[m\x1b[46m  gYw  \x1b[0m \x1b[m\x1b[47m  gYw  \x1b[0m
   1m \x1b[1m  gYw   \x1b[1m\x1b[40m  gYw  \x1b[0m \x1b[1m\x1b[41m  gYw  \x1b[0m \x1b[1m\x1b[42m  gYw  \x1b[0m \x1b[1m\x1b[43m  gYw  \x1b[0m \x1b[1m\x1b[44m  gYw  \x1b[0m \x1b[1m\x1b[45m  gYw  \x1b[0m \x1b[1m\x1b[46m  gYw  \x1b[0m \x1b[1m\x1b[47m  gYw  \x1b[0m
  30m \x1b[30m  gYw   \x1b[30m\x1b[40m  gYw  \x1b[0m \x1b[30m\x1b[41m  gYw  \x1b[0m \x1b[30m\x1b[42m  gYw  \x1b[0m \x1b[30m\x1b[43m  gYw  \x1b[0m \x1b[30m\x1b[44m  gYw  \x1b[0m \x1b[30m\x1b[45m  gYw  \x1b[0m \x1b[30m\x1b[46m  gYw  \x1b[0m \x1b[30m\x1b[47m  gYw  \x1b[0m
1;30m \x1b[1;30m  gYw   \x1b[1;30m\x1b[40m  gYw  \x1b[0m \x1b[1;30m\x1b[41m  gYw  \x1b[0m \x1b[1;30m\x1b[42m  gYw  \x1b[0m \x1b[1;30m\x1b[43m  gYw  \x1b[0m \x1b[1;30m\x1b[44m  gYw  \x1b[0m \x1b[1;30m\x1b[45m  gYw  \x1b[0m \x1b[1;30m\x1b[46m  gYw  \x1b[0m \x1b[1;30m\x1b[47m  gYw  \x1b[0m
  31m \x1b[31m  gYw   \x1b[31m\x1b[40m  gYw  \x1b[0m \x1b[31m\x1b[41m  gYw  \x1b[0m \x1b[31m\x1b[42m  gYw  \x1b[0m \x1b[31m\x1b[43m  gYw  \x1b[0m \x1b[31m\x1b[44m  gYw  \x1b[0m \x1b[31m\x1b[45m  gYw  \x1b[0m \x1b[31m\x1b[46m  gYw  \x1b[0m \x1b[31m\x1b[47m  gYw  \x1b[0m
1;31m \x1b[1;31m  gYw   \x1b[1;31m\x1b[40m  gYw  \x1b[0m \x1b[1;31m\x1b[41m  gYw  \x1b[0m \x1b[1;31m\x1b[42m  gYw  \x1b[0m \x1b[1;31m\x1b[43m  gYw  \x1b[0m \x1b[1;31m\x1b[44m  gYw  \x1b[0m \x1b[1;31m\x1b[45m  gYw  \x1b[0m \x1b[1;31m\x1b[46m  gYw  \x1b[0m \x1b[1;31m\x1b[47m  gYw  \x1b[0m
  32m \x1b[32m  gYw   \x1b[32m\x1b[40m  gYw  \x1b[0m \x1b[32m\x1b[41m  gYw  \x1b[0m \x1b[32m\x1b[42m  gYw  \x1b[0m \x1b[32m\x1b[43m  gYw  \x1b[0m \x1b[32m\x1b[44m  gYw  \x1b[0m \x1b[32m\x1b[45m  gYw  \x1b[0m \x1b[32m\x1b[46m  gYw  \x1b[0m \x1b[32m\x1b[47m  gYw  \x1b[0m
1;32m \x1b[1;32m  gYw   \x1b[1;32m\x1b[40m  gYw  \x1b[0m \x1b[1;32m\x1b[41m  gYw  \x1b[0m \x1b[1;32m\x1b[42m  gYw  \x1b[0m \x1b[1;32m\x1b[43m  gYw  \x1b[0m \x1b[1;32m\x1b[44m  gYw  \x1b[0m \x1b[1;32m\x1b[45m  gYw  \x1b[0m \x1b[1;32m\x1b[46m  gYw  \x1b[0m \x1b[1;32m\x1b[47m  gYw  \x1b[0m
  33m \x1b[33m  gYw   \x1b[33m\x1b[40m  gYw  \x1b[0m \x1b[33m\x1b[41m  gYw  \x1b[0m \x1b[33m\x1b[42m  gYw  \x1b[0m \x1b[33m\x1b[43m  gYw  \x1b[0m \x1b[33m\x1b[44m  gYw  \x1b[0m \x1b[33m\x1b[45m  gYw  \x1b[0m \x1b[33m\x1b[46m  gYw  \x1b[0m \x1b[33m\x1b[47m  gYw  \x1b[0m
1;33m \x1b[1;33m  gYw   \x1b[1;33m\x1b[40m  gYw  \x1b[0m \x1b[1;33m\x1b[41m  gYw  \x1b[0m \x1b[1;33m\x1b[42m  gYw  \x1b[0m \x1b[1;33m\x1b[43m  gYw  \x1b[0m \x1b[1;33m\x1b[44m  gYw  \x1b[0m \x1b[1;33m\x1b[45m  gYw  \x1b[0m \x1b[1;33m\x1b[46m  gYw  \x1b[0m \x1b[1;33m\x1b[47m  gYw  \x1b[0m
  34m \x1b[34m  gYw   \x1b[34m\x1b[40m  gYw  \x1b[0m \x1b[34m\x1b[41m  gYw  \x1b[0m \x1b[34m\x1b[42m  gYw  \x1b[0m \x1b[34m\x1b[43m  gYw  \x1b[0m \x1b[34m\x1b[44m  gYw  \x1b[0m \x1b[34m\x1b[45m  gYw  \x1b[0m \x1b[34m\x1b[46m  gYw  \x1b[0m \x1b[34m\x1b[47m  gYw  \x1b[0m
1;34m \x1b[1;34m  gYw   \x1b[1;34m\x1b[40m  gYw  \x1b[0m \x1b[1;34m\x1b[41m  gYw  \x1b[0m \x1b[1;34m\x1b[42m  gYw  \x1b[0m \x1b[1;34m\x1b[43m  gYw  \x1b[0m \x1b[1;34m\x1b[44m  gYw  \x1b[0m \x1b[1;34m\x1b[45m  gYw  \x1b[0m \x1b[1;34m\x1b[46m  gYw  \x1b[0m \x1b[1;34m\x1b[47m  gYw  \x1b[0m
  35m \x1b[35m  gYw   \x1b[35m\x1b[40m  gYw  \x1b[0m \x1b[35m\x1b[41m  gYw  \x1b[0m \x1b[35m\x1b[42m  gYw  \x1b[0m \x1b[35m\x1b[43m  gYw  \x1b[0m \x1b[35m\x1b[44m  gYw  \x1b[0m \x1b[35m\x1b[45m  gYw  \x1b[0m \x1b[35m\x1b[46m  gYw  \x1b[0m \x1b[35m\x1b[47m  gYw  \x1b[0m
1;35m \x1b[1;35m  gYw   \x1b[1;35m\x1b[40m  gYw  \x1b[0m \x1b[1;35m\x1b[41m  gYw  \x1b[0m \x1b[1;35m\x1b[42m  gYw  \x1b[0m \x1b[1;35m\x1b[43m  gYw  \x1b[0m \x1b[1;35m\x1b[44m  gYw  \x1b[0m \x1b[1;35m\x1b[45m  gYw  \x1b[0m \x1b[1;35m\x1b[46m  gYw  \x1b[0m \x1b[1;35m\x1b[47m  gYw  \x1b[0m
  36m \x1b[36m  gYw   \x1b[36m\x1b[40m  gYw  \x1b[0m \x1b[36m\x1b[41m  gYw  \x1b[0m \x1b[36m\x1b[42m  gYw  \x1b[0m \x1b[36m\x1b[43m  gYw  \x1b[0m \x1b[36m\x1b[44m  gYw  \x1b[0m \x1b[36m\x1b[45m  gYw  \x1b[0m \x1b[36m\x1b[46m  gYw  \x1b[0m \x1b[36m\x1b[47m  gYw  \x1b[0m
1;36m \x1b[1;36m  gYw   \x1b[1;36m\x1b[40m  gYw  \x1b[0m \x1b[1;36m\x1b[41m  gYw  \x1b[0m \x1b[1;36m\x1b[42m  gYw  \x1b[0m \x1b[1;36m\x1b[43m  gYw  \x1b[0m \x1b[1;36m\x1b[44m  gYw  \x1b[0m \x1b[1;36m\x1b[45m  gYw  \x1b[0m \x1b[1;36m\x1b[46m  gYw  \x1b[0m \x1b[1;36m\x1b[47m  gYw  \x1b[0m
  37m \x1b[37m  gYw   \x1b[37m\x1b[40m  gYw  \x1b[0m \x1b[37m\x1b[41m  gYw  \x1b[0m \x1b[37m\x1b[42m  gYw  \x1b[0m \x1b[37m\x1b[43m  gYw  \x1b[0m \x1b[37m\x1b[44m  gYw  \x1b[0m \x1b[37m\x1b[45m  gYw  \x1b[0m \x1b[37m\x1b[46m  gYw  \x1b[0m \x1b[37m\x1b[47m  gYw  \x1b[0m
1;37m \x1b[1;37m  gYw   \x1b[1;37m\x1b[40m  gYw  \x1b[0m \x1b[1;37m\x1b[41m  gYw  \x1b[0m \x1b[1;37m\x1b[42m  gYw  \x1b[0m \x1b[1;37m\x1b[43m  gYw  \x1b[0m \x1b[1;37m\x1b[44m  gYw  \x1b[0m \x1b[1;37m\x1b[45m  gYw  \x1b[0m \x1b[1;37m\x1b[46m  gYw  \x1b[0m \x1b[1;37m\x1b[47m  gYw  \x1b[0m
\x1b[5;47;30;4mBLINKI|NG |👩🏿‍⚕️ |日本語|のキー||ボード|  TEXT\x1b[0m
★|☆|☐|☑|☜☝|☞☟|⛃|⛶✔|↺↻|⟲⟳|⬅➡⬆⬇|⬈⬉⬊⬋⬌⬍⮨⮩⮪⮫ ♡ 📅📆|||
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
👩🏿‍⚕️|0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789\n\n1209387123987";

impl TerminalTab {
    pub fn new() -> Self {
        Self {
            data: TEST_TEXT.into(), //"hello \x1b[4;33mYellow underlined text\x1b[0mtt\nthis is a test\n01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567895\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24".into(),
            mode: TerminalMode::Basic,
            cursor: None,
        }
    }
}

mod terminal_formatter {
    use eframe::{
        egui::TextFormat,
        epaint::text::{FontsImpl, LayoutJob},
    };

    pub struct Paragraph {
        default_format: TextFormat,
        character_width: f32,
        target_line_width: f32,
        target_lines: u32,
        lines: Vec<Line>,
    }

    impl Paragraph {
        pub fn new(
            character_width: f32,
            characters_per_line: impl Into<f32>,
            target_lines: u32,
            default_format: TextFormat,
        ) -> Self {
            Self {
                character_width,
                target_line_width: characters_per_line.into() * character_width,
                target_lines,
                lines: Default::default(),
                default_format,
            }
        }

        fn last(&mut self) -> &mut Line {
            if self.lines.is_empty() {
                self.lines.push(Line::new());
            }
            match self.lines.last_mut() {
                Some(last) => return last,
                None => {
                    panic!("lines are empty? but pushed if empty IMPOSSIBLE")
                }
            }
        }

        fn append_last(&mut self, section: impl Into<Section>) {
            self.last().append(section.into());
        }

        fn finish_current_last_line(&mut self) {
            let leading_space =
                ((self.target_line_width - self.last().width()) / self.character_width).round();
            let mut str = String::new();
            for _ in 0..leading_space as usize {
                str.push(' ');
            }
            str.push('\n');

            self.append_last(Section {
                text_width: 0.0,
                leading_space: 0.0,
                text: str,
                format: self.default_format.clone(),
            });
        }

        fn new_line(&mut self) {
            self.finish_current_last_line();
            self.lines.push(Line::new());
        }

        pub fn append_text(&mut self, str: &str, format: TextFormat, fonts: &mut FontsImpl) {
            let mut tmp = Section::new(format.clone());
            let font = fonts.font(&format.font_id);

            for char in str.chars() {
                if char == '\n' {
                    if !tmp.is_empty() {
                        self.append_last(tmp.clone());
                        tmp = Section::new(format.clone());
                    }
                    // if self.last_empty(){
                    //     self.new_line();
                    //     continue;
                    // }
                    self.new_line();
                } else {
                    if char == '\t' {
                        for _ in 0..4 {
                            let width = font.glyph_width(' ');
                            //let width = font.round_to_pixel(width);
                            if font.round_to_pixel(self.last().width() + tmp.width() + width)
                                > font.round_to_pixel(self.target_line_width)
                            {
                            } else {
                                tmp.append_char(' ', width);
                            }
                        }
                    } else {
                        let mut width = font.glyph_width(char);
                        let mut char = char;
                        if width != self.character_width && width != 0.0 {
                            char = '�';
                            width = font.glyph_width(char);
                        }
                        if font.round_to_pixel(self.last().width() + tmp.width() + width)
                            > font.round_to_pixel(self.target_line_width)
                        {
                            self.append_last(tmp);
                            self.new_line();
                            tmp = Section::new(format.clone());
                        }
                        tmp.append_char(char, width);
                    }
                }
            }
            if !tmp.is_empty() {
                self.append_last(tmp);
            }
        }

        pub fn layout(&mut self, layout: &mut LayoutJob) {
            while self.lines.len() < self.target_lines as usize {
                self.new_line();
            }
            self.finish_current_last_line();
            //removes last new line
            self.last().sections.last_mut().unwrap().text.pop();

            for i in (self.lines.len() - self.target_lines as usize)..(self.lines.len()) {
                for section in &mut self.lines[i].sections {
                    layout.append(&section.text, section.leading_space, section.format.clone());
                }
            }
        }
    }
    pub struct Line {
        width: f32,
        sections: Vec<Section>,
    }

    impl Line {
        fn width(&self) -> f32 {
            self.width
        }
        fn new() -> Self {
            Self {
                width: 0.0,
                sections: Vec::new(),
            }
        }
        fn append(&mut self, section: Section) {
            self.width += section.width();
            self.sections.push(section);
        }
    }
    #[derive(Clone)]
    pub struct Section {
        text_width: f32,
        leading_space: f32,
        text: String,
        format: TextFormat,
    }
    impl Section {
        fn width(&self) -> f32 {
            self.text_width + self.leading_space
        }
        fn is_empty(&self) -> bool {
            self.text.is_empty()
        }
        fn append_char(&mut self, char: char, width: f32) {
            self.text.push(char);
            self.text_width += width;
        }
        fn new(format: TextFormat) -> Self {
            Self {
                text_width: 0.0,
                leading_space: 0.0,
                text: String::new(),
                format,
            }
        }
    }
}

impl Tab for TerminalTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let mut layout = LayoutJob::default();

        let font_family = FontFamily::Monospace; //FontFamily::Name("DroidSansMono".into());

        //font size calculation
        //max horizontal size
        let mut size = (ui.max_rect().width() * 1.8 / 80.0) as i32;
        if size <= 0 {
            size = 1;
        };
        //max vertical size
        while size > 1 {
            if FontId::new(size as f32, font_family.clone()).size * 24.0 > ui.available_height() {
                size -= 1;
            } else {
                break;
            }
        }
        let size = size as f32;

        let font_id = FontId::new(size, font_family);

        let term_background = ui.style().visuals.extreme_bg_color;

        let single_width = ui
            .ctx()
            .fonts()
            .lock()
            .fonts
            .font(&font_id)
            .glyph_width(' ');

        let mut paragraph = terminal_formatter::Paragraph::new(
            single_width,
            80.0,
            24,
            TextFormat {
                font_id: font_id.clone(),
                background: term_background,
                ..Default::default()
            },
        );

        let pre_colors = [
            Color32::from_gray(0x00),            //black
            Color32::from_rgb(0xCD, 0x31, 0x31), //red
            Color32::from_rgb(0x0D, 0xDC, 0x79), //green
            Color32::from_rgb(0xE5, 0xE5, 0x10), //yellow
            Color32::from_rgb(0x24, 0x72, 0xC8), //blue
            Color32::from_rgb(0xBC, 0x3F, 0xBC), //purple
            Color32::from_rgb(0x11, 0xA8, 0xCD), //cyan
            Color32::from_gray(0xE5),            //white
        ];

        let pre_colors_vib = [
            Color32::from_gray(0x66),            //black
            Color32::from_rgb(0xF1, 0x4C, 0x4C), //red
            Color32::from_rgb(0x23, 0xD1, 0x8B), //green
            Color32::from_rgb(0xF5, 0xF5, 0x43), //yellow
            Color32::from_rgb(0x3B, 0x8E, 0xEA), //blue
            Color32::from_rgb(0xD6, 0x70, 0xD6), //purple
            Color32::from_rgb(0x29, 0xB8, 0xDB), //cyan
            Color32::from_gray(0xE5),            //white
        ];

        let mut background: Option<Color32> = Option::None;
        let mut forground: Option<Color32> = Option::None;
        let mut blink = false;
        let mut bold = false;
        let mut underline = false;
        let mut italics = false;
        let mut strike = false;

        fn color_code_parser(codes: &mut Vec<u8>) -> Result<Color32, ()> {
            if codes.is_empty() {
                return Err(());
            }
            match codes.remove(0) {
                5 => Err(()),
                2 => {
                    if codes.len() < 3 {
                        return Err(());
                    }
                    Ok(Color32::from_rgb(
                        codes.remove(0),
                        codes.remove(0),
                        codes.remove(0),
                    ))
                }
                _ => Err(()),
            }
        }

        let iter = TerminalParser::new(self.data.as_str());

        let alpha = 255 * (1 & ((time::duration_since_epoch().as_millis()) * 120 / 60000)) as u8;

        for (str, mut num_codes) in iter {
            while !num_codes.is_empty() {
                match num_codes.remove(0) {
                    0 => {
                        forground = None;
                        background = None;
                        bold = false;
                        italics = false;
                        strike = false;
                        underline = false;
                        blink = false;
                    }
                    1 => bold = true,
                    2 => bold = false,
                    3 => italics = true,
                    4 => underline = true,
                    5 => blink = true,
                    9 => strike = false,
                    21 => bold = false,
                    22 => italics = false,
                    24 => underline = false,
                    25 => blink = false,
                    29 => strike = false,

                    color @ 30..=37 => forground = Option::Some(pre_colors[color as usize - 30]),
                    color @ 90..=97 => {
                        forground = Option::Some(pre_colors_vib[color as usize - 90])
                    }
                    38 => {
                        if let Ok(val) = color_code_parser(&mut num_codes) {
                            forground = Option::Some(val);
                        }
                    }
                    39 => forground = None,

                    color @ 40..=47 => background = Option::Some(pre_colors[color as usize - 40]),
                    color @ 100..=107 => {
                        background = Option::Some(pre_colors_vib[color as usize - 100])
                    }
                    48 => {
                        if let Ok(val) = color_code_parser(&mut num_codes) {
                            background = Option::Some(val);
                        }
                    }
                    49 => background = None,
                    _ => {}
                }
            }

            let mut forground = forground.unwrap_or(ui.style().visuals.widgets.open.text_color());
            if blink {
                ui.ctx().request_repaint();
                if alpha > 0 {
                    forground = Color32::TRANSPARENT
                }
            }
            let background = background.unwrap_or(term_background);
            let stroke = Stroke::new(1.0, forground);
            // fn generate_text_format(&self) -> TextFormat{
            let format = TextFormat {
                font_id: font_id.clone(),
                color: forground,
                background,
                italics: italics,
                underline: if underline { stroke } else { Stroke::none() },
                strikethrough: if strike { stroke } else { Stroke::none() },
                ..Default::default()
            };
            if bold {
                //lpol
            }

            paragraph.append_text(str.as_str(), format, &mut ui.fonts().lock().fonts);
        }

        paragraph.layout(&mut layout);

        layout.wrap = TextWrapping {
            max_rows: 24,
            ..Default::default()
        };

        let gallery = ui.fonts().layout_job(layout);

        let y_space = (ui.available_height() - gallery.rect.height()) / 2.0;
        if y_space > 0.0 {
            ui.add_space(y_space);
        }

        ui.horizontal(|ui| {
            let x_space = (ui.available_width() - gallery.rect.width()) / 2.0;
            if x_space > 0.0 {
                ui.add_space(x_space);
            }

            let (pos, response) = ui.allocate_exact_size(
                gallery.size(),
                Sense {
                    click: true,
                    drag: true,
                    focusable: true,
                },
            );
            if response.hovered() && ui.input().pointer.any_pressed() {
                ui.memory().request_focus(response.id);
            }
            if ui.memory().has_focus(response.id) {
                ui.memory().lock_focus(response.id, true);
            }
            //ui.ctx().fonts().font(FontId::monospace(size));

            //let data = UnsafeCell::new(Pin::new(&12));

            response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, gallery.text()));

            if ui.is_rect_visible(response.rect) {
                let stroke = if response.has_focus() {
                    Stroke::new(5.0, Color32::RED)
                } else {
                    ui.style().visuals.widgets.noninteractive.bg_stroke
                };
                ui.painter()
                    .rect_stroke(pos.expand(2.0), Rounding::same(5.0), stroke);

                ui.painter().add(epaint::TextShape {
                    pos: pos.left_top(),
                    galley: gallery,
                    override_text_color: None,
                    underline: Stroke::none(),
                    angle: 0.0,
                });
            }
        });

        //ui.label(layout);
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        "Mips Terminal".into()
    }
}

struct TerminalParser<'a> {
    iter: Chars<'a>,
    string: String,
    #[allow(unused)]
    row_count: usize,
    // background: Option<Color32>,
    // forground: Option<Color32>,
    // bold: bool,
    // underline: bool,
    // italics: bool,
    // strike: bool,
    state: usize,
    // font_size: f32,
}

impl<'a> TerminalParser<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            iter: str.chars(),
            // background: Option::None,
            // forground: Option::None,
            // bold: false,
            // underline: false,
            // italics: false,
            // strike: false,
            row_count: 0,
            state: 0,
            string: String::new(),
            //font_size,
        }
    }
}

impl<'a> Iterator for TerminalParser<'a> {
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut send: bool = false;
        let mut num_codes: Vec<u8> = Vec::new();
        loop {
            match self.iter.next() {
                Some(char) => {
                    match self.state {
                        0 => match char {
                            '\x1b' => {
                                self.state = 1;
                                send = true;
                            }
                            char => {
                                self.string.push(char);
                            }
                        },
                        1 => {
                            match char {
                                '[' => {
                                    let mut code = String::new();
                                    while let Option::Some(char) = self.iter.next() {
                                        if char == 'm' {
                                            break;
                                        }
                                        code.push(char);
                                    }
                                    let codes = code.split(';');
                                    num_codes.clear();
                                    for code in codes {
                                        let res = code.parse();
                                        match res {
                                            Ok(val) => num_codes.push(val),
                                            Err(_) => {
                                                self.state = 0;
                                                num_codes.clear();
                                                continue;
                                            }
                                        }
                                    }
                                    self.state = 0;
                                }
                                _ => {
                                    //this is an error but we just ignore it
                                    self.state = 0;
                                    continue;
                                }
                            }
                        }
                        _ => {
                            panic!()
                        }
                    }
                }
                None => {
                    if self.string.is_empty() {
                        return None;
                    } else {
                        send = true;
                    }
                }
            }
            if send {
                if !self.string.is_empty() {
                    let mut string = String::new();
                    std::mem::swap(&mut string, &mut self.string);
                    return Option::Some((string, num_codes));
                }
            }
        }
    }
}
