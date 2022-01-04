use std::collections::HashMap;
use std::time::Duration;

use rust_gpiozero::LED;

use super::GPIOPin;

/// `LEDGroup` manages multiple LEDs and offers helper functions for interacting with them.
///
/// /// Provides a way to iterate over button press events:
///
/// ```
/// use std::time::Duration;
///
/// let mut leds = LEDGroup::new(&[17, 18]);
///
/// // Blink with built-in helper methods
/// leds.blink(17, 1, Duration::from_millis(500), Duration::from_millis(250));
/// leds.blink_all(3, Duration::from_millis(500), Duration::from_millis(250));
///
/// // Or use [`rust_gpiozero::LED`] methods directly:
/// if let Some(mut red) = leds.get_mut(17) {
///     println!("The red LED is on: {}", red.on());
/// }
/// ```
pub struct LEDGroup {
    leds: HashMap<u8, LED>,
}

impl LEDGroup {
    /// Create a new `LEDGroup` for the given GPIO pin numbers
    pub fn new(gpio_pins: &[GPIOPin]) -> Self {
        let leds = gpio_pins
            .iter()
            .map(|gpio_pin| (*gpio_pin, LED::new(*gpio_pin)))
            .collect();

        Self { leds }
    }

    /// Blink a single LED (with the given `gpio_pin`)
    /// - `blink_count`: number of times to blink
    /// - `on_time`: duration of time to stay lit
    /// - `off_time`: duration of time to turn off between blinks
    pub fn blink(
        &mut self,
        gpio_pin: GPIOPin,
        blink_count: i32,
        on_time: Duration,
        off_time: Duration,
    ) {
        if let Some(led) = self.leds.get_mut(&gpio_pin) {
            led.set_blink_count(blink_count);
            led.blink(on_time.as_secs_f32(), off_time.as_secs_f32());
        }
    }

    /// Blink all LEDs in this group
    /// - `blink_count`: number of times to blink
    /// - `on_time`: duration of time to stay lit
    /// - `off_time`: duration of time to turn off between blinks
    pub fn blink_all(
        &mut self,
        gpio_pins: &[GPIOPin],
        blink_count: i32,
        on_time: Duration,
        off_time: Duration,
    ) {
        for gpio_pin in gpio_pins.iter() {
            self.blink(*gpio_pin, blink_count, on_time, off_time);
        }
    }

    /// Get a reference to an LED in this group (by `gpio_pin`), if the LED is present
    pub fn get(&self, gpio_pin: GPIOPin) -> Option<&'_ LED> {
        self.leds.get(&gpio_pin)
    }

    /// Get a mutable reference to an LED in this group (by `gpio_pin`), if the LED is present
    pub fn get_mut(&mut self, gpio_pin: GPIOPin) -> Option<&'_ mut LED> {
        self.leds.get_mut(&gpio_pin)
    }
}

/// Make sure LEDs are turned off when this `LEDGroup` drops
impl Drop for LEDGroup {
    fn drop(&mut self) {
        for (_, led) in self.leds.iter_mut() {
            led.off();
        }
    }
}
