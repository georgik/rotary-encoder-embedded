//! # rotary-encoder
//! A rotary encoder library built for embedded applications

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use embedded_hal::digital::v2::InputPin;

/// Velocity type, the value is between 0.0 and 1.0
pub type Velocity = f32;

/// Direction of Rotary Encoder rotation
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// No Direction is specified,
    None,
    /// Clockwise direction
    Clockwise,
    /// Anti-clockwise direction
    Anticlockwise,
}
/// The Sensitivity of the Rotary Encoder
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sensitivity {
    /// Default sensitivity
    Default = 2,
    /// Low sensitivity
    Low = 4,
}

/// State table for recognizing valid rotary encoder values
const STATES: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];

#[cfg(feature = "angular-velocity")]
/// Default angular velocity increasing factor
const DEFAULT_VELOCITY_INC_FACTOR: f32 = 0.2;
#[cfg(feature = "angular-velocity")]
/// Default angular velocity decreasing factor
const DEFAULT_VELOCITY_DEC_FACTOR: f32 = 0.01;
#[cfg(feature = "angular-velocity")]
/// Angular velocity action window duration in milliseconds
const DEFAULT_VELOCITY_ACTION_MS: u64 = 25;

/// Rotary Encoder
pub struct RotaryEncoder<DT, CLK> {
    pin_dt: DT,
    pin_clk: CLK,
    pos_calc: i8,
    sensitivity: Sensitivity,
    transition: u8,
    direction: Direction,
}

#[cfg(feature = "angular-velocity")]
/// Rotary Encoder with velocity
pub struct RotaryEncoderWithVelocity<DT, CLK> {
    inner: RotaryEncoder<DT, CLK>,
    velocity: Velocity,
    velocity_inc_factor: f32,
    velocity_dec_factor: f32,
    velocity_action_ms: u64,
    previous_time: u64,
}

impl<DT, CLK> RotaryEncoder<DT, CLK>
where
    DT: InputPin,
    CLK: InputPin,
{
    /// Initiates a new Rotary Encoder, taking two InputPins [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html).
    pub fn new(pin_dt: DT, pin_clk: CLK) -> Self {
        RotaryEncoder {
            pin_dt,
            pin_clk,
            pos_calc: 0,
            transition: 0,
            sensitivity: Sensitivity::Default,
            direction: Direction::None,
        }
    }

    /// Set the sensitivity of the rotary encoder
    pub fn set_sensitivity(&mut self, sensitivity: Sensitivity) {
        self.sensitivity = sensitivity;
    }

    /// Borrow a mutable reference to the underlying InputPins. This is useful for clearing hardware interrupts.
    pub fn borrow_pins(&mut self) -> (&mut DT, &mut CLK) {
        (&mut self.pin_dt, &mut self.pin_clk)
    }

    /// Release the underying resources such as the InputPins back to the initiator
    pub fn release(self) -> (DT, CLK) {
        (self.pin_dt, self.pin_clk)
    }

    /// Update the state machine of the RotaryEncoder. This should be called ideally from an interrupt vector
    /// when either the DT or CLK pins state changes. This function will update the RotaryEncoder's Direction
    pub fn update(&mut self) {
        let dt_state = self.pin_dt.is_high().unwrap_or_default() as u8;
        let clk_state = self.pin_clk.is_high().unwrap_or_default() as u8;

        let current = (dt_state << 1) | clk_state;
        self.transition = (self.transition << 2) | current;
        let index = (self.transition & 0x0F) as usize;
        self.pos_calc += STATES[index];

        let sensitivity = self.sensitivity as i8;
        if self.pos_calc == sensitivity || self.pos_calc == -sensitivity {
            self.direction = if self.pos_calc == sensitivity {
                Direction::Clockwise
            } else {
                Direction::Anticlockwise
            };

            self.pos_calc = 0;
            return;
        }

        self.direction = Direction::None;
    }

    /// Returns the current Direction of the RotaryEncoder
    pub fn direction(&self) -> Direction {
        self.direction
    }
}

#[cfg(feature = "angular-velocity")]
impl<DT, CLK> RotaryEncoderWithVelocity<DT, CLK>
where
    DT: InputPin,
    CLK: InputPin,
{
    /// Initiates a new Rotary Encoder with velocity, taking two InputPins [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html).
    /// Optionally the behaviour of the angular velocity can be modified:
    pub fn new(pin_dt: DT, pin_clk: CLK) -> Self {
        RotaryEncoderWithVelocity {
            inner: RotaryEncoder::new(pin_dt, pin_clk),
            velocity: 0.0,
            velocity_inc_factor: DEFAULT_VELOCITY_INC_FACTOR,
            velocity_dec_factor: DEFAULT_VELOCITY_DEC_FACTOR,
            velocity_action_ms: DEFAULT_VELOCITY_ACTION_MS,
            previous_time: 0,
        }
    }

    /// Set the velocity_inc_factor. How quickly the velocity increases to 1.0.
    pub fn set_velocity_inc_factor(&mut self, inc_factor: f32) {
        self.velocity_inc_factor = inc_factor;
    }

    /// Set the velocity_dec_factor. How quickly the velocity decreases or cools-down
    pub fn set_velocity_dec_factor(&mut self, dec_factor: f32) {
        self.velocity_dec_factor = dec_factor;
    }

    /// Set the velocity_action_ms. The window of duration (milliseconds) that the velocity will increase
    pub fn set_velocity_action_ms(&mut self, action_ms: u64) {
        self.velocity_action_ms = action_ms;
    }

    /// This function should be called periodically, either via a timer or the main loop.
    /// This function will reduce the angular velocity over time, the amount is configurable via the constructor
    pub fn decay_velocity(&mut self) {
        self.velocity -= self.velocity_dec_factor;
        if self.velocity < 0.0 {
            self.velocity = 0.0;
        }
    }

    /// Borrow a mutable reference to the underlying InputPins. This is useful for clearing hardware interrupts.
    pub fn borrow_pins(&mut self) -> (&mut DT, &mut CLK) {
        self.inner.borrow_pins()
    }

    /// Set the sensitivity of the rotary encoder
    pub fn set_sensitivity(&mut self, sensitivity: Sensitivity) {
        self.inner.sensitivity = sensitivity;
    }

    /// Borrow a reference to the underlying RotaryEncoder. Useful for configuring the RotaryEncoder
    pub fn borrow_inner(&mut self) -> &mut RotaryEncoder<DT, CLK> {
        &mut self.inner
    }

    /// Release the underying resources such as the InputPins back to the initiator
    pub fn release(self) -> (DT, CLK) {
        self.inner.release()
    }

    /// Update the state machine of the RotaryEncoder. This should be called ideally from an interrupt vector
    /// when either the DT or CLK pins state changes. This function will update the RotaryEncoder's
    /// Direction and current Angular Velocity.
    /// * `current_time` - Current timestamp in ms (strictly monotonously increasing)
    pub fn update(&mut self, current_time: u64) {
        self.inner.update();

        if self.inner.direction() != Direction::None {
            if current_time - self.previous_time < self.velocity_action_ms && self.velocity < 1.0 {
                self.velocity += self.velocity_inc_factor;
                if self.velocity > 1.0 {
                    self.velocity = 1.0;
                }
            }
            return;
        }

        self.previous_time = current_time;
    }

    /// Returns the current Direction of the RotaryEncoder
    pub fn direction(&self) -> Direction {
        self.inner.direction
    }

    /// Returns the current angular velocity of the RotaryEncoder
    /// The Angular Velocity is a value between 0.0 and 1.0
    /// This is useful for incrementing/decrementing a value in an exponential fashion
    pub fn velocity(&self) -> Velocity {
        self.velocity
    }
}
