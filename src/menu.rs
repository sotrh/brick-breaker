use crate::{
    input,
    render::{self, Sprite},
    Settings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Start,
    Exit,
    Fullscreen,
}

pub struct Menu {
    focus: Focus,
    title: Sprite,
    start: Button,
    exit: Button,
    fullscreen: Toggle,
    screen_size: glam::Vec2,
    atlas_size: glam::Vec2,
}

impl Menu {
    pub fn new(atlas: &render::TextureAtlas, screen_size: glam::Vec2) -> Self {
        let atlas_size = atlas.size();
        let title = *atlas.get_sprite("title").unwrap();
        let start = Button::new(
            *atlas.get_sprite("start_button").unwrap(),
            *atlas.get_sprite("start_button_alt").unwrap(),
        );
        let exit = Button::new(
            *atlas.get_sprite("exit_button").unwrap(),
            *atlas.get_sprite("exit_button_alt").unwrap(),
        );
        let fullscreen = Toggle::new(
            *atlas.get_sprite("fullscreen").unwrap(),
            *atlas.get_sprite("fullscreen_alt").unwrap(),
            *atlas.get_sprite("check_box").unwrap(),
            *atlas.get_sprite("check_box_alt").unwrap(),
        );

        Self {
            focus: Focus::Start,
            atlas_size,
            screen_size,
            title,
            start,
            exit,
            fullscreen,
        }
    }

    pub fn input(&mut self, controller: &input::Controller, messages: &mut Vec<Message>) {
        if controller.down_just_pressed() {
            self.focus = match self.focus {
                Focus::Start => Focus::Exit,
                Focus::Exit => Focus::Fullscreen,
                Focus::Fullscreen => Focus::Start,
            }
        }
        if controller.up_just_pressed() {
            self.focus = match self.focus {
                Focus::Start => Focus::Fullscreen,
                Focus::Exit => Focus::Start,
                Focus::Fullscreen => Focus::Exit,
            }
        }

        if controller.fire_just_pressed() {
            match self.focus {
                Focus::Start => messages.push(Message::Start),
                Focus::Exit => messages.push(Message::Exit),
                Focus::Fullscreen => messages.push(Message::ToggleFullscreen),
            }
        }
    }

    pub fn layout(&self, device: &wgpu::Device, settings: &Settings) -> render::Mesh {
        let padding = 4.0;
        let mut layout =
            TopDownLayout::new(glam::vec2(padding, self.screen_size.y - padding), padding);

        let data = vec![
            (layout.place(self.title.size), &self.title),
            (
                layout.place_with_offset_x(self.start.selected.size, padding),
                if self.focus == Focus::Start {
                    &self.start.selected
                } else {
                    &self.start.unselected
                },
            ),
            (
                layout.place_with_offset_x(self.exit.selected.size, padding),
                if self.focus == Focus::Exit {
                    &self.exit.selected
                } else {
                    &self.exit.unselected
                },
            ),
            (
                glam::vec2(padding, padding),
                if self.focus == Focus::Fullscreen {
                    &self.fullscreen.selected
                } else {
                    &self.fullscreen.unselected
                },
            ),
            (
                glam::vec2(padding * 2.0 + self.fullscreen.selected.size.x, padding),
                if settings.fullscreen {
                    &self.fullscreen.checked
                } else {
                    &self.fullscreen.unchecked
                }
            )
        ];

        render::Mesh::from_sprites_with_positions(device, self.atlas_size, &data)
    }
}

pub struct Button {
    selected: Sprite,
    unselected: Sprite,
}

impl Button {
    pub fn new(selected: Sprite, unselected: Sprite) -> Self {
        Self {
            selected,
            unselected,
        }
    }
}

pub struct Toggle {
    selected: Sprite,
    unselected: Sprite,
    checked: Sprite,
    unchecked: Sprite,
}

impl Toggle {
    pub fn new(selected: Sprite, deselected: Sprite, checked: Sprite, unchecked: Sprite) -> Self {
        Self {
            selected,
            unselected: deselected,
            checked,
            unchecked,
        }
    }
}

pub enum Message {
    Start,
    Exit,
    ToggleFullscreen,
}

pub struct TopDownLayout {
    cursor: glam::Vec2,
    padding: f32,
}

impl TopDownLayout {
    pub fn new(start: glam::Vec2, padding: f32) -> Self {
        Self {
            cursor: start,
            padding,
        }
    }

    pub fn place(&mut self, sprite_size: glam::Vec2) -> glam::Vec2 {
        self.cursor.y -= sprite_size.y;
        let out = self.cursor;
        self.cursor.y -= self.padding;
        out
    }

    pub fn place_with_offset(&mut self, sprite_size: glam::Vec2, offset: glam::Vec2) -> glam::Vec2 {
        self.place(sprite_size) + offset
    }

    pub fn place_with_offset_x(&mut self, sprite_size: glam::Vec2, offsetx: f32) -> glam::Vec2 {
        self.place_with_offset(sprite_size, glam::vec2(offsetx, 0.0))
    }

    pub fn place_with_offset_y(&mut self, sprite_size: glam::Vec2, offset_y: f32) -> glam::Vec2 {
        self.place_with_offset(sprite_size, glam::vec2(0.0, offset_y))
    }
}
