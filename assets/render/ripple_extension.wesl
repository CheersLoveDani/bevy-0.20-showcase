// Fragment shader for `ExtendedMaterial2d<ColorMaterial, Ripple>`
// (see src/demos/render/extended_material_2d.rs).

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::{globals, view};
// The base material's bindings stay available to the extension's shader.
import bevy_sprite_render::mesh2d::color_material::material as color_material;

@if(TONEMAP_IN_SHADER)
import bevy_core_pipeline::tonemapping;
@if(SRGB_OUTPUT)
import bevy_render::color_operations::linear_to_srgb;
@if(OKLAB_OUTPUT)
import bevy_render::color_operations::linear_rgb_to_oklab;

struct Ripple {
    ring_color: vec4<f32>,
    rings: f32,
    speed: f32,
}

// Binding 20, so it does not collide with `ColorMaterial`'s bindings.
@group(constants::MATERIAL_BIND_GROUP) @binding(20) var<uniform> ripple: Ripple;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // `ColorMaterial::color` comes from the base material ...
    let base = color_material.color;

    // ... and the extension draws rings travelling outwards from the centre of the mesh.
    let distance = length(mesh.uv - vec2(0.5)) * 2.0;
    let wave = sin((distance * ripple.rings - globals.time * ripple.speed) * 6.2831853);
    let ring = smoothstep(0.2, 0.9, wave);
    var color = vec4(mix(base.rgb, ripple.ring_color.rgb, ring * ripple.ring_color.a), base.a);

@if(TONEMAP_IN_SHADER)
    color = tonemapping::tone_mapping(color, view.color_grading);
@if(SRGB_OUTPUT)
    color = vec4(linear_to_srgb(color.rgb), color.a);
@if(OKLAB_OUTPUT)
    color = vec4(linear_rgb_to_oklab(color.rgb), color.a);

    return color;
}
