use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Sender};
use rppal::gpio;
use rust_gpiozero::{Button, Debounce, Debounced, LED};

const GPIO_BUTTON_RED: u8 = 4;
const GPIO_BUTTON_GREEN: u8 = 5;

const DOUBLE_PRESS_THRESH: Duration = Duration::from_millis(30);

fn main() {
    let red_led = {
        let mut led = LED::new(17);
        led.set_blink_count(1);
        Arc::new(Mutex::new(led))
    };
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

    let (tx, events) = bounded(1);
    let _button_group = ButtonGroup::new(vec![GPIO_BUTTON_RED, GPIO_BUTTON_GREEN], tx);

    println!("Waiting for press...");

    for event in events {
        use ButtonPress::*;
        match event {
            Single(GPIO_BUTTON_RED) => {
                println!("RED");
                red_led.lock().unwrap().toggle();
            }
            Single(GPIO_BUTTON_GREEN) => {
                println!("GREEN");
                green_led.lock().unwrap().toggle();
            }
            Single(_) => {}
            Double => {
                red_led.lock().unwrap().off();
                green_led.lock().unwrap().off();
                println!("Double press!");
            }
        }
    }

    std::thread::sleep(Duration::from_secs(3600));
    println!("Game Over");
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
}

impl ButtonGroup {
    fn new(gpio_pins: Vec<u8>, sender: Sender<ButtonPress>) -> Self {
        let last_trigger: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let queue: Arc<Mutex<Option<ButtonPress>>> = Arc::new(Mutex::new(None));

        let buttons = gpio_pins
            .into_iter()
            .map(|gpio_pin| {
                let tx = sender.clone();
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
        }
    }
}
