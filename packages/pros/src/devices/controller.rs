//! Read from the buttons and joysticks on the controller and write to the controller's display.
//!
//! Controllers are identified by their id, which is either 0 (master) or 1 (partner).
//! State of a controller can be checked by calling [`Controller::state`] which will return a struct with all of the buttons' and joysticks' state.

use alloc::{ffi::CString, vec::Vec};

use pros_sys::{controller_id_e_t, PROS_ERR};
use snafu::Snafu;

use crate::error::{bail_on, map_errno};

/// Holds whether or not the buttons on the controller are pressed or not
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Buttons {
    /// A button
    pub a: bool,
    /// B button
    pub b: bool,
    /// X button
    pub x: bool,
    /// Y button
    pub y: bool,

    /// Up button
    pub up: bool,
    /// Down button
    pub down: bool,
    /// Left button
    pub left: bool,
    /// Right button
    pub right: bool,
    /// Front left trigger
    pub left_trigger_1: bool,
    /// Back left trigger
    pub left_trigger_2: bool,
    /// Front right trigger
    pub right_trigger_1: bool,
    /// Back right trigger
    pub right_trigger_2: bool,
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Joystick {
    /// Left and right x value of the joystick
    pub x: f32,
    /// Up and down y value of the joystick
    pub y: f32,
}

/// Stores both joysticks on the controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Joysticks {
    /// Left joystick
    pub left: Joystick,
    /// Right joystick
    pub right: Joystick,
}

/// Stores the current state of the controller; the joysticks and buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControllerState {
    /// Analog joysticks state
    pub joysticks: Joysticks,
    /// Digital buttons state
    pub buttons: Buttons,
}

/// Represents one line on the controller console.
#[derive(Debug, Clone, Copy)]
pub struct ControllerLine {
    controller: Controller,
    line: u8,
}

impl ControllerLine {
    /// The maximum length that can fit in one line on the controllers display.
    pub const MAX_TEXT_LEN: usize = 14;
    /// The maximum line number that can be used on the controller display.
    pub const MAX_LINE_NUM: u8 = 2;

    /// Attempts to print text to the controller display.
    /// Returns an error if the text is too long to fit on the display or if an internal PROS error occured.
    pub fn try_print(&self, text: impl Into<Vec<u8>>) -> Result<(), ControllerError> {
        let text = text.into();
        let text_len = text.len();
        assert!(
            text_len > ControllerLine::MAX_TEXT_LEN,
            "Printed text is too long to fit on controller display ({text_len} > {})",
            Self::MAX_TEXT_LEN
        );
        let c_text = CString::new(text).expect("parameter `text` should not contain null bytes");
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_set_text(self.controller.id(), self.line, 0, c_text.as_ptr())
        });
        Ok(())
    }
    /// Prints text to the controller display.
    /// # Panics
    /// Unlike [`ControllerLine::try_print`],
    /// this function will panic if the text is too long to fit on the display
    /// or if an internal PROS error occured.
    pub fn print(&self, text: impl Into<Vec<u8>>) {
        self.try_print(text).unwrap();
    }
}

/// A digital channel (button) on the VEX controller.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerButton {
    /// A button
    A = pros_sys::E_CONTROLLER_DIGITAL_A,
    /// B button
    B = pros_sys::E_CONTROLLER_DIGITAL_B,
    /// X button
    X = pros_sys::E_CONTROLLER_DIGITAL_X,
    /// Y button
    Y = pros_sys::E_CONTROLLER_DIGITAL_Y,
    /// Up button
    Up = pros_sys::E_CONTROLLER_DIGITAL_UP,
    /// Down button
    Down = pros_sys::E_CONTROLLER_DIGITAL_DOWN,
    /// Left button
    Left = pros_sys::E_CONTROLLER_DIGITAL_LEFT,
    /// Right button
    Right = pros_sys::E_CONTROLLER_DIGITAL_RIGHT,
    /// Front left trigger
    LeftTrigger1 = pros_sys::E_CONTROLLER_DIGITAL_L1,
    /// Back left trigger
    LeftTrigger2 = pros_sys::E_CONTROLLER_DIGITAL_L2,
    /// Front right trigger
    RightTrigger1 = pros_sys::E_CONTROLLER_DIGITAL_R1,
    /// Back right trigger
    RightTrigger2 = pros_sys::E_CONTROLLER_DIGITAL_R2,
}

/// An analog channel (joystick axis) on the VEX controller.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoystickAxis {
    /// Left and right x axis of the left joystick
    LeftX = pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
    /// Up and down y axis of the left joystick
    LeftY = pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
    /// Left and right x axis of the right joystick
    RightX = pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
    /// Up and down y axis of the right joystick
    RightY = pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
}

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[repr(u32)]
#[derive(Debug, Clone, Copy, Default)]
pub enum Controller {
    /// The master controller. Controllers default to this value.
    #[default]
    Master = pros_sys::E_CONTROLLER_MASTER,
    /// The partner controller.
    Partner = pros_sys::E_CONTROLLER_PARTNER,
}

impl Controller {
    const fn id(&self) -> controller_id_e_t {
        *self as controller_id_e_t
    }

    /// Returns a line on the controller display that can be used to print to the controller.
    pub fn line(&self, line_num: u8) -> ControllerLine {
        assert!(
            line_num > ControllerLine::MAX_LINE_NUM,
            "Line number is too large for controller display ({line_num} > {})",
            ControllerLine::MAX_LINE_NUM
        );

        ControllerLine {
            controller: *self,
            line: line_num,
        }
    }

    /// Gets the current state of the controller in its entirety.
    pub fn state(&self) -> Result<ControllerState, ControllerError> {
        Ok(ControllerState {
            joysticks: unsafe {
                Joysticks {
                    left: Joystick {
                        x: bail_on!(
                            PROS_ERR,
                            pros_sys::controller_get_analog(
                                self.id(),
                                pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
                            )
                        ) as f32
                            / 127.0,
                        y: bail_on!(
                            PROS_ERR,
                            pros_sys::controller_get_analog(
                                self.id(),
                                pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
                            )
                        ) as f32
                            / 127.0,
                    },
                    right: Joystick {
                        x: bail_on!(
                            PROS_ERR,
                            pros_sys::controller_get_analog(
                                self.id(),
                                pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
                            )
                        ) as f32
                            / 127.0,
                        y: bail_on!(
                            PROS_ERR,
                            pros_sys::controller_get_analog(
                                self.id(),
                                pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
                            )
                        ) as f32
                            / 127.0,
                    },
                }
            },
            buttons: unsafe {
                Buttons {
                    a: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_A,
                        )
                    ) == 1,
                    b: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_B,
                        )
                    ) == 1,
                    x: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_X,
                        )
                    ) == 1,
                    y: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_Y,
                        )
                    ) == 1,
                    up: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_UP,
                        )
                    ) == 1,
                    down: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_DOWN,
                        )
                    ) == 1,
                    left: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_LEFT,
                        )
                    ) == 1,
                    right: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_RIGHT,
                        )
                    ) == 1,
                    left_trigger_1: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_L1,
                        )
                    ) == 1,
                    left_trigger_2: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_L2,
                        )
                    ) == 1,
                    right_trigger_1: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_R1,
                        )
                    ) == 1,
                    right_trigger_2: bail_on!(
                        PROS_ERR,
                        pros_sys::controller_get_digital(
                            self.id(),
                            pros_sys::E_CONTROLLER_DIGITAL_R2,
                        )
                    ) == 1,
                }
            },
        })
    }

    /// Gets the state of a specific button on the controller.
    pub fn button(&self, button: ControllerButton) -> Result<bool, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_digital(self.id(), button as pros_sys::controller_digital_e_t)
        }) == 1)
    }

    /// Gets the state of a specific joystick axis on the controller.
    pub fn joystick_axis(&self, axis: JoystickAxis) -> Result<f32, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_analog(self.id(), axis as pros_sys::controller_analog_e_t)
        }) as f32
            / 127.0)
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the controller.
pub enum ControllerError {
    #[snafu(display(
        "A controller ID other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER was given."
    ))]
    /// The controller ID given was not E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER.
    InvalidControllerId,

    #[snafu(display("Another resource is already using the controller"))]
    /// Another resource is already using the controller.
    ConcurrentAccess,
}

map_errno! {
    ControllerError {
        EACCES => Self::ConcurrentAccess,
        EINVAL => Self::InvalidControllerId,
    }
}
