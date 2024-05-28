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

    fn process(&mut self) {
        if self.anim_timer == 0xff {
            return;
        }

        match self.anim_timer {
            0 => {
                self.green.set_low();
                self.yellow.set_high();
                self.anim_timer += 1;
            }
            20 => {
                self.yellow.set_low();
                self.red.set_high();
                self.anim_timer += 1;
            }
            100 => {
                self.red.set_low();
                self.green.set_high();
                self.anim_timer = 0;
            }
            _ => {
                self.anim_timer += 1;
            }
        }
    }
    fn force_speedup(&mut self) {
        if self.anim_timer <= 20 {
            self.anim_timer = 20;
        } else if self.anim_timer <= 100 {
            self.anim_timer = 100;
        }
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

    
    let mut t2 = TrafficLight::new(
        pins.d10.into_output().downgrade(),
        pins.d9.into_output().downgrade(),
        pins.d8.into_output().downgrade(),
    );

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

        t1.process();
        t2.process();

        delay_ms(50);
    }
}
