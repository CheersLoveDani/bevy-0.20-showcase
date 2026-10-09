// A field of animated columns built entirely on the GPU: there is no vertex or index buffer.
//
//   task shader      runs once and decides how many mesh shader workgroups to launch
//   mesh shader      runs once per column and writes its 24 vertices and 12 triangles
//   fragment shader  shades the result as usual
import bevy_render::globals::Globals;
import bevy_render::view::View;

enable wgpu_mesh_shader;

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> view: View;

// Columns per side of the grid.
const GRID: u32 = 28u;
const SUN_DIRECTION = vec3(0.42, 0.85, 0.32);
const QUAD_CORNERS = array(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));

// Data handed from the task shader to every mesh shader it launches. Keep it small.
struct TaskPayload {
    phase: f32,
}

var<task_payload> payload: TaskPayload;

@task
@payload(payload)
@workgroup_size(1)
fn task() -> @builtin(mesh_task_size) vec3<u32> {
    payload.phase = globals.time * 2.0;
    // One mesh shader workgroup per column.
    return vec3(GRID, 1u, GRID);
}

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct Primitive {
    @builtin(triangle_indices) indices: vec3<u32>,
    // Per-primitive outputs are constant across a triangle, which gives flat shading for free.
    @per_primitive @location(1) light: f32,
}

struct MeshOutput {
    @builtin(vertices) vertices: array<Vertex, 24>,
    @builtin(primitives) primitives: array<Primitive, 12>,
    @builtin(vertex_count) vertex_count: u32,
    @builtin(primitive_count) primitive_count: u32,
}

var<workgroup> mesh_output: MeshOutput;

@mesh(mesh_output)
@payload(payload)
@workgroup_size(1)
fn mesh(@builtin(workgroup_id) cell: vec3<u32>) {
    mesh_output.vertex_count = 24u;
    mesh_output.primitive_count = 12u;

    // The column's height follows a ripple spreading out from the middle of the grid.
    let grid_position = vec2<f32>(cell.xz) + 0.5 - f32(GRID) / 2.0;
    let ripple = 0.5 + 0.5 * sin(length(grid_position) * 0.7 - payload.phase);
    let height = 0.3 + 2.5 * ripple;
    let center = vec3(grid_position.x, height / 2.0, grid_position.y);
    let half_size = vec3(0.4, height / 2.0, 0.4);
    let color = mix(vec3(0.05, 0.2, 0.9), vec3(1.0, 0.4, 0.05), ripple);

    for (var face = 0u; face < 6u; face++) {
        // Faces come in pairs along each axis: -X, +X, -Y, +Y, -Z, +Z.
        let axis = face / 2u;
        let side = f32(face % 2u) * 2.0 - 1.0;
        var normal = vec3(0.0);
        var tangent = vec3(0.0);
        var bitangent = vec3(0.0);
        normal[axis] = side;
        tangent[(axis + 1u) % 3u] = side;
        bitangent[(axis + 2u) % 3u] = 1.0;

        let first_vertex = face * 4u;
        for (var corner = 0u; corner < 4u; corner++) {
            let uv = QUAD_CORNERS[corner];
            let local = normal + tangent * uv.x + bitangent * uv.y;
            let world = center + local * half_size;
            let vertex = first_vertex + corner;
            mesh_output.vertices[vertex].position = view.clip_from_world * vec4(world, 1.0);
            // Darker towards the base of the column.
            mesh_output.vertices[vertex].color = color * mix(0.35, 1.0, local.y * 0.5 + 0.5);
        }

        let light = 0.3 + 0.7 * max(dot(normal, SUN_DIRECTION), 0.0);
        let first_primitive = face * 2u;
        mesh_output.primitives[first_primitive].indices =
            vec3(first_vertex, first_vertex + 1u, first_vertex + 2u);
        mesh_output.primitives[first_primitive].light = light;
        mesh_output.primitives[first_primitive + 1u].indices =
            vec3(first_vertex, first_vertex + 2u, first_vertex + 3u);
        mesh_output.primitives[first_primitive + 1u].light = light;
    }
}

struct PrimitiveInput {
    @per_primitive @location(1) light: f32,
}

@fragment
fn fragment(vertex: Vertex, primitive: PrimitiveInput) -> @location(0) vec4<f32> {
    return vec4(vertex.color * primitive.light, 1.0);
}
