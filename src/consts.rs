use std::time::Duration;

pub const GPIO_BUTTON_RED: u8 = 4;
pub const GPIO_LED_RED: u8 = 17;
pub const GPIO_BUTTON_GREEN: u8 = 5;
pub const GPIO_LED_GREEN: u8 = 23;

pub const GPIO_LEDS: [u8; 2] = [GPIO_LED_RED, GPIO_LED_GREEN];
pub const GPIO_BUTTONS: [u8; 2] = [GPIO_BUTTON_RED, GPIO_BUTTON_GREEN];

pub const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);
pub const DEBOUNCE_THRESH: Duration = Duration::from_millis(100);