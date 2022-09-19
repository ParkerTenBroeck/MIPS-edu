use eframe::egui::RichText;
use egui_dock::Tab;

use crate::{
    emulator::{
        debug_target::Breakpoint,
        debugger_thread::{ConnectionInfo, State},
    },
    platform,
};

type Debugger = std::sync::Arc<
    std::sync::Mutex<
        dyn crate::emulator::debugger_thread::DebuggerConnection<
            crate::emulator::debug_target::MipsTargetInterface<
                crate::emulator::handlers::ExternalHandler,
            >,
        >,
    >,
>;

pub struct DebuggerTab {
    debugger: Debugger,
    con_info: Option<ConnectionInfo>,
    bps: Option<Vec<Breakpoint>>,
    status: State,
}
impl Tab for DebuggerTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        if let Ok(mut debugger) = self.debugger.try_lock() {
            let debugger = &mut *debugger;
            self.status = debugger.state();
            
            if matches!(self.status, State::Connected){ 
                if let Ok(con_info) = debugger.try_get_connection_info() {
                    if con_info.is_some(){
                        self.con_info = con_info;
                    }
                }
                debugger.try_target(Box::new(|target| {
                    
                    self.bps = Some(target.breakpoints().to_owned())
                }));       
            }
        }

        

        if matches!(self.status, State::Connected | State::Connecting){
            ui.ctx().request_repaint();
        }else{
            self.con_info = None;
            self.bps = None;
        }

        ui.label(format!("Connection Status: {:#?}", self.status));
        ui.label(format!("{:#?}", self.con_info));
        ui.label("Breakpoints");
        ui.label(format!("{:#?}", self.bps));
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        if let Ok(debugger) = self.debugger.try_lock() {
            self.status = debugger.state();
        }

        match self.status {
            State::Disconnected => RichText::new("Debugger D").into(),
            State::Connecting => {
                let millis = platform::time::duration_since_epoch().as_millis() / 500;
                let mut tmp = *b"Debugger C   ";
                let mut i = (millis % 4) as usize;
                if i > 2 {
                    i = 1;
                }
                tmp[tmp.len() - 1 - i] = b'*';

                RichText::new(std::str::from_utf8(&tmp).unwrap()).into()
            }
            State::Connected => RichText::new("Debugger R").into(),
        }
    }
}
impl DebuggerTab {
    pub fn new(debugger: Debugger) -> Self {
        Self {
            debugger,
            con_info: None,
            bps: None,
            status: State::Disconnected,
        }
    }
}
