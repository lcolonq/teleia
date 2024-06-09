use crate::utils;

fn compute_reverse(frames: u64, tick: u64, start: u64) -> u64 {
    let leftover = frames - (tick - start)
        .clamp(0, frames);
    tick - leftover
}

pub enum ModeToggle {
    Inactive { start: u64 },
    Active { start: u64 },
}

pub struct Mode {
    frames: u64,
    toggle: ModeToggle,
    locked: bool,
}

impl Mode {
    pub fn new(frames: u64) -> Self {
        Self {
            frames,
            toggle: ModeToggle::Inactive { start: 0 },
            locked: false,
        }
    }

    /// Is the current state active?
    pub fn is_active(&self) -> bool {
        match self.toggle {
            ModeToggle::Inactive {..} => false,
            ModeToggle::Active {..} => true,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Has the current transition finished?
    pub fn is_ready(&self, tick: u64) -> bool {
        let started = match self.toggle {
            ModeToggle::Inactive { start } => start,
            ModeToggle::Active { start } => start,
        };
        tick - started > self.frames
    }

    pub fn progress(&self, tick: u64) -> f32 {
        match self.toggle {
            ModeToggle::Inactive { start } => {
                1.0 - (((tick - start) as f32) / self.frames as f32)
                    .clamp(0.0, 1.0)
            },
            ModeToggle::Active { start } => {
                (((tick - start) as f32) / self.frames as f32)
                    .clamp(0.0, 1.0)
            }
        }
    }

    pub fn reset(&mut self) {
        self.locked = false;
        self.toggle = ModeToggle::Inactive { start: 0 };
    }

    pub fn reverse(&mut self, tick: u64) -> bool {
        if !self.locked {
            self.locked = true;
            match self.toggle {
                ModeToggle::Inactive { start } => {
                    self.toggle = ModeToggle::Active {
                        start: compute_reverse(self.frames, tick, start)
                    };
                },
                ModeToggle::Active { start } => {
                    self.toggle = ModeToggle::Inactive {
                        start: compute_reverse(self.frames, tick, start)
                    };
                },
            }
            true
        } else { false }
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

pub struct Cursor {
    pub index: i32,
    pub prev_index: i32,
    pub change_started: u64,
    pub bound: i32,
    pub frames: u64,
    pub locked: bool,
}

impl Cursor {
    pub fn new(bound: i32, frames: u64) -> Self {
        Self {
            index: 0,
            prev_index: 0,
            change_started: 0,
            bound,
            frames,
            locked: false,
        }
    }

    pub fn animation_index(&self, tick: u64) -> f32 {
        let progress = ((tick - self.change_started) as f32)
            / (self.frames as f32 / 2.0);
        utils::lerp(
            self.prev_index as f32,
            self.index as f32,
            progress
        )
    }

    pub fn is_ready(&self, tick: u64) -> bool {
        tick - self.change_started > self.frames
    }

    pub fn set(&mut self, val: i32, tick: u64) -> bool {
        if self.is_ready(tick) || !self.locked {
            self.change_started = tick;
            self.prev_index = self.index;
            self.index = val;
            self.index %= self.bound;
            self.locked = true;
            true
        } else { false }
    }

    pub fn increment(&mut self, tick: u64) -> bool {
        self.set(self.index + 1, tick)
    }

    pub fn decrement(&mut self, tick: u64) -> bool {
        self.set(self.index + self.bound - 1, tick)
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }
}
