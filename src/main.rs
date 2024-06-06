// Authors: Jason Cameron & Colin Cai
// Date: Mid May-Mid June
// Teacher: Mr Wong 
// Course: TEJ3M
// Description: This program controls a simulated intersection which includes automatic street lights, a parking lot with an automatic gate & a full extendable traffic lights.
#![no_std]
#![no_main]

use arduino_hal::{
    delay_ms,
    hal::port::Dynamic,
    port::{mode::Output, Pin},
    prelude::_unwrap_infallible_UnwrapInfallible,
    simple_pwm::*,
};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
use ufmt::uwriteln;

const YELLOW_TIME: u8 = 30;
const RED_TIME: u8 = 100;
const GREEN_TIME: u8 = 70;

struct TrafficLight {
    red: Pin<Output, Dynamic>,
    yellow: Pin<Output, Dynamic>,
    green: Pin<Output, Dynamic>,

    anim_timer: u8,
    anim_state: u8,
}

impl TrafficLight {
    fn new(
        red: Pin<Output, Dynamic>,
        yellow: Pin<Output, Dynamic>,
        green: Pin<Output, Dynamic>,
    ) -> Self {
        Self {
            red,
            yellow,
            green,
            anim_timer: 0,
            anim_state: 0,
        }
    }

    fn tick(&mut self) {
        if self.anim_timer > 0 {
            self.anim_timer -= 1;
            return;
        }
        
        self.next();
    }
    fn next(&mut self) {
        self.anim_state += 1;
        if self.anim_state == 3{
            self.anim_state = 0;
        }
        self.anim_timer = match self.anim_state {
            0 => {
                self.green.set_high();
                self.red.set_low();
                GREEN_TIME
            }
            1 => {
                self.yellow.set_high();
                self.green.set_low();
                YELLOW_TIME
            }
            2 => {
                self.red.set_high();
                self.yellow.set_low();
                RED_TIME
            }
            _ => unreachable!(),
        };
    }
    fn force_speedup_by(&mut self, mut speedup: u8) {
        loop{
            if self.anim_timer >= speedup {
                self.anim_timer -= speedup;
                break;
            }
            speedup -= self.anim_timer;
            self.next();
        }
    }
}

struct Timer {
    prev_res: bool,
    state: u8,
    reset: u8,
}
impl Timer {
    fn new(reset: u8) -> Self {
        Self { prev_res: false, state: 0, reset }
    }
    fn tick(&mut self) -> Option<bool> {
        let res = if self.state > 0 {
            self.state -= 1;
            true
        } else {
            false
        };
        let changed = self.prev_res != res;
        self.prev_res = res;
        changed.then_some(res)
    }
    fn pulse(&mut self) {
        self.state = self.reset;
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let streetlight_ldr = pins.a0.into_analog_input(&mut adc);
    let mut streetlight_led = pins.d2.into_output().downgrade();
    let mut gate_pht = pins.a5.into_analog_input(&mut adc);
    let timer0 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);
    let mut gate_servo = pins.d11.into_output().into_pwm(&timer0);
    let mut gate_servo_motion_timer = Timer::new(10);
    let mut gate_timer = Timer::new(40);

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
    t2.anim_timer = 100;

    let mut was_button_pressed = false; // init as false

    loop {
        let streetlight_read = streetlight_ldr.analog_read(&mut adc);
        let gate_read = gate_pht.analog_read(&mut adc);

        if gate_read > 100 {
            gate_timer.pulse();
        }

        uwriteln!(&mut serial, "gate {}", gate_read).unwrap_infallible();

        // streetlight_led
        //     .set_state((streetlight_read < 150).into())
        //     .unwrap_infallible();

        let is_button_pressed = pedestrian_button.is_high();
        if !was_button_pressed && is_button_pressed {
            t2.force_speedup_by(50);
            t1.force_speedup_by(50);
        }
        was_button_pressed = is_button_pressed;

        // gate_servo.

        t1.tick();
        t2.tick();
        if let Some(res) = gate_timer.tick() {
            gate_servo_motion_timer.pulse();
            if res {
                uwriteln!(&mut serial, "gate opened").unwrap_infallible();
                gate_servo.set_duty(250);
            } else {
                uwriteln!(&mut serial, "gate closed").unwrap_infallible();
                gate_servo.set_duty(100);
            }
        }
        if let Some(res) = gate_servo_motion_timer.tick() {
            if res {
                gate_servo.enable();
            } else {
                gate_servo.disable();
            }
        }

        delay_ms(50);
    }
}
