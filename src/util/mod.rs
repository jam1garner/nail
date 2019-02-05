pub mod event;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;

#[derive(Clone)]
pub struct RandomSignal {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomSignal {
    pub fn new(lower: u64, upper: u64) -> RandomSignal {
        RandomSignal {
            distribution: Uniform::new(lower, upper),
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

#[derive(Clone)]
pub struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinSignal {
        SinSignal {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct HexCursor {
    pub pos : (usize, usize),
}

impl HexCursor {
    pub fn new(pos: (usize, usize)) -> HexCursor {
        HexCursor { pos }
    }

    pub fn up(&mut self){
        let y = (self.pos.1 as i64) - 1;
        if y < 0 {
            self.pos = (0,0)
        }
        else {
            self.pos.1 = y as usize;
        }
    }

    pub fn down(&mut self, filesize: usize){
        self.pos.1 += 1;
        if (self.pos.1 * 0x10) + (self.pos.0 / 2) > filesize {
            self.pos = ((filesize % 0x10) * 2, filesize / 10);
        }
    }

    pub fn left(&mut self){
        if self.pos.0 == 0 {
            self.pos.0 = 0x1F;
            self.up();
        }
        else {
            self.pos.0 -= 1;
        }
    }

    pub fn right(&mut self, filesize: usize){
        if ((self.pos.0 + 1) / 2) + (self.pos.1 * 0x10) == filesize {
            return;
        }
        
        if self.pos.0 == 0x1F {
            self.pos.0 = 0;
            self.down(filesize)
        }
        else {
            self.pos.0 += 1;
        }
    }
}
