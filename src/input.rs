use winit::event::{DeviceEvent, ElementState, VirtualKeyCode};

#[derive(Debug)]
pub enum Input {
    Device(DeviceEvent),
    KeyboardInput(VirtualKeyCode, bool),
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    left: Axis,
    right: Axis,
    up: Axis,
    down: Axis,
    fire: Axis,
    back: Axis,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            up: Default::default(),
            down: Default::default(),
            fire: Default::default(),
            back: Default::default(),
        }
    }

    #[allow(dead_code)]
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
                VirtualKeyCode::W | VirtualKeyCode::Up => self.up.set_digital(*pressed),
                VirtualKeyCode::S | VirtualKeyCode::Down => self.down.set_digital(*pressed),
                VirtualKeyCode::Space | VirtualKeyCode::Return => self.fire.set_digital(*pressed),
                VirtualKeyCode::Escape => self.back.set_digital(*pressed),
                _ => (),
            }
            Input::Device(DeviceEvent::Button { button, state }) => match (button, *state == ElementState::Pressed) {
                (0, pressed) => self.fire.set_digital(pressed),
                _ => (),
            },
            _ => (),
        }
    }

    pub fn dir(&self) -> f32 {
        self.right.value - self.left.value
    }

    #[allow(dead_code)]
    pub fn fire(&self) -> f32 {
        self.fire.value
    }

    pub fn fire_just_pressed(&self) -> bool {
        self.fire.new_input
    }

    pub fn back_just_pressed(&self) -> bool {
        self.back.new_input
    }

    pub fn up_just_pressed(&self) -> bool {
        self.up.new_input
    }

    pub fn down_just_pressed(&self) -> bool {
        self.down.new_input
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Axis {
    value: f32,
    new_input: bool,
}

impl Axis {
    #[allow(dead_code)]
    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.new_input = value > 0.0;
    }

    pub fn set_digital(&mut self, pressed: bool) {
        self.value = if pressed { 1.0 } else { 0.0 };
        self.new_input = pressed;
    }

    #[allow(dead_code)]
    pub fn press(&mut self) {
        self.set(1.0);
    }
    
    #[allow(dead_code)]
    pub fn release(&mut self) {
        self.set(0.0);
    }
}