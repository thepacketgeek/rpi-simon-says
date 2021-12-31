use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rust_gpiozero::{Button, LED, Debounce};

fn main() {
    let mut red_button = Button::new(4).debounce(Duration::from_millis(500));
    let red_led = {
        let mut led = LED::new(17);
        led.set_blink_count(1);
        Arc::new(Mutex::new(led))
    };
    let mut green_button = Button::new(5).debounce(Duration::from_millis(500));
    let green_led = {
        let mut led = LED::new(23);
        led.set_blink_count(1);
        Arc::new(Mutex::new(led))
    };

    {
        let red_led = red_led.clone();
        let green_led = green_led.clone();
        ctrlc::set_handler(move || {
            println!("Shutting down...");
            red_led.lock().unwrap().off();
            green_led.lock().unwrap().off();
            std::process::exit(0);
        })
        .unwrap();
    }

    println!("Waiting for press...");
    {
        let led = red_led.clone();
        red_button
            .when_pressed(move |_level| {
                led.lock().unwrap().blink(0.3, 0.1);
            }).unwrap();
        let led = green_led.clone();
        green_button
            .when_pressed(move |_level| {
                println!("pressed green");
                led.lock().unwrap().blink(0.3, 0.1);
            }).unwrap();
    }

    std::thread::sleep(Duration::from_secs(3600));
    println!("Game Over");
}
