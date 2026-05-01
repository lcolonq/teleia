use crate::{state, utils};

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
}

impl Mode {
    pub fn new(frames: u64) -> Self {
        Self {
            frames,
            toggle: ModeToggle::Inactive { start: 0 },
        }
    }

    /// Is the current state active?
    pub fn is_active(&self) -> bool {
        match self.toggle {
            ModeToggle::Inactive {..} => false,
            ModeToggle::Active {..} => true,
        }
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
        self.toggle = ModeToggle::Inactive { start: 0 };
    }

    pub fn toggle(&mut self, tick: u64) {
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
    }
}

pub struct Cursor {
    pub index: i32,
    pub prev_index: i32,
    pub change_started: u64,
    pub bound: i32,
    pub frames: u64,
}

impl Cursor {
    pub fn new(bound: i32, frames: u64) -> Self {
        Self {
            index: 0,
            prev_index: 0,
            change_started: 0,
            bound,
            frames,
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
        if self.is_ready(tick) {
            self.change_started = tick;
            self.prev_index = self.index;
            self.index = val;
            self.index %= self.bound;
            true
        } else { false }
    }
    pub fn increment(&mut self, tick: u64) -> bool { self.set(self.index + 1, tick) }
    pub fn decrement(&mut self, tick: u64) -> bool { self.set(self.index + self.bound - 1, tick) }

    pub fn set_unlocked(&mut self, val: i32, tick: u64) {
        self.change_started = tick;
        self.prev_index = self.index;
        self.index = val;
        self.index %= self.bound;
    }
    pub fn increment_unlocked(&mut self, tick: u64) -> bool { self.set_unlocked(self.index + 1, tick); true }
    pub fn decrement_unlocked(&mut self, tick: u64) -> bool { self.set_unlocked(self.index + self.bound - 1, tick); true }

    /// Read keypresses to update this cursor (assuming that the left/right keys decrement/increment)
    /// Returns true if an update was performed (e.g. to determine whether to play a sound).
    pub fn update_horizontal(&mut self, st: &state::State) -> bool {
        if st.keys.new_left() {
            self.decrement_unlocked(st.tick);
            true
        } else if st.keys.new_right() {
            self.increment_unlocked(st.tick);
            true
        } else if st.keys.left() {
            self.decrement(st.tick)
        } else if st.keys.right() {
            self.increment(st.tick)
        } else { false }
    }

    pub fn update_vertical(&mut self, st: &state::State) -> bool {
        if st.keys.new_up() {
            self.decrement_unlocked(st.tick)
        } else if st.keys.new_down() {
            self.increment_unlocked(st.tick)
        } else if st.keys.up() {
            self.decrement(st.tick)
        } else if st.keys.down() {
            self.increment(st.tick)
        } else { false }
    }

    pub fn update_lr(&mut self, st: &state::State) -> bool {
        if st.keys.new_l() {
            self.decrement_unlocked(st.tick)
        } else if st.keys.new_r() {
            self.increment_unlocked(st.tick)
        } else if st.keys.l() {
            self.decrement(st.tick)
        } else if st.keys.r() {
            self.increment(st.tick)
        } else { false }
    }
}
