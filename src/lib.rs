use rand::prelude::SliceRandom;

mod buttons;
pub use buttons::ButtonGroup;

pub mod consts;
pub use consts::*;

mod leds;
pub use leds::LEDGroup;

/// An event emitted when button(s) are pressed, either with the button GPIO or
/// a doublepress event
#[derive(Debug)]
pub enum ButtonPress {
    Single(u8),
    Double,
}

/// State for each complete round of Simon Says
#[derive(Debug)]
pub struct Game {
    /// GPIO button pins
    sequence: Vec<u8>,
    current_item: usize,
}

impl Game {
    /// Create a new game round with the given length
    /// - creates a random sequence using the available Buttons
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

    /// The length of this game round
    #[allow(clippy::len_without_is_empty)]
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
