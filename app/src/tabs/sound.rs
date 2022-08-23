
use eframe::egui::plot::{PlotPoints};
use eframe::egui::{ComboBox, Slider};
use egui_dock::Tab;
use rodio::source::{Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::f64::consts::PI;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use self::filter::FirstOrderFilter;

pub struct SoundTab {
    #[allow(unused)]
    stream_handle: OutputStreamHandle,
    #[allow(unused)]
    stream: OutputStream,
    sound_state: SharedSoundState,
    wave: [f32; 1024],
}

#[derive(Debug, PartialEq, Eq)]
enum WaveType{
    Square,
    Sine,
}

impl SoundTab {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let sound_state = SoundState::new();
        let source = APU::new(sound_state.clone());
        sink.append(source);
        sink.detach();
        SoundTab {
            sound_state,
            stream_handle,
            stream,
            wave: [0.0; 1024],
        }
    }
}

impl Tab for SoundTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        
        let mut lock = self.sound_state.write().unwrap();
        let sound_state = &mut *lock;

        if ui.selectable_label(sound_state.mute, "Mute").clicked(){
            sound_state.mute = !sound_state.mute;
        }

        ui.add(
            Slider::new(&mut sound_state.freq, 10.0..=2000.0)
                    .max_decimals(2)
                    .min_decimals(2)
                    .show_value(true).prefix("Freq")
        );

        if ui.selectable_label(sound_state.use_filters, "Use Filters").clicked(){
            sound_state.use_filters = !sound_state.use_filters;
        };
        ComboBox::from_label("WaveForm")
            .selected_text(format!("{:?}", sound_state.wave_type))
            .show_ui(ui, |ui|{
                ui.selectable_value(&mut sound_state.wave_type, WaveType::Square, "Square");
                ui.selectable_value(&mut sound_state.wave_type, WaveType::Sine, "Sin");
            });

        ui.add(
            Slider::new(&mut sound_state.volume, 0.0..=1.0)
                .max_decimals(2)
                .min_decimals(2)
                .show_value(true).prefix("Volume")
            );

        sound_state.wave_form.copy_to_slice(self.wave.len(), &mut self.wave);
        let plot = eframe::egui::plot::Plot::new("sound wave").legend(eframe::egui::plot::Legend::default())
                        //.data_aspect(500.0)
                        //.view_aspect(500.0)
                        .include_x(self.wave.len() as f64)
                        .include_x(0)
                        .include_y(1).include_y(-1)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .show_axes([false, false])
                        .show_background(false)
                        //.height(2.0)
                        //.width(200.0)
                        .center_y_axis(true);
        plot.show(ui, |plot_ui|{
            plot_ui.line(eframe::egui::plot::Line::new(
                PlotPoints::from_ys_f32(&self.wave)));
        });

        if sound_state.volume > 0.0 {
            ui.ctx().request_repaint();
        }
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        "MIPS Sound".into()
    }
}

pub fn temp(reg: u16, _data: u8){
    //t = 1000000.0*1.789773/(16*fpulse) - 1
    //let freq = 1000000.0*1.789773/(16*(t+1))
    match reg{
        0x4000 => {
            //DDLCNNNN
            //D = duity cycle
            //L = loop enbelope / disable length counter
            //C = constant volume
            //N = envelope pediod/volume
        }
        0x4001 => {
            //EPPPNSSS
            //S = control how much 0x4002 is shifted by to get the new frequency
            //N = controlls wheather to iuncrease or decrease frequency for sweep (1=increase, 0=decrease)
            //P = control how fast the sweep runs(0-7)
            //E = enables sweep
        }
        0x4002 => {
            //first 8 bits of the frequency
        }
        0x4003 => {
            //bits 0-2 wavelength stuff
            //bits 3-7 load the down-counter
        }

        0x4004 => {
            
        }
        0x4005 => {
            
        }
        0x4006 => {
            
        }
        0x4007 => {
            
        }

        0x4008 => {
            
        }
        0x4009 => {
            
        }
        0x400A => {
            
        }
        0x400B => {
            
        }

        0x400C => {
            
        }
        0x400D => {
            
        }
        0x400E => {
            
        }
        0x400F => {
            
        }

        0x4010 => {
            
        }
        0x4012 => {
            
        }
        0x4013 => {
            
        }
        0x4015 => {
            //bit 0 = square wave channel 1 enable
            //bit 1 = square wave channel 2 enable
            //bit 2 = triangle wave channel 3 enable
            //bit 3 = noise wave channel 4 enable
            //bit 4 =  DMC/PCM playback channel
            //Bits 5-7 = unused
        }
        _ => {panic!()}
    }
}

const WAVE_FORM_SIZE: usize = 2048;
struct WaveTest{
    wave_form: [f32; WAVE_FORM_SIZE],
    curr: usize,
}

impl Default for WaveTest{
    fn default() -> Self {
        Self { 
            wave_form: [0.0; WAVE_FORM_SIZE], 
            curr: Default::default()
        }
    }
}

impl WaveTest{
    pub fn add(&mut self, val: f32){
        self.add_last(1);
        self.wave_form[self.curr] = val;
    }
    fn add_last(&mut self, amount: usize){
        self.curr += amount;
        self.curr = self.curr % WAVE_FORM_SIZE;
    }
    pub fn copy_to_slice(&mut self, amount: usize, slice: &mut [f32]) {
        let mut tmp = self.curr.wrapping_sub(amount / 2) % WAVE_FORM_SIZE;
        let mut start = self.curr;
        let mut last = self.wave_form[tmp];
        for _i in 0..WAVE_FORM_SIZE{
            let curr = self.wave_form[tmp];
            if last <= 0.0 && curr > 0.0{
                break;
            }
            last = curr;
            tmp = tmp.wrapping_sub(1) % WAVE_FORM_SIZE;
            start = start.wrapping_sub(1) % WAVE_FORM_SIZE;
        }

        for i in (0..amount).rev(){
            slice[i] = self.wave_form[(start.wrapping_sub(i)) % WAVE_FORM_SIZE];
        }
    }
}

type SharedSoundState = Arc<RwLock<SoundState>>;

struct SoundState {
    freq: f64,
    use_filters: bool,
    wave_type: WaveType,
    volume: f64,
    mute: bool,
    wave_form: WaveTest,
}

impl SoundState {
    pub fn new() -> SharedSoundState {
        Arc::new(RwLock::new(SoundState {
            freq: 440.0,
            use_filters:true,
            wave_type: WaveType::Sine,
            volume: 0.15,
            wave_form: Default::default(),
            mute: true,
        }))
    }
}

struct APU {
    sound_state: SharedSoundState,
    num_sample: usize,
    filters: Vec<FirstOrderFilter>,
}

impl APU {
    pub fn new(sound_state: SharedSoundState) -> Self {
        Self {
            sound_state,
            num_sample: 0,
            filters: vec![
                FirstOrderFilter::high_pass(44100.0, 90.0),
                FirstOrderFilter::low_pass(44100.0, 14_000.0),
            ],
        }
    }
}

impl Iterator for APU {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_sample = self.num_sample.wrapping_add(1);

        let mut lock = self.sound_state.write().unwrap();
        let sound_state = &*lock;

        let freq = sound_state.freq;

        let tmp = freq * self.num_sample as f64 / 44100.0;

        let mut output = match sound_state.wave_type{
            WaveType::Square => {
                 if tmp % 1.0 > 0.5{
                    1.0
                }else{
                    -1.0
                }
            },
            WaveType::Sine => {
                (tmp * 2.0 * PI).sin()
            },
        };

        if sound_state.use_filters{
            for filter in &mut self.filters {
                output = filter.tick(output);
            }
        }    

        let output = (output * sound_state.volume) as f32;
        lock.wave_form.add(output);
        if lock.mute {Some(0.0)} else {Some(output)}
    }
}

impl Source for APU {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        44100
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub mod filter {

    use std::f64::consts::PI;

    pub struct FirstOrderFilter {
        b0: f64,
        b1: f64,
        a1: f64,
        prev_x: f64,
        prev_y: f64,
    }

    impl FirstOrderFilter {
        pub fn high_pass(sample_rate: f64, cutoff_frequency: f64) -> Self {
            let c = sample_rate / PI / cutoff_frequency;
            let a0i = 1.0 / (1.0 + c);

            FirstOrderFilter {
                b0: c * a0i,
                b1: -c * a0i,
                a1: (1.0 - c) * a0i,
                prev_x: 0.0,
                prev_y: 0.0,
            }
        }

        pub fn low_pass(sample_rate: f64, cutoff_frequency: f64) -> Self {
            let c = sample_rate / PI / cutoff_frequency;
            let a0i = 1.0 / (1.0 + c);

            FirstOrderFilter {
                b0: a0i,
                b1: a0i,
                a1: (1.0 - c) * a0i,
                prev_x: 0.0,
                prev_y: 0.0,
            }
        }

        pub fn tick(&mut self, x: f64) -> f64 {
            let y = self.b0 * x + self.b1 * self.prev_x - self.a1 * self.prev_y;
            self.prev_y = y;
            self.prev_x = x;
            y
        }
    }
}
