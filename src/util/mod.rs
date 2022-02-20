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

const WORD_LEN: usize = 4;

pub struct HexCursor {
    pub pos: (usize, usize),
}

impl HexCursor {
    pub fn new(pos: (usize, usize)) -> HexCursor {
        HexCursor { pos }
    }

    pub fn up(&mut self) {
        let y = (self.pos.1 as i64) - 1;
        if y < 0 {
            self.pos = (0, 0)
        } else {
            self.pos.1 = y as usize;
        }
    }

    pub fn down(&mut self, filesize: usize) {
        self.pos.1 += 1;
        if (self.pos.1 * 0x10) + (self.pos.0 / 2) > filesize {
            self.pos = ((filesize % 0x10) * 2, filesize / 0x10);
            self.left();
        }
    }

    pub fn left(&mut self) {
        if self.pos.0 == 0 {
            self.pos.0 = 0x1F;
            self.up();
        } else {
            self.pos.0 -= 1;
        }
    }

    pub fn right(&mut self, filesize: usize) {
        if ((self.pos.0 + 1) / 2) + (self.pos.1 * 0x10) == filesize {
            return;
        }

        if self.pos.0 == 0x1F {
            self.pos.0 = 0;
            self.down(filesize);
        } else {
            self.pos.0 += 1;
        }
    }

    pub fn next_word(&mut self, filesize: usize) {
        const WORD_LEN: usize = 4;

        let new_loc = (self.loc() + WORD_LEN) & !(WORD_LEN - 1);
        let new_loc = usize::min(new_loc, filesize - 1);

        let y = new_loc / 0x10;
        let x = (new_loc % 0x10) * 2;

        self.pos = (x, y);
    }

    pub fn prev_word(&mut self) {
        let new_loc = match self.loc() {
            loc if loc % WORD_LEN == 0 => loc.saturating_sub(WORD_LEN),
            loc => loc & !(WORD_LEN - 1),
        };

        let y = new_loc / 0x10;
        let x = (new_loc % 0x10) * 2;

        self.pos = (x, y);
    }

    pub fn goto(&mut self, loc: usize) {
        self.pos = ((loc % 0x10) * 2, loc / 0x10)
    }

    pub fn loc(&self) -> usize {
        (self.pos.0 / 2) + (self.pos.1 * 0x10)
    }
}
