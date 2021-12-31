use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam::channel::{bounded, Receiver};
use rust_gpiozero::{Button, Debounce, Debounced, LED};

const GPIO_BUTTON_RED: u8 = 4;
const GPIO_LED_RED: u8 = 17;
const GPIO_BUTTON_GREEN: u8 = 5;
const GPIO_LED_GREEN: u8 = 23;

const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);

fn get_led_from_button(button_gpio: u8) -> u8 {
    match button_gpio {
        GPIO_BUTTON_RED => GPIO_LED_RED,
        GPIO_BUTTON_GREEN => GPIO_LED_GREEN,
        _ => panic!("Unknown Button GPIO: {}", button_gpio),
    }
}

fn play_game(length: usize, lights: &mut LEDGroup, buttons: &mut ButtonGroup) {
    let mut game = Game::new(length);
    loop {
        // dbg!(&game);
        // dbg!(&game.sequence[..=game.current_item]);
        for button_gpio in &game.sequence[..=game.current_item] {
            let led_gpio = get_led_from_button(*button_gpio);
            // dbg!(&led_gpio);
            lights.blink(led_gpio, 1, 0.4, 0.4);
            std::thread::sleep(Duration::from_millis(1000));
        }

        let mut answer: Vec<u8> = Vec::with_capacity(game.current_item + 1);

        println!("Waiting for press...");
        for press in &buttons {
            if answer.len() == game.current_item {
                break;
            }
            match press {
                ButtonPress::Single(GPIO_BUTTON_RED) => {
                    println!("RED");
                    answer.push(GPIO_BUTTON_RED);
                    lights.blink(GPIO_LED_RED, 1, 0.4, 0.1);
                }
                ButtonPress::Single(GPIO_BUTTON_GREEN) => {
                    println!("GREEN");
                    answer.push(GPIO_BUTTON_GREEN);
                    lights.blink(GPIO_LED_GREEN, 1, 0.4, 0.1);
                }
                ButtonPress::Single(_) => {}
                ButtonPress::Double => {
                    println!("Double press!");
                    lights.blink_all(&[GPIO_LED_RED, GPIO_LED_GREEN], 3, 0.15, 0.15);
                    return;
                }
            }
        }

        if game.matches(&answer) {
            if game.is_finished() {
                lights.blink(GPIO_LED_GREEN, 5, 0.200, 0.15);
                println!("You Won!");
                return;
            }
            game.current_item += 1;
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        } else {
            lights.blink(GPIO_LED_RED, 3, 0.15, 0.15);
            println!("You Lost!");
            return;
        }
    }
}

fn main() {
    let mut lights = LEDGroup::new(vec![GPIO_LED_RED, GPIO_LED_GREEN]);
    let mut buttons = ButtonGroup::new(vec![GPIO_BUTTON_RED, GPIO_BUTTON_GREEN]);

    loop {
        play_game(4, &mut lights, &mut buttons);
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
    fn new(gpio_pins: Vec<u8>) -> Self {
        let last_trigger: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let queue: Arc<Mutex<Option<ButtonPress>>> = Arc::new(Mutex::new(None));
        let (tx, rx) = bounded(1);

        let buttons = gpio_pins
            .into_iter()
            .map(|gpio_pin| {
                let tx = tx.clone();
                let lt = last_trigger.clone();
                let q = queue.clone();
                let mut button = Button::new(gpio_pin).debounce(Duration::from_millis(100));
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
                            q.lock().unwrap().replace(ButtonPress::Single(gpio_pin));
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
        self.rx.recv().map_or(None, |event| Some(event))
    }
}

struct LEDGroup {
    leds: HashMap<u8, LED>,
}

impl LEDGroup {
    pub fn new(gpio_pins: Vec<u8>) -> Self {
        let leds = gpio_pins
            .into_iter()
            .map(|gpio_pin| (gpio_pin, LED::new(gpio_pin)))
            .collect();

        Self { leds }
    }

    pub fn blink(&mut self, gpio_pin: u8, blink_count: i32, on_time: f32, off_time: f32) {
        self.leds.get_mut(&gpio_pin).map(|led| {
            led.set_blink_count(blink_count);
            led.blink(on_time, off_time);
        });
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
        Self {
            sequence: vec![
                GPIO_BUTTON_RED,
                GPIO_BUTTON_RED,
                GPIO_BUTTON_GREEN,
                GPIO_BUTTON_RED,
            ],
            current_item: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    pub fn is_finished(&self) -> bool {
        self.current_item == (self.len() - 1)
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
