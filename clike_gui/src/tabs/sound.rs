use eframe::egui::{ComboBox, Slider};
use rodio::source::{SineWave, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::f64::consts::PI;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use self::filter::FirstOrderFilter;

use super::tabbed_area::Tab;

pub struct SoundTab {
    #[allow(unused)]
    stream_handle: OutputStreamHandle,
    #[allow(unused)]
    stream: OutputStream,
    sound_state: SharedSoundState,
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
        }
    }
}

impl Tab for SoundTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        
        let mut lock = self.sound_state.write().unwrap();
        let sound_state = &mut *lock;
        
        
        let res = ui.button("TEST");
        
        if res.clicked() {
            if sound_state.freq == 440.0 {
                sound_state.freq = 550.0;
            } else {
                sound_state.freq = 440.0;
            }
        }
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
    }

    fn get_name(&self) -> eframe::egui::WidgetText {
        "Sound Tab".into()
    }
}


type SharedSoundState = Arc<RwLock<SoundState>>;

struct SoundState {
    freq: f64,
    use_filters: bool,
    wave_type: WaveType,
    volume: f64,
}

impl SoundState {
    pub fn new() -> SharedSoundState {
        Arc::new(RwLock::new(SoundState {
            freq: 440.0,
            use_filters:true,
            wave_type: WaveType::Sine,
            volume: 0.15,
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
                FirstOrderFilter::high_pass(44100.0, 440.0),
                FirstOrderFilter::low_pass(44100.0, 14_000.0),
            ],
        }
    }
}

impl Iterator for APU {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_sample = self.num_sample.wrapping_add(1);

        let lock = self.sound_state.read().unwrap();
        let sound_state = &*lock;

        let freq = sound_state.freq;

        let tmp = freq * self.num_sample as f64 / 44100.0;

        let mut output = match sound_state.wave_type{
            WaveType::Square => {
                 if tmp % 1.0 > 0.5{
                    1.0
                }else{
                    0.0
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

        Some((output * sound_state.volume) as f32)
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
