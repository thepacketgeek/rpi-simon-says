use std::env;
use std::time::Duration;

use rpi_simon_says::{consts::*, get_led_from_button, ButtonGroup, ButtonPress, Game, LEDGroup};

const DEFAULT_STARTING_LENGTH: usize = 5;

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
        match play_game(current_length, &mut lights, &mut buttons) {
            Some(true) => {
                std::thread::sleep(Duration::from_millis(300));
                println!("You Won!");
                lights.blink(GPIO_LED_GREEN, 5, 0.200, 0.15);
                current_length += 1;
            }
            Some(false) => {
                lights.blink(GPIO_LED_RED, 3, 0.15, 0.15);
                println!("You Lost!");
                current_length = starting_length;
            }
            None => {
                lights.blink_all(&GPIO_LEDS, 2, 0.3, 0.3);
                println!("starting over [{}]", current_length);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// Executes a round of Simon Says for the given length
fn play_game(length: usize, lights: &mut LEDGroup, buttons: &mut ButtonGroup) -> Option<bool> {
    let mut game = Game::new(length);
    loop {
        for button_gpio in game.current_sequence() {
            let led_gpio = get_led_from_button(*button_gpio);
            lights.blink(led_gpio, 1, 0.4, 0.4);
            std::thread::sleep(Duration::from_millis(1000));
        }

        let mut answer: Vec<u8> = Vec::with_capacity(game.current_len());

        println!(
            "Waiting for press... [{}/{}]",
            game.current_len() - 1,
            length
        );
        for press in &buttons {
            match press {
                ButtonPress::Single(GPIO_BUTTON_RED) => {
                    println!("RED");
                    answer.push(GPIO_BUTTON_RED);
                    lights.blink(GPIO_LED_RED, 1, 0.4, 0.01);
                }
                ButtonPress::Single(GPIO_BUTTON_GREEN) => {
                    println!("GREEN");
                    answer.push(GPIO_BUTTON_GREEN);
                    lights.blink(GPIO_LED_GREEN, 1, 0.4, 0.01);
                }
                ButtonPress::Single(GPIO_BUTTON_BLUE) => {
                    println!("BLUE");
                    answer.push(GPIO_BUTTON_BLUE);
                    lights.blink(GPIO_LED_BLUE, 1, 0.4, 0.01);
                }
                ButtonPress::Single(_) => {}
                ButtonPress::Double => {
                    println!("Double press!");
                    return None;
                }
            }
            if answer.len() == game.current_len() {
                break;
            }
            if !game.matches(&answer) {
                return Some(false);
            }
        }

        if game.matches(&answer) {
            if game.is_finished() {
                return Some(true);
            }
            game.advance();
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        } else {
            return Some(false);
        }
    }
}
