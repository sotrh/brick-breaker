pub struct State {
    pub bricks: Vec<Brick>,
    pub player: Player,
    pub arena_size: glam::Vec2,
    pub brick_size: glam::Vec2,
}

impl State {
    pub fn new(
        arena_size: glam::Vec2,
        player_size: glam::Vec2,
        brick_size: glam::Vec2,
    ) -> Self {
        let player_pos = glam::vec2(arena_size.x * 0.5 - brick_size.x * 0.5, 0.0);
        Self {
            bricks: Vec::new(),
            player: Player {
                body: Body {
                    pos: player_pos,
                    size: player_size,
                },
                vel: glam::Vec2::ZERO,
            },
            arena_size,
            brick_size,
        }
    }

    pub fn setup_bricks(&mut self, num_x: u32, num_y: u32) {
        self.bricks.clear();
        let padding = self.arena_size.x - self.brick_size.x * num_x as f32;
        let start_x = padding * 0.5;
        for y in 0..num_y {
            for x in 0..num_x {
                self.bricks.push(Brick { 
                    body: Body {
                        pos: glam::vec2(start_x + x as f32 * self.brick_size.x, self.arena_size.y - y as f32 * self.brick_size.y - self.brick_size.y),
                        size: self.brick_size,
                    },
                    status: 4,
            });
            }
        }
    }

    pub fn remove_body(&mut self, index: usize) {
        self.bricks.remove(index);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Body {
    pub pos: glam::Vec2,
    pub size: glam::Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct Brick {
    pub body: Body,
    pub status: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub body: Body,
    pub vel: glam::Vec2,
}
