#[derive(Clone, Copy, Debug)]
pub struct Drill {
    started: f32,
    last_pull: Option<f32>,
    progress: u8,
    needed: u8,
    strain: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pull {
    Gained,
    Wait,
    Strained,
    Lost,
    Landed,
}

impl Drill {
    pub fn new(now: f32, deep: bool) -> Self {
        Self { started: now, last_pull: None, progress: 0, needed: if deep { 4 } else { 3 }, strain: 0 }
    }

    pub fn tension(self, now: f32) -> f32 {
        let time = (now - self.started).max(0.);
        let pace = if self.needed == 4 { 3.3 } else { 4.1 };
        let surge = ((time / pace * std::f32::consts::TAU - 0.85).sin() * 0.5 + 0.5).powi(2);
        let pull = self.last_pull.map_or(0., |last| (1. - (now - last) / 1.35).clamp(0., 1.)) * 0.41;
        (0.17 + surge * 0.66 + pull).min(1.)
    }

    pub fn counts(self) -> (u8, u8) {
        (self.progress, self.needed)
    }

    pub fn pull(&mut self, now: f32) -> Pull {
        if self.last_pull.is_some_and(|last| now - last < 0.33) {
            return Pull::Wait;
        }
        let strain = self.tension(now);
        self.last_pull = Some(now);
        if strain > 0.64 {
            self.strain += 1;
            return if self.strain >= 3 { Pull::Lost } else { Pull::Strained };
        }
        self.progress += 1;
        if self.progress >= self.needed { Pull::Landed } else { Pull::Gained }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fish_can_be_landed_after_safe_pulls_and_force_can_break_the_line() {
        let mut drill = Drill::new(0., false);
        assert_eq!(drill.pull(0.), Pull::Gained);
        assert_eq!(drill.pull(0.08), Pull::Wait);
        assert_eq!(drill.pull(4.1), Pull::Gained);
        assert_eq!(drill.pull(8.2), Pull::Landed);
        let mut drill = Drill::new(0., true);
        assert_eq!(drill.pull(1.27), Pull::Strained);
        assert_eq!(drill.pull(4.57), Pull::Strained);
        assert_eq!(drill.pull(7.87), Pull::Lost);
    }
}
