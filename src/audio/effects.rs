pub struct Reverb {
    delay_lines: Vec<DelayLine>,
    feedback: f32,
    wet_mix: f32,
}

pub struct DelayLine {
    buffer: Vec<f32>,
    read_pos: usize,
    write_pos: usize,
}

impl DelayLine {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn process(&mut self, input: f32, feedback: f32) -> f32 {
        let delayed = self.buffer[self.read_pos];
        self.buffer[self.write_pos] = input + delayed * feedback;

        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        delayed
    }
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let delays = vec![
            (0.03 * sample_rate) as usize,
            (0.05 * sample_rate) as usize,
            (0.07 * sample_rate) as usize,
            (0.11 * sample_rate) as usize,
        ];

        let delay_lines = delays
            .into_iter()
            .map(DelayLine::new)
            .collect();

        Self {
            delay_lines,
            feedback: 0.6,
            wet_mix: 0.3,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;

        for delay_line in &mut self.delay_lines {
            output += delay_line.process(input, self.feedback);
        }

        output /= self.delay_lines.len() as f32;
        input * (1.0 - self.wet_mix) + output * self.wet_mix
    }
}

pub struct Filter {
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

impl Filter {
    pub fn new(filter_type: FilterType, cutoff: f32, resonance: f32, sample_rate: f32) -> Self {
        Self {
            filter_type,
            cutoff,
            resonance,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let frequency = self.cutoff / (self.sample_rate * 0.5);
        let frequency = frequency.max(0.001).min(0.999);
        
        let c = 1.0 / (2.0 * std::f32::consts::PI * frequency).tan();
        let a1 = 1.0 / (1.0 + self.resonance * c + c * c);
        let a2 = 2.0 * a1;
        let a3 = a1;
        let b1 = 2.0 * (1.0 - c * c) * a1;
        let b2 = (1.0 - self.resonance * c + c * c) * a1;

        let output = match self.filter_type {
            FilterType::LowPass => {
                a1 * input + a2 * self.x1 + a3 * self.x2 - b1 * self.y1 - b2 * self.y2
            }
            FilterType::HighPass => {
                a1 * c * c * (input - 2.0 * self.x1 + self.x2) - b1 * self.y1 - b2 * self.y2
            }
            FilterType::BandPass => {
                a1 * self.resonance * c * (input - self.x2) - b1 * self.y1 - b2 * self.y2
            }
        };

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }
}