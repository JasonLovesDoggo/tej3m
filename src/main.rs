#![no_std]
#![no_main]

use arduino_hal::{
    delay_ms,
    hal::port::Dynamic,
    port::{mode::Output, Pin},
    prelude::_unwrap_infallible_UnwrapInfallible,
};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

const YELLOW_TIME: u8 = 80;
const RED_TIME: u8 = 100;
const GREEN_TIME: u8 = 0;
const RESET_TIME: u8 = (RED_TIME - (RED_TIME - YELLOW_TIME)) * 2; // ecodistant

struct TrafficLight {
    red: Pin<Output, Dynamic>,
    yellow: Pin<Output, Dynamic>,
    green: Pin<Output, Dynamic>,

    anim_timer: u8,
}

impl TrafficLight {
    fn new(
        red: Pin<Output, Dynamic>,
        yellow: Pin<Output, Dynamic>,
        mut green: Pin<Output, Dynamic>,
    ) -> Self {
        green.set_high();
        Self {
            red,
            yellow,
            green,
            anim_timer: 0,
        }
    }

    fn sync_with(&mut self, master: &TrafficLight) {
        if master.green.is_set_high() {
            self.green.set_low();
            self.yellow.set_low();
            self.red.set_high();
        } else {
            if master.anim_timer >= YELLOW_TIME {
                // set to red
                self.green.set_low();
                self.yellow.set_low();
                self.red.set_high();
            } else {
                self.green.set_high();
                self.yellow.set_low();
                self.red.set_low();
            }
        }
    }

    fn process(&mut self) {
        if self.anim_timer == 0xff {
            return;
        }
        match self.anim_timer {
            GREEN_TIME => {
                self.green.set_high();
                self.yellow.set_low();
                self.red.set_low();
                self.anim_timer += 1;
            }
            YELLOW_TIME => {
                self.green.set_low();
                self.yellow.set_high();
                self.red.set_low();
                self.anim_timer += 1;
            }
            RED_TIME => {
                self.green.set_low();
                self.yellow.set_low();
                self.red.set_high();
                self.anim_timer += 1;
            }
            RESET_TIME => {
                self.anim_timer = GREEN_TIME;
            }
            _ => {
                self.anim_timer += 1;
            }
        }
    }
    fn force_speedup(&mut self) {
        // if self.anim_timer > RED_TIME {
        // return // don't speed up if it's not in red stage
        // }
        self.anim_timer = RESET_TIME - 5; // speed up 
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let streetlight_ldr = pins.a0.into_analog_input(&mut adc);
    let mut streetlight_led = pins.d2.into_output().downgrade();

    let pedestrian_button = pins.d6.into_pull_up_input();

    let mut t1 = TrafficLight::new(
        pins.d3.into_output().downgrade(),
        pins.d4.into_output().downgrade(),
        pins.d5.into_output().downgrade(),
    );

    // let mut t2 = TrafficLight::new(
    //     pins.d10.into_output().downgrade(),
    //     pins.d9.into_output().downgrade(),
    //     pins.d8.into_output().downgrade(),
    // );

    let mut was_button_pressed = false; // init as false

    loop {
        let read = streetlight_ldr.analog_read(&mut adc);
        // uwriteln!(&mut serial, "Read {}   \r", read).unwrap_infallible();

        // Streetlight LED begins
        streetlight_led
            .set_state((read < 150).into())
            .unwrap_infallible();

        // End of streetlight LED

        let is_button_pressed = pedestrian_button.is_high();
        if !was_button_pressed && is_button_pressed {
            t1.force_speedup(); // it's only placed for the straight one
                                // t2.force_speedup();
        }
        was_button_pressed = is_button_pressed;

        t1.process(); // master (straight) traffic light process
                      // t2.sync_with(&t1);

        delay_ms(50);
    }
}
