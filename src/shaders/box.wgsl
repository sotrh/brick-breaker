struct BoxVertex {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct Uniforms {
    camera: mat4x4<f32>,
}

struct VSOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) frag_pos: vec4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var tex: texture_2d<f32>;

@group(0)
@binding(2)
var samp: sampler;

@vertex
fn vs_main(in: BoxVertex) -> VSOut {
    let frag_pos = uniforms.camera * vec4(in.position, 0.0, 1.0);
    return VSOut(in.uv, frag_pos);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}