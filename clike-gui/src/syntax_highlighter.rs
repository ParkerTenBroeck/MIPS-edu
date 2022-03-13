use clike::parsing_lexer::tokenizer::IdentifierMode;
#[allow(unused_imports)]
use eframe::{egui, epi};
use eframe::egui::{Color32};
use eframe::egui::text::LayoutJob;
use crate::syntax_highlighter;

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(ui: &mut eframe::egui::Ui, mut code: &str) {
    let language = "rs";
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };

    ui.add(
        eframe::egui::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code, language))
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntect"))]
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
enum TokenType {
    Comment,
    Documentation,
    Keyword,
    Identifier,
    NumberLiteral,
    StringLiteral,
    Punctuation,
    Whitespace,
    Error,
}

#[derive(Clone, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeTheme {
    dark_mode: bool,

    #[cfg(feature = "syntect")]
    syntect_theme: SyntectTheme,

    #[cfg(not(feature = "syntect"))]
    formats: enum_map::EnumMap<TokenType, eframe::egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &eframe::egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data()
                .get_persisted(egui::Id::new("dark"))
                .unwrap_or_else(CodeTheme::dark)
        } else {
            ctx.data()
                .get_persisted(egui::Id::new("light"))
                .unwrap_or_else(CodeTheme::light)
        }
    }

    pub fn store_in_memory(self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.data().insert_persisted(egui::Id::new("dark"), self);
        } else {
            ctx.data().insert_persisted(egui::Id::new("light"), self);
        }
    }
}

#[cfg(not(feature = "syntect"))]
impl CodeTheme {
    pub fn dark() -> Self {
        let font_id = eframe::egui::FontId::monospace(14.0);
        use eframe::egui::{TextFormat};
        Self {
            //func  255, 198, 109 #ffc66d
            //yellow 187, 181, 41 #bbb529
            dark_mode: true,
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::from_gray(128)),
                TokenType::Documentation => TextFormat::simple(font_id.clone(), Color32::from_rgb(98, 151, 85)),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(204, 120, 50)),
                TokenType::Identifier => TextFormat::simple(font_id.clone(), Color32::from_rgb(152, 118, 170)),
                TokenType::NumberLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(72, 130, 186)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(106, 135, 89)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::from_rgb(160, 182, 197)),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
                TokenType::Error => {let mut fmt = TextFormat::simple(font_id.clone(), Color32::from_rgb(160, 182, 197)); fmt.background = Color32::RED; fmt},
            ],
        }
    }

    pub fn light() -> Self {
        let font_id =eframe::egui::FontId::monospace(14.0);
        use eframe::egui::{TextFormat};
        Self {
            dark_mode: false,
            #[cfg(not(feature = "syntect"))]
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(235, 0, 0)),
                TokenType::Identifier => TextFormat::simple(font_id.clone(), Color32::from_rgb(153, 134, 255)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(37, 203, 105)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),

                TokenType::NumberLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(72, 130, 186)),
                TokenType::Documentation => TextFormat::simple(font_id.clone(), Color32::from_rgb(98, 151, 85)),
                TokenType::Error => {let mut fmt = TextFormat::simple(font_id.clone(), Color32::from_rgb(160, 182, 197)); fmt.background = Color32::RED; fmt},
            ],
        }
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal_top(|ui| {
            let selected_id =eframe::egui::Id::null();
            let mut selected_tt: TokenType = *ui
                .data()
                .get_persisted_mut_or(selected_id, TokenType::Comment);

            ui.vertical(|ui| {
                ui.set_width(150.0);
               eframe::egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.scope(|ui| {
                    for (tt, tt_name) in [
                        (TokenType::Comment, "// comment"),
                        (TokenType::Documentation, "/// documentation"),
                        (TokenType::Keyword, "keyword"),
                        (TokenType::Identifier, "ident"),
                        (TokenType::StringLiteral, "\"string literal\""),
                        (TokenType::NumberLiteral, "number 12 + 3.0e+3"),
                        (TokenType::Punctuation, "punctuation + / -"),
                        // (TokenType::Whitespace, "whitespace"),
                    ] {
                        let format = &mut self.formats[tt];
                        ui.style_mut().override_font_id = Some(format.font_id.clone());
                        ui.visuals_mut().override_text_color = Some(format.color);
                        ui.radio_value(&mut selected_tt, tt, tt_name);
                    }
                });

                let reset_value = if self.dark_mode {
                    CodeTheme::dark()
                } else {
                    CodeTheme::light()
                };

                if ui
                    .add_enabled(*self != reset_value,eframe::egui::Button::new("Reset theme"))
                    .clicked()
                {
                    *self = reset_value;
                }
            });

            ui.add_space(16.0);

            ui.data().insert_persisted(selected_id, selected_tt);

           eframe::egui::Frame::group(ui.style())
                .margin(egui::Vec2::splat(2.0))
                .show(ui, |ui| {
                    // ui.group(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    ui.spacing_mut().slider_width = 128.0; // Controls color picker size
                   eframe::egui::widgets::color_picker::color_picker_color32(
                        ui,
                        &mut self.formats[selected_tt].color,
                       eframe::egui::color_picker::Alpha::Opaque,
                    );
                });
        });
    }
}



#[cfg(not(feature = "syntect"))]
#[derive(Default)]
struct Highlighter {}

#[cfg(not(feature = "syntect"))]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, text: &str, _language: &str) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect
        use clike::parsing_lexer::highlighter_tokenizer::HighlighterTokenizer;

        let mut job = LayoutJob::default();
        use clike::parsing_lexer::tokenizer::{Tokenizer, TokenType};
        let mut tokenizer = HighlighterTokenizer::new(
            Tokenizer::from_str(text).ident_mode(IdentifierMode::Unicode));

        loop {
            match tokenizer.next(){
                Option::None => break,
                Option::Some(val) => {
                    match val{
                        Result::Err(val) => {
                            let mut theme = theme.formats[syntax_highlighter::TokenType::Punctuation].clone();
                            theme.background = Color32::from_rgb(255,0,0);
                            //theme.color = theme.background;
                            job.append(&text[val.0..val.0 + val.1], 0.0, theme)
                        }
                        Result::Ok(val) => {
                            let (t_type, t_data, under) = match val{
                                Result::Ok(val) =>{
                                    (val.1.t_type, val.0, false)
                                }
                                Result::Err(val) =>{
                                    (val.2, val.0, true)
                                }
                            };
                            //println!("{:?}", (&t_type, &t_data, &under));
                            let mut theme = match t_type{
                                TokenType::StringLiteral(_) |
                                TokenType::CharLiteral(_) => {
                                    theme.formats[syntax_highlighter::TokenType::StringLiteral].clone()
                                }
                                TokenType::Comma | TokenType::Semicolon => {
                                    theme.formats[syntax_highlighter::TokenType::Keyword].clone()
                                }
                                TokenType::VoidKeyword |
                                TokenType::StructKeyword |
                                TokenType::AsmKeyword |
                                TokenType::ConstKeyword |
                                TokenType::StaticKeyword |
                                TokenType::SizeofKeyword |
                                TokenType::EnumKeyword |
                                TokenType::FnKeyword |
                                TokenType::PubKeyword |
                                TokenType::SuperKeyword |
                                TokenType::SelfKeyword |
                                TokenType::LetKeyword |
                                TokenType::IfKeyword |
                                TokenType::ElseKeyword |
                                TokenType::WhileKeyword |
                                TokenType::DoKeyword |
                                TokenType::ReturnKeyword |
                                TokenType::ForKeyword |
                                TokenType::BreakKeyword |
                                TokenType::SwitchKeyword |
                                TokenType::CaseKeyword |
                                TokenType::GotoKeyword |
                                TokenType::RestrictKeyword => {
                                    theme.formats[syntax_highlighter::TokenType::Keyword].clone()
                                }
                                TokenType::BoolLiteral(_) => {
                                    theme.formats[syntax_highlighter::TokenType::Keyword].clone()
                                }
                                TokenType::I8Literal(_) |
                                TokenType::I16Literal(_) |
                                TokenType::I32Literal(_) |
                                TokenType::I64Literal(_) |
                                TokenType::I128Literal(_) |
                                TokenType::U8Literal(_) |
                                TokenType::U16Literal(_) |
                                TokenType::U32Literal(_) |
                                TokenType::U64Literal(_) |
                                TokenType::U128Literal(_) |
                                TokenType::F32Literal(_) |
                                TokenType::F64Literal(_) => {
                                    theme.formats[syntax_highlighter::TokenType::NumberLiteral].clone()
                                }
                                TokenType::Comment(_) => {
                                    theme.formats[syntax_highlighter::TokenType::Comment].clone()
                                }
                                TokenType::OuterDocumentation(_) |
                                TokenType::InnerDocumentation(_)=> {
                                    theme.formats[syntax_highlighter::TokenType::Documentation].clone()
                                }
                                TokenType::Identifier(_) => {
                                    theme.formats[syntax_highlighter::TokenType::Identifier].clone()
                                }
                                TokenType::Whitespace => {
                                    theme.formats[syntax_highlighter::TokenType::Whitespace].clone()
                                }
                                _ => {
                                    theme.formats[syntax_highlighter::TokenType::Punctuation].clone()
                                }
                            };
                            if under{
                                theme.underline.color = Color32::from_rgb(255,0,0);
                                theme.underline.width = 1.0;
                            }
                            job.append(tokenizer.t().str_from_token_data(&t_data), 0.0, theme)
                        }
                    }
                }
            }
        }
        job
    }
}
