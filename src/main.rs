use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam::channel::{bounded, Receiver};
use rand::prelude::SliceRandom;
use rust_gpiozero::{Button, Debounce, Debounced, LED};

const GPIO_BUTTON_RED: u8 = 4;
const GPIO_LED_RED: u8 = 17;
const GPIO_BUTTON_GREEN: u8 = 5;
const GPIO_LED_GREEN: u8 = 23;

const GPIO_LEDS: [u8; 2] = [GPIO_LED_RED, GPIO_LED_GREEN];
const GPIO_BUTTONS: [u8; 2] = [GPIO_BUTTON_RED, GPIO_BUTTON_GREEN];

const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);

fn get_led_from_button(button_gpio: u8) -> u8 {
    match button_gpio {
        GPIO_BUTTON_RED => GPIO_LED_RED,
        GPIO_BUTTON_GREEN => GPIO_LED_GREEN,
        _ => panic!("Unknown Button GPIO: {}", button_gpio),
    }
}

fn play_game(length: usize, lights: &mut LEDGroup, buttons: &mut ButtonGroup) -> Option<bool> {
    let mut game = Game::new(length);
    loop {
        for button_gpio in game.current_sequence() {
            let led_gpio = get_led_from_button(*button_gpio);
            lights.blink(led_gpio, 1, 0.4, 0.4);
            std::thread::sleep(Duration::from_millis(1000));
        }

        let mut answer: Vec<u8> = Vec::with_capacity(game.current_len());

        println!("Waiting for press...");
        for press in &buttons {
            match press {
                ButtonPress::Single(GPIO_BUTTON_RED) => {
                    println!("RED");
                    answer.push(GPIO_BUTTON_RED);
                    lights.blink(GPIO_LED_RED, 1, 0.4, 0.3);
                }
                ButtonPress::Single(GPIO_BUTTON_GREEN) => {
                    println!("GREEN");
                    answer.push(GPIO_BUTTON_GREEN);
                    lights.blink(GPIO_LED_GREEN, 1, 0.4, 0.3);
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

fn main() {
    let length = env::args()
        .skip(1)
        .map(|arg| arg.parse().expect("Invalid length provided"))
        .next()
        .unwrap_or(8);

    let mut lights = LEDGroup::new(&GPIO_LEDS);
    let mut buttons = ButtonGroup::new(&GPIO_BUTTONS);

    loop {
        match play_game(length, &mut lights, &mut buttons) {
            Some(true) => {
                println!("You Won!");
                lights.blink(GPIO_LED_GREEN, 5, 0.200, 0.15);
            }
            Some(false) => {
                lights.blink(GPIO_LED_RED, 3, 0.15, 0.15);
                println!("You Lost!");
            }
            None => {
                lights.blink_all(&GPIO_LEDS, 2, 0.3, 0.3);
                println!("starting over");
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[derive(Debug)]
enum ButtonPress {
    Single(u8),
    Double,
}

struct ButtonGroup {
    _buttons: Vec<Debounced>,
    _last_trigger: Arc<Mutex<Option<Instant>>>,
    _queue: Arc<Mutex<Option<ButtonPress>>>,
    rx: Receiver<ButtonPress>,
}

impl ButtonGroup {
    fn new(gpio_pins: &'static [u8]) -> Self {
        let last_trigger: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let queue: Arc<Mutex<Option<ButtonPress>>> = Arc::new(Mutex::new(None));
        let (tx, rx) = bounded(1);

        let buttons = gpio_pins
            .iter()
            .map(|gpio_pin| {
                let tx = tx.clone();
                let lt = last_trigger.clone();
                let q = queue.clone();
                let mut button = Button::new(*gpio_pin).debounce(Duration::from_millis(100));
                button
                    .when_pressed(move |_level| {
                        /* Check for double press by seeing if another button was pressed recently
                           (last_trigger hasn't been updated for this button yet, so a recent press has to be another button)

                           In order to prevent the first push of a double press going through, use an intermediate queue
                           that is slightly delayed, and replace the queued singlepress with a doublepress
                        */
                        let last_pressed_elapsed = lt
                            .lock()
                            .unwrap()
                            .unwrap_or_else(|| Instant::now() - Duration::from_millis(500))
                            .elapsed();
                        if last_pressed_elapsed < DOUBLE_PRESS_THRESH {
                            q.lock().unwrap().replace(ButtonPress::Double);
                        } else {
                            q.lock().unwrap().replace(ButtonPress::Single(*gpio_pin));
                        }
                        (*lt.lock().unwrap()).replace(Instant::now());

                        let nq = q.clone();
                        let ntx = tx.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(50));
                            if let Some(event) = nq.lock().unwrap().take() {
                                ntx.send(event).unwrap();
                            }
                        });
                    })
                    .unwrap();
                button
            })
            .collect();

        Self {
            _buttons: buttons,
            _last_trigger: last_trigger,
            _queue: queue,
            rx,
        }
    }
}

impl Iterator for &&mut ButtonGroup {
    type Item = ButtonPress;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}

struct LEDGroup {
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

#[derive(Debug)]
struct Game {
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
