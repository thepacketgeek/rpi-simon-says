use std::time::Duration;

use rand::prelude::SliceRandom;

mod buttons;
pub use buttons::ButtonGroup;

pub mod consts;
use consts::*;

mod leds;
pub use leds::LEDGroup;

#[derive(Debug)]
pub enum ButtonPress {
    Single(u8),
    Double,
}

#[derive(Debug)]
pub struct Game {
    /// GPIO button pins
    sequence: Vec<u8>,
    current_item: usize,
}

impl Game {
    pub fn new(length: usize) -> Self {
        let mut sequence = Vec::with_capacity(length);

        let mut rng = rand::thread_rng();

        for _ in 0..length {
            let item = GPIO_BUTTONS
                .choose(&mut rng)
                .copied()
                .unwrap_or(GPIO_BUTTON_RED);
            sequence.push(item);
        }

        Self {
            sequence,
            current_item: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    pub fn is_finished(&self) -> bool {
        self.current_item == (self.len() - 1)
    }

    pub fn advance(&mut self) {
        self.current_item += 1;
    }

    pub fn current_len(&self) -> usize {
        self.current_item + 1
    }

    pub fn current_sequence(&self) -> &[u8] {
        &self.sequence[..=self.current_item]
    }

    pub fn matches(&self, answer: &[u8]) -> bool {
        for (i, button_gpio) in answer.iter().enumerate() {
            if button_gpio != &self.sequence[i] {
                return false;
            }
        }
        true
    }
}
