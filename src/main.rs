use std::env;
use std::time::Duration;

use rpi_simon_says::{ButtonGroup, ButtonPress, GPIOPin, LEDGroup, Round};

/// GPIOs for LEDs in use
pub const GPIO_BUTTON_RED: GPIOPin = 4;
pub const GPIO_BUTTON_GREEN: GPIOPin = 17;
pub const GPIO_BUTTON_BLUE: GPIOPin = 27;
pub const GPIO_LED_RED: GPIOPin = 12;
pub const GPIO_LED_GREEN: GPIOPin = 16;
pub const GPIO_LED_BLUE: GPIOPin = 18;

// Adding a 4th LED is as easy as adding the button/LED constants here and adding it
// to these const arrays, then adding a mapping in [`get_led_from_button`] below
pub const GPIO_LEDS: [GPIOPin; 3] = [GPIO_LED_RED, GPIO_LED_GREEN, GPIO_LED_BLUE];
pub const GPIO_BUTTONS: [GPIOPin; 3] = [GPIO_BUTTON_RED, GPIO_BUTTON_GREEN, GPIO_BUTTON_BLUE];

const DEFAULT_STARTING_LENGTH: usize = 5;
const BLINK_LONG: Duration = Duration::from_millis(450);
const BLINK_MED: Duration = Duration::from_millis(300);
const BLINK_SHORT: Duration = Duration::from_millis(150);
const TURN_DELAY: Duration = Duration::from_millis(1000);

/// Mapping of buttons to their corresponding LEDs
pub fn get_led_from_button(button_gpio: GPIOPin) -> GPIOPin {
    match button_gpio {
        GPIO_BUTTON_RED => GPIO_LED_RED,
        GPIO_BUTTON_GREEN => GPIO_LED_GREEN,
        GPIO_BUTTON_BLUE => GPIO_LED_BLUE,
        _ => panic!("Unknown Button GPIO: {}", button_gpio),
    }
}

fn main() {
    let starting_length = env::args()
        .skip(1)
        .map(|arg| arg.parse().expect("Invalid length provided"))
        .next()
        .unwrap_or(DEFAULT_STARTING_LENGTH);

    let mut lights = LEDGroup::new(&GPIO_LEDS);
    let mut buttons = ButtonGroup::new(&GPIO_BUTTONS);
    let mut current_length = starting_length;

    loop {
        match play_round(current_length, &mut lights, &mut buttons) {
            Some(true) => {
                std::thread::sleep(Duration::from_millis(300));
                println!("You Won!");
                lights.blink(GPIO_LED_GREEN, 5, BLINK_MED, BLINK_SHORT);
                current_length += 1;
            }
            Some(false) => {
                lights.blink(GPIO_LED_RED, 3, BLINK_SHORT, BLINK_SHORT);
                println!("You Lost!");
                current_length = starting_length;
            }
            None => {
                lights.blink_all(&GPIO_LEDS, 2, BLINK_MED, BLINK_SHORT);
                println!("starting over [{}]", current_length);
            }
        }
        std::thread::sleep(TURN_DELAY * 2);
    }
}

/// Executes a round of Simon Says for the given length
fn play_round(length: usize, lights: &mut LEDGroup, buttons: &mut ButtonGroup) -> Option<bool> {
    let mut round = Round::new(length, &GPIO_BUTTONS);
    loop {
        for button_gpio in round.current_sequence() {
            let led_gpio = get_led_from_button(*button_gpio);
            lights.blink(led_gpio, 1, BLINK_LONG, BLINK_LONG);
            std::thread::sleep(TURN_DELAY);
        }

        let mut answer: Vec<u8> = Vec::with_capacity(round.current_len());

        println!(
            "Waiting for press... [{}/{}]",
            round.current_len() - 1,
            length
        );
        for press in &buttons {
            match press {
                ButtonPress::Single(GPIO_BUTTON_RED) => {
                    println!("RED");
                    answer.push(GPIO_BUTTON_RED);
                    lights.blink(GPIO_LED_RED, 1, BLINK_LONG, BLINK_SHORT);
                }
                ButtonPress::Single(GPIO_BUTTON_GREEN) => {
                    println!("GREEN");
                    answer.push(GPIO_BUTTON_GREEN);
                    lights.blink(GPIO_LED_GREEN, 1, BLINK_LONG, BLINK_SHORT);
                }
                ButtonPress::Single(GPIO_BUTTON_BLUE) => {
                    println!("BLUE");
                    answer.push(GPIO_BUTTON_BLUE);
                    lights.blink(GPIO_LED_BLUE, 1, BLINK_LONG, BLINK_SHORT);
                }
                ButtonPress::Single(_) => {}
                ButtonPress::Double => {
                    println!("Double press!");
                    return None;
                }
            }
            if answer.len() == round.current_len() {
                break;
            }
            if !round.matches(&answer) {
                return Some(false);
            }
        }

        if round.matches(&answer) {
            if round.is_finished() {
                return Some(true);
            }
            round.advance();
            std::thread::sleep(TURN_DELAY);
            continue;
        } else {
            return Some(false);
        }
    }
}
