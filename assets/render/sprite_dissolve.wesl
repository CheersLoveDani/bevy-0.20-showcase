// Fragment shader for `DissolveMaterial`, a `SpriteMaterial` (see src/demos/render/sprite_material.rs).

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::{globals, view};
// The helpers Bevy 0.20 exports for sprite shaders.
import bevy_sprite_render::sprite_mesh::functions::{get_final_color, sample_sprite_texture};

@if(TONEMAP_IN_SHADER)
import bevy_core_pipeline::tonemapping;
@if(SRGB_OUTPUT)
import bevy_render::color_operations::linear_to_srgb;
@if(OKLAB_OUTPUT)
import bevy_render::color_operations::linear_rgb_to_oklab;

struct DissolveMaterial {
    edge_color: vec4<f32>,
    amount: f32,
    wobble: f32,
}

// Binding 20, so it does not collide with the sprite's own material bindings.
@group(constants::MATERIAL_BIND_GROUP) @binding(20) var<uniform> material: DissolveMaterial;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let t = smoothstep(vec2(0.0), vec2(1.0), fract(p));
    return mix(
        mix(hash(cell), hash(cell + vec2(1.0, 0.0)), t.x),
        mix(hash(cell + vec2(0.0, 1.0)), hash(cell + vec2(1.0, 1.0)), t.x),
        t.y,
    );
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the sprite's texture through a UV offset that ripples over time.
    var uv = mesh.uv;
    uv.x += sin(uv.y * 20.0 + globals.time * 4.0) * material.wobble;
    let texel = sample_sprite_texture(uv, mesh.instance_index);

    // Texels whose noise value is below `amount` are gone; the ones just above it glow.
    let noise = value_noise(mesh.uv * 10.0);
    let visible = step(material.amount, noise);
    let glow = (1.0 - smoothstep(0.0, 0.1, noise - material.amount)) * step(0.001, material.amount);
    let dissolved = vec4(mix(texel.rgb, material.edge_color.rgb, glow), texel.a * visible);

    // Applies `Sprite::color` and `Sprite::alpha_mode`, like the built-in sprite shader.
    var color = get_final_color(dissolved, mesh.instance_index);

@if(TONEMAP_IN_SHADER)
    color = tonemapping::tone_mapping(color, view.color_grading);
@if(SRGB_OUTPUT)
    color = vec4(linear_to_srgb(color.rgb), color.a);
@if(OKLAB_OUTPUT)
    color = vec4(linear_rgb_to_oklab(color.rgb), color.a);

    return color;
}
