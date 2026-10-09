// A small library module. It has no entry point of its own:
// `wesl_material.wesl` imports these functions with `import super::wesl_patterns::{...};`.

const TAU: f32 = 6.2831853;

/// A smooth rainbow that repeats every time `t` grows by one.
fn rainbow(t: f32) -> vec3<f32> {
    return vec3(0.5) + 0.5 * cos(TAU * (vec3(t) + vec3(0.0, 0.33, 0.67)));
}

/// 1.0 inside a stripe and 0.0 between stripes, `frequency` stripes per unit.
fn stripes(position: f32, frequency: f32) -> f32 {
    return smoothstep(0.4, 0.6, 0.5 + 0.5 * sin(position * frequency * TAU));
}
