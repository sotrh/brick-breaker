use crate::{
    input::Controller,
    state::{self, State},
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Win,
    Bounce,
    Drop,
    Fire,
}

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

    pub fn update(&self, state: &mut State, dt: f32, messages: &mut Vec<Message>) {
        state.player.vel.x = self.dir * self.speed;
        state.player.body.pos += state.player.vel * dt;

        if state.player.body.pos.x < 0.0 {
            state.player.body.pos.x = 0.0
        } else if state.player.body.pos.x > state.arena_size.x - state.player.body.size.x {
            state.player.body.pos.x = state.arena_size.x - state.player.body.size.x
        }

        if self.fire && !state.ball.fired && !state.game_just_started {
            state.ball.vel = glam::vec2(self.dir, 1.0).normalize() * self.speed * 0.5;
            state.ball.fired = true;
            messages.push(Message::Fire);
        } else if !state.ball.fired {
            state.ball.body.pos = state.player.body.pos
                + glam::vec2(
                    (state.player.body.size.x - state.ball.body.size.x) * 0.5,
                    1.0 + state.player.body.size.y,
                );
        }

        state.ball.body.pos += state.ball.vel * dt;

        // collision
        if state.ball.fired {
            let mut bounced = false;
            if collide(&state.ball.body, &state.player.body) {
                let paddle_rel_x = state.ball.body.pos.x - state.player.body.pos.x;
                state.ball.vel.x = paddle_rel_x / (state.player.body.size.x - state.ball.body.size.x) * 2.0 - 1.0;
                state.ball.vel.y = 2.0;
                state.ball.vel = state.ball.vel.normalize() * 0.5 * self.speed;
                bounced = true;
            }
    
            let mut bricks_to_remove = Vec::new();
            for (i, brick) in state.bricks.iter_mut().enumerate() {
                if collide(&state.ball.body, &brick.body) {
                    state.ball.body.pos.y = brick.body.pos.y - brick.body.size.y - state.ball.body.size.y;
                    state.ball.vel.y *= -1.0;
    
                    brick.status -= 1;
    
                    if brick.status <= 0 {
                        bricks_to_remove.push(i);
                    }
                    bounced = true;
                }
            }
    
            for i in bricks_to_remove {
                state.bricks.remove(i);
            }
    
            if state.ball.body.pos.x < 0.0 {
                state.ball.body.pos.x = 0.0;
                state.ball.vel.x *= -1.0;
                bounced = true;
            } else if state.ball.body.pos.x + state.ball.body.size.x > state.arena_size.x {
                state.ball.body.pos.x = state.arena_size.x - state.ball.body.size.x;
                state.ball.vel.x *= -1.0;
                bounced = true;
            }
            if state.ball.body.pos.y < 0.0 {
                messages.push(Message::Drop);
                bounced = false;
                state.ball.fired = false;
            } else if state.ball.body.pos.y + state.ball.body.size.y > state.arena_size.y {
                state.ball.body.pos.y = state.arena_size.y - state.ball.body.size.y;
                state.ball.vel.y *= -1.0;
                bounced = true;
            }
    
            if state.bricks.len() == 0 {
                messages.push(Message::Win);
            }
            if bounced {
                messages.push(Message::Bounce);
            }
        }

        state.game_just_started = false;
    }
}

fn collide(a: &state::Body, b: &state::Body) -> bool {
    a.pos.x < b.pos.x + b.size.x
        && a.pos.x + a.size.x > b.pos.x
        && a.pos.y < b.pos.y + b.size.y
        && a.pos.y + a.size.y > b.pos.y
}
