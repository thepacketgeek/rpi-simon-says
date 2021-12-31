use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam::channel::{bounded, Receiver};
use rust_gpiozero::{Button, Debounce, Debounced};

use super::*;

pub struct ButtonGroup {
    _buttons: Vec<Debounced>,
    _last_trigger: Arc<Mutex<Option<Instant>>>,
    _queue: Arc<Mutex<Option<ButtonPress>>>,
    rx: Receiver<ButtonPress>,
}

impl ButtonGroup {
    pub fn new(gpio_pins: &'static [u8]) -> Self {
        let last_trigger: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let queue: Arc<Mutex<Option<ButtonPress>>> = Arc::new(Mutex::new(None));
        let (tx, rx) = bounded(1);

        let buttons = gpio_pins
            .iter()
            .map(|gpio_pin| {
                let tx = tx.clone();
                let lt = last_trigger.clone();
                let q = queue.clone();
                let mut button = Button::new(*gpio_pin).debounce(DEBOUNCE_THRESH);
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
                            // If last_pressed is non-existant, use something longer than the doublepress thresh
                            // so a single press will always be eligible
                            .unwrap_or_else(|| Instant::now() - (DOUBLE_PRESS_THRESH * 2))
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
