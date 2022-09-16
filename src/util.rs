use winit::window::Fullscreen;

pub fn get_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: None,
    })
}

pub fn set_fullscreen(fullscreen: bool, window: &winit::window::Window) {
    if fullscreen {
        let m = window
            .current_monitor()
            .expect("System should have a display");
        let vm = m
            .video_modes()
            .next()
            .expect("Monitor should support video");
        window.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
    } else {
        window.set_fullscreen(None);
    }
}