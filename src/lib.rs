use std::time::Duration;

use rand::prelude::SliceRandom;

mod buttons;
pub use buttons::ButtonGroup;

mod leds;
pub use leds::LEDGroup;

/// Represents a BCM pin number
pub type GPIOPin = u8;

pub const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);
pub const DEBOUNCE_THRESH: Duration = Duration::from_millis(100);

/// An event emitted when button(s) are pressed, either with the pressed button GPIO or
/// a doublepress event
#[derive(Debug)]
pub enum ButtonPress {
    /// A single button was pressed (w/ corresponding `GPIOPin`)
    Single(GPIOPin),
    /// At least two buttons were pressed at the ~same time
    Double,
}

/// State for each complete round of Simon Says
///
/// ```
/// let mut round = Round::new(length, &[4, 5]);
///
/// let mut answer: Vec<u8> = Vec::with_capacity(round.current_len());

/// // ... Code to blink & receive button press events, calling round.advance() with each turn
///
/// if round.matches(&answer) && round.is_finished() {
///     println!("You won!");
/// }
/// ```
#[derive(Debug)]
pub struct Round {
    /// GPIO button pins
    sequence: Vec<GPIOPin>,
    current_item: usize,
}

impl Round {
    /// Create a new game round with the given length
    /// - creates a random sequence using the available Buttons
    pub fn new(length: usize, available_gpio_buttons: &[GPIOPin]) -> Self {
        let mut sequence = Vec::with_capacity(length);

        let mut rng = rand::thread_rng();

        assert!(
            available_gpio_buttons.len() > 0,
            "At least one Button GPIOPin must be provided"
        );

        for _ in 0..length {
            let item = available_gpio_buttons
                .choose(&mut rng)
                .copied()
                .unwrap_or(available_gpio_buttons[0]);
            sequence.push(item);
        }

        Self {
            sequence,
            current_item: 0,
        }
    }

    /// The sequence length (number of guesses needed) of this game round
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    /// Has this round been exhausted?
    pub fn is_finished(&self) -> bool {
        self.current_item == (self.len() - 1)
    }

    /// Progress the current item after a successful guess
    pub fn advance(&mut self) {
        self.current_item += 1;
    }

    /// The current sequence length (will increase as round advances)
    pub fn current_len(&self) -> usize {
        self.current_item + 1
    }

    /// The current sequence (to compare to user guesses)
    pub fn current_sequence(&self) -> &[GPIOPin] {
        &self.sequence[..=self.current_item]
    }

    /// Does the given answer match the sequence (only tests up to the answer length)
    pub fn matches(&self, answer: &[GPIOPin]) -> bool {
        for (i, button_gpio) in answer.iter().enumerate() {
            if button_gpio != &self.sequence[i] {
                return false;
            }
        }
        true
    }
}
