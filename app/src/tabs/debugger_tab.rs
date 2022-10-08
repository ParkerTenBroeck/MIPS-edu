use std::sync::{Arc, Mutex};

use eframe::egui::RichText;
use egui_dock::Tab;
use mips_emulator::cpu::CpuExternalHandler;

use crate::{
    emulator::{
        debug_target::{Breakpoint, MipsTargetInterface},
        debugger_thread::{ConnectionInfo, DebuggerConnection, State},
    },
    platform,
};

type Debugger<T> = Arc<Mutex<dyn DebuggerConnection<MipsTargetInterface<T>>>>;

pub struct DebuggerTab<T: CpuExternalHandler> {
    debugger: Debugger<T>,
    con_info: Option<ConnectionInfo>,
    bps: Option<Vec<Breakpoint>>,
    status: State,
    show_close_dailog: bool,
    force_close: bool,
}

impl<T: CpuExternalHandler> Tab for DebuggerTab<T> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        if let Ok(mut debugger) = self.debugger.try_lock() {
            if self.force_close {
                debugger.force_close();
            }
            let debugger = &mut *debugger;
            self.status = debugger.state();

            if matches!(self.status, State::Connected) {
                if let Ok(con_info) = debugger.try_get_connection_info() {
                    if con_info.is_some() {
                        self.con_info = con_info;
                    }
                }
                debugger.try_target(Box::new(|target| {
                    self.bps = Some(target.breakpoints().to_owned())
                }));
            }
        }

        if self.show_close_dailog {
            eframe::egui::Window::new("Are you sure").show(ui.ctx(), |ui| {
                ui.label("Closing this tab will disconnect the debugger");
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.force_close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_close_dailog = false;
                    }
                });
            });
        }

        if matches!(self.status, State::Connected | State::Connecting) {
            ui.ctx().request_repaint();
        } else {
            self.con_info = None;
            self.bps = None;
        }

        ui.label(format!("Connection Status: {:#?}", self.status));
        ui.label(format!("{:#?}", self.con_info));
        ui.label("Breakpoints");
        ui.label(format!("{:#?}", self.bps));
    }

    fn on_close(&mut self) -> bool {
        if matches!(self.status, State::Disconnected) {
            return true;
        }
        self.show_close_dailog = true;
        false
    }

    fn force_close(&mut self) -> bool {
        if matches!(self.status, State::Disconnected) {
            self.force_close
        } else {
            false
        }
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

impl<T: CpuExternalHandler> DebuggerTab<T> {
    pub fn new(debugger: Debugger<T>) -> Self {
        Self {
            debugger,
            con_info: None,
            bps: None,
            status: State::Disconnected,
            show_close_dailog: false,
            force_close: false,
        }
    }
}
