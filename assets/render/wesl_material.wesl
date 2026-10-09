// Fragment shader for `WeslMaterial` (see src/demos/render/wesl_material.rs).

// Modules embedded in Bevy's crates are imported by crate and file path.
import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::globals;
// Shaders loaded from the assets folder are modules too, named after their asset path.
// This file is `render/wesl_material.wesl`, so `super` is `render/`.
import super::wesl_patterns::{rainbow, stripes};

// `constants::NAME` reads an integer shader def.
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> base_color: vec4<f32>;

// `@if` / `@else` choose between whole declarations when the shader is compiled.
// RAINBOW_STRIPES is a shader def pushed by `WeslMaterial::specialize`.
@if(RAINBOW_STRIPES)
fn surface_color(world_position: vec3<f32>) -> vec3<f32> {
    let height = world_position.y - globals.time * 0.4;
    return mix(base_color.rgb, rainbow(height * 0.35), stripes(height, 2.5));
}
@else
fn surface_color(world_position: vec3<f32>) -> vec3<f32> {
    return base_color.rgb;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // One fixed light is enough to make the shapes read as 3D.
    let to_light = normalize(vec3(0.4, 0.8, 0.5));
    let light = 0.25 + 0.75 * max(dot(normalize(in.world_normal), to_light), 0.0);
    return vec4(surface_color(in.world_position.xyz) * light, 1.0);
}
