use std::time::Duration;

pub const GPIO_BUTTON_RED: u8 = 4;
pub const GPIO_LED_RED: u8 = 17;
pub const GPIO_BUTTON_GREEN: u8 = 16;
pub const GPIO_LED_GREEN: u8 = 12;
pub const GPIO_BUTTON_BLUE: u8 = 27;
pub const GPIO_LED_BLUE: u8 = 18;

pub const GPIO_LEDS: [u8; 3] = [GPIO_LED_RED, GPIO_LED_GREEN, GPIO_LED_BLUE];
pub const GPIO_BUTTONS: [u8; 3] = [GPIO_BUTTON_RED, GPIO_BUTTON_GREEN, GPIO_BUTTON_BLUE];

pub const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);
pub const DEBOUNCE_THRESH: Duration = Duration::from_millis(100);

/// Mapping of buttons to their corresponding LEDs
pub fn get_led_from_button(button_gpio: u8) -> u8 {
    match button_gpio {
        GPIO_BUTTON_RED => GPIO_LED_RED,
        GPIO_BUTTON_GREEN => GPIO_LED_GREEN,
        GPIO_BUTTON_BLUE => GPIO_LED_BLUE,
        _ => panic!("Unknown Button GPIO: {}", button_gpio),
    }
}
