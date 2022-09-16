use std::cell::Cell;

use crate::{state::State, input::Controller};

pub struct MovementSystem {
    speed: f32,
    dir: f32,
    fire: bool,
}

impl MovementSystem {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            dir: 0.0,
            fire: false,
        }
    }

    pub fn input(&mut self, controller: &Controller) {
        self.dir = controller.dir();
        self.fire = controller.fire_just_pressed();
    }

    pub fn update(&self, state: &mut State, dt: f32) {
        state.player.vel.x = self.dir * self.speed;
        state.player.body.pos += state.player.vel * dt;

        if state.player.body.pos.x < 0.0 {
            state.player.body.pos.x = 0.0
        } else if state.player.body.pos.x > state.arena_size.x - state.brick_size.x {
            state.player.body.pos.x = state.arena_size.x - state.brick_size.x
        }

    }
}