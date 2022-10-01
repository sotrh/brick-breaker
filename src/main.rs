mod render;
mod state;
mod util;
mod system;
mod input;
mod menu;
mod sound;

use std::{io::Write, fs::read_to_string};

use util::*;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{render::{BoxRenderer, TextureAtlas}, menu::Menu};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    fullscreen: bool,
    width: u32,
    height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            width: 800,
            height: 600,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    MainMenu,
    Quitting,
}

async fn run() -> Result<(), anyhow::Error> {
    let mut settings = match std::fs::read_to_string("settings.json") {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    };

    let ev_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Brick Breaker")
        .with_visible(false)
        .build(&ev_loop)?;

    if settings.fullscreen {
        set_fullscreen(true, &window);
    } else {
        window.set_inner_size(LogicalSize::new(settings.width, settings.height));
    }

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .enumerate_adapters(wgpu::Backends::all())
        .filter(|a| a.is_surface_supported(&surface))
        .next()
        .expect("Platform should have a supported adapter");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        )
        .await?;

    let format = *surface
        .get_supported_formats(&adapter)
        .first()
        .expect("Platform should have supported texture formats");

    let size = window.inner_size();
    let mut surf_cfg = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
    };
    surface.configure(&device, &surf_cfg);

    let texture_atlas = TextureAtlas::with_json(&device, &queue, "./assets/atlas.json")?;
    let screen_size = glam::vec2(80.0, 80.0);
    let box_renderer =
        BoxRenderer::new(&device, surf_cfg.format, screen_size, &texture_atlas)?;
    let mut controller = input::Controller::new();
    let mut game_state = state::State::new(
        screen_size,
        texture_atlas.get_sprite("player").unwrap().size,
        texture_atlas.get_sprite("ball").unwrap().size,
        texture_atlas.get_sprite("brick1").unwrap().size,
    );
    let mut movement = system::MovementSystem::new(10.0);
    let mut game_messages = Vec::new();

    let mut menu_up = true;
    let mut menu_messages = Vec::new();
    let mut menu = Menu::new(&texture_atlas, screen_size);

    let sound_config = read_to_string("./assets/sounds.json")?;
    let mut sound_system = sound::SoundSystem::with_json(&sound_config)?;

    window.set_visible(true);
    ev_loop.run(move |ev, _, control_flow| match ev {
        Event::NewEvents(_) => (),
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                surf_cfg.width = size.width;
                surf_cfg.height = size.height;
                settings.width = size.width;
                settings.height = size.height;
                surface.configure(&device, &surf_cfg);
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match (key, state == ElementState::Pressed) {
                (VirtualKeyCode::F11, true) => {
                    settings.fullscreen = !settings.fullscreen;
                    set_fullscreen(settings.fullscreen, &window);
                }
                (key, pressed) => {
                    controller.input(&input::Input::KeyboardInput(key, pressed));
                    if menu_up {
                        menu.input(&controller, &mut menu_messages);
                        for msg in menu_messages.drain(..) {
                            match msg {
                                menu::Message::Exit => *control_flow = ControlFlow::Exit,
                                menu::Message::Start => {
                                    menu_up = false;
                                    game_state.setup(10, 4);
                                } 
                                menu::Message::ToggleFullscreen => {
                                    settings.fullscreen = !settings.fullscreen;
                                    set_fullscreen(settings.fullscreen, &window);
                                }
                                menu::Message::FocusChanged => {
                                    sound_system.play_sound("bounce");
                                }
                            }
                        }
                    }
                    else if controller.back_just_pressed() {
                        menu_up = true;
                    }
                },
            },
            _ => (),
        },
        Event::DeviceEvent { event, .. } => {
            controller.input(&input::Input::Device(event))
        },
        Event::UserEvent(_) => (),
        Event::Suspended => (),
        Event::Resumed => (),
        Event::MainEventsCleared => {}
        Event::RedrawRequested(_) => {
            window.request_redraw();
            if !menu_up {
                movement.input(&controller);
                movement.update(&mut game_state, 1.0 / 60.0, &mut game_messages);
                for msg in game_messages.drain(..) {
                    match msg {
                        system::Message::Win => {
                            menu_up = true;
                            sound_system.play_sound("win")
                        }
                        system::Message::Fire => sound_system.play_sound("fire"),
                        system::Message::Bounce => sound_system.play_sound("bounce"),
                        system::Message::Drop => sound_system.play_sound("fail"),
                    }
                }
            }
            
            match surface.get_current_texture() {
                Ok(tex) => {
                    let mesh = if menu_up {
                        menu.layout(&device, &settings)
                    } else {
                        box_renderer.mesh_from_state(&device, &game_state, &texture_atlas)
                    };
                    let view = tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = get_encoder(&device);
                    {
                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });
                        box_renderer.draw_mesh(&mut pass, &mesh);
                    }
                    queue.submit(Some(encoder.finish()));
                    tex.present();
                }
                Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Timeout) => (),
                e => {
                    log::error!("An error occured while getting render texture: {:?}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
        }
        Event::RedrawEventsCleared => (),
        Event::LoopDestroyed => {
            let mut file = std::fs::File::create("./settings.json").unwrap();
            let contents = serde_json::to_string_pretty(&settings).unwrap();
            write!(&mut file, "{}", contents).unwrap();
        }
    });
}

fn main() {
    env_logger::init();
    pollster::block_on(run()).unwrap();
}
