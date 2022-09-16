use winit::event::{DeviceEvent, KeyboardInput, ElementState, VirtualKeyCode};

#[derive(Debug)]
pub enum Input {
    Device(DeviceEvent),
    KeyboardInput(VirtualKeyCode, bool),
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    left: Axis,
    right: Axis,
    fire: Axis,
    back: Axis,
    cursor_pos: glam::Vec2,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            fire: Default::default(),
            back: Default::default(),
            cursor_pos: glam::Vec2::ZERO,
        }
    }

    pub fn reset(&mut self) {
        self.left.new_input = false;
        self.right.new_input = false;
        self.fire.new_input = false;
        self.back.new_input = false;
    }

    pub fn input(&mut self, event: &Input) {
        match event {
            Input::KeyboardInput(key, pressed) => match key {
                VirtualKeyCode::A | VirtualKeyCode::Left => self.left.set_digital(*pressed),
                VirtualKeyCode::D | VirtualKeyCode::Right => self.right.set_digital(*pressed),
                VirtualKeyCode::Space | VirtualKeyCode::Up => self.fire.set_digital(*pressed),
                VirtualKeyCode::Escape => self.back.set_digital(*pressed),
                _ => (),
            }
            Input::Device(DeviceEvent::Button { button, state }) => {
                
            }
            _ => (),
        }
    }

    pub fn dir(&self) -> f32 {
        self.right.value - self.left.value
    }

    pub fn fire(&self) -> f32 {
        self.fire.value
    }

    pub fn fire_just_pressed(&self) -> bool {
        self.fire.new_input
    }

    pub fn back_just_pressed(&self) -> bool {
        self.back.new_input
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Axis {
    value: f32,
    new_input: bool,
}

impl Axis {
    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.new_input = value > 0.0;
    }

    pub fn set_digital(&mut self, pressed: bool) {
        self.value = if pressed { 1.0 } else { 0.0 };
        self.new_input = pressed;
    }

    pub fn press(&mut self) {
        self.set(1.0);
    }

    pub fn release(&mut self) {
        self.set(0.0);
    }
}