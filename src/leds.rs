use std::collections::HashMap;

use rust_gpiozero::LED;

pub struct LEDGroup {
    leds: HashMap<u8, LED>,
}

impl LEDGroup {
    pub fn new(gpio_pins: &[u8]) -> Self {
        let leds = gpio_pins
            .iter()
            .map(|gpio_pin| (*gpio_pin, LED::new(*gpio_pin)))
            .collect();

        Self { leds }
    }

    pub fn blink(&mut self, gpio_pin: u8, blink_count: i32, on_time: f32, off_time: f32) {
        if let Some(led) = self.leds.get_mut(&gpio_pin) {
            led.set_blink_count(blink_count);
            led.blink(on_time, off_time);
        }
    }

    pub fn blink_all(&mut self, gpio_pins: &[u8], blink_count: i32, on_time: f32, off_time: f32) {
        for gpio_pin in gpio_pins.iter() {
            self.blink(*gpio_pin, blink_count, on_time, off_time);
        }
    }
}

impl Drop for LEDGroup {
    fn drop(&mut self) {
        for (_, led) in self.leds.iter_mut() {
            led.off();
        }
    }
}
