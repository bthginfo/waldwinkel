use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Ready, garden::Garden};

const MORNING: u16 = 6 * 60;
const AFTERNOON: u16 = 12 * 60;
const DUSK: u16 = 19 * 60;
const MARKET_CLOSE: u16 = 20 * 60;
const NIGHT: u16 = 22 * 60;
const MIDNIGHT: u16 = 24 * 60 - 1;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FarmClock(f32);

impl Default for FarmClock {
    fn default() -> Self {
        Self(f32::from(MORNING))
    }
}

impl FarmClock {
    pub fn minute(self) -> u16 {
        self.0 as u16
    }

    pub fn night(self) -> bool {
        self.minute() >= NIGHT
    }

    pub fn market_open(self) -> bool {
        self.minute() < MARKET_CLOSE
    }

    pub fn dusk(self) -> f32 {
        ((self.0 - f32::from(DUSK)) / f32::from(NIGHT - DUSK)).clamp(0., 1.)
    }

    pub fn label(self) -> &'static str {
        match self.minute() {
            NIGHT.. => "Nacht",
            DUSK.. => "Abend",
            AFTERNOON.. => "Nachmittag",
            _ => "Vormittag",
        }
    }

    pub fn display(self) -> String {
        let minute = self.minute();
        format!("{:02}:{:02}", minute / 60, minute % 60)
    }

    pub fn restore(&mut self) {
        self.0 = self.0.clamp(f32::from(MORNING), f32::from(MIDNIGHT));
    }

    fn advance(&mut self, seconds: f32) {
        self.0 = (self.0 + seconds).min(f32::from(MIDNIGHT));
    }
}

pub fn tick(time: Res<Time>, ready: Res<Ready>, mut garden: ResMut<Garden>) {
    if ready.0 && garden.clock.minute() < MIDNIGHT {
        garden.clock.advance(time.delta_secs());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_closes_the_market_then_pauses_until_sleep() {
        let mut garden = Garden::default();
        garden.clock.advance(14. * 60.);
        assert!(!garden.clock.market_open());
        assert!(!garden.clock.night());
        garden.clock.advance(4. * 60. + 13.);
        assert_eq!(garden.clock.display(), "23:59");
        assert_eq!(garden.day, 1);
        garden.sleep();
        assert_eq!(garden.clock.display(), "06:00");
        assert_eq!(garden.day, 2);
    }
}
