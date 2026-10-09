//! Mesh shaders (Bevy 0.20): `MeshPipelineDescriptor` pipelines live in the regular
//! `PipelineCache` and are drawn with `TrackedRenderPass::draw_mesh_tasks`.
//!
//! A task shader launches one mesh shader workgroup per column, and each of those emits a whole
//! box. No vertex or index buffer exists anywhere; see `assets/mesh_shaders/columns.wesl`.
//!
//! Mesh shaders are an optional GPU feature (`WgpuFeatures::EXPERIMENTAL_MESH_SHADER`). Bevy's
//! default `WgpuSettings` already request every feature the adapter offers, so nothing has to be
//! configured up front, but the demo has to check that the device really has the feature.

use bevy::{
    camera::RenderTarget,
    core_pipeline::{
        core_3d::{main_opaque_pass_3d, CORE_3D_DEPTH_FORMAT},
        Core3d, Core3dSystems,
    },
    material::descriptor::{MeshPipelineDescriptor, MeshState, TaskState},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        globals::{GlobalsBuffer, GlobalsUniform},
        render_resource::{
            binding_types::uniform_buffer, BindGroupEntries, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            CompareFunction, DepthBiasState, DepthStencilState, FragmentState, PipelineCache,
            RenderPassDescriptor, ShaderStages, StencilState, StoreOp, TextureFormat,
        },
        renderer::{RenderAdapterInfo, RenderContext, RenderDevice, ViewQuery},
        settings::{Backends, WgpuFeatures},
        view::{ViewDepthStencilTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        RenderApp, RenderStartup,
    },
    ui::widget::ViewportNode,
};

use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

const SHADER: &str = "mesh_shaders/columns.wesl";
/// Format of the texture the demo's camera renders to, which the pipeline has to match.
const TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::MeshShaders)
        .add_plugins(ExtractComponentPlugin::<MeshShaderView>::default())
        .add_systems(OnEnter(Demo::MeshShaders), setup);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .add_systems(RenderStartup, queue_pipeline)
        .add_systems(
            Core3d,
            draw_columns
                .after(main_opaque_pass_3d)
                .in_set(Core3dSystems::MainPass),
        );
}

/// Marks the camera the mesh shader draws into, so the other demos' cameras are left alone.
#[derive(Component, Clone, ExtractComponent)]
#[extract_app(RenderApp)]
struct MeshShaderView;

/// Explains why mesh shaders cannot be used on this device, or returns `Ok` if they can.
fn check_support(device: &RenderDevice, adapter: &RenderAdapterInfo) -> Result<(), String> {
    if !device
        .features()
        .contains(WgpuFeatures::EXPERIMENTAL_MESH_SHADER)
    {
        return Err(format!(
            "{} ({}) does not expose the wgpu feature EXPERIMENTAL_MESH_SHADER.",
            adapter.name, adapter.backend
        ));
    }
    // wgpu 30 can only translate WGSL / WESL mesh shaders for Vulkan. DX12 and Metal support the
    // feature, but need precompiled passthrough shaders.
    if Backends::from(adapter.backend) != Backends::VULKAN {
        return Err(format!(
            "{} supports mesh shaders, but wgpu can only compile them from WESL on Vulkan and \
             this app is running on {}. Try starting it with WGPU_BACKEND=vulkan.",
            adapter.name, adapter.backend
        ));
    }
    Ok(())
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    device: Res<RenderDevice>,
    adapter: Res<RenderAdapterInfo>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let caption = match check_support(&device, &adapter) {
        Ok(()) => {
            spawn_scene(*root, &mut commands, &mut images, &mut meshes, &mut materials);
            format!(
                "28 x 28 columns, 24 vertices and 12 triangles each, rebuilt every frame by a \
                 mesh shader on {} ({}).\nThe grey floor is an ordinary Mesh3d sharing the same \
                 depth buffer.",
                adapter.name, adapter.backend
            )
        }
        Err(reason) => format!("Mesh shaders are unsupported on this GPU.\n\n{reason}"),
    };
    commands.spawn((
        ChildOf(*root),
        DespawnOnExit(Demo::MeshShaders),
        Node {
            padding: UiRect::all(px(16)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![(
            Text(caption),
            TextFont {
                font_size: FontSize::Rem(0.9),
                ..default()
            },
        )],
    ));
}

fn spawn_scene(
    root: Entity,
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // The camera renders into a texture that a `ViewportNode` shows in the content area and
    // keeps at the node's size.
    let target = images.add(Image::new_target_texture(0, 0, TARGET_FORMAT, None));
    let camera = commands
        .spawn((
            DespawnOnExit(Demo::MeshShaders),
            Camera3d::default(),
            RenderTarget::Image(target.into()),
            MeshShaderView,
            Transform::from_xyz(22.0, 22.0, 30.0).looking_at(Vec3::new(0.0, -1.0, 0.0), Vec3::Y),
            // The pipeline below is single-sampled.
            Msaa::Off,
        ))
        .id();
    commands.spawn((
        ChildOf(root),
        DespawnOnExit(Demo::MeshShaders),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ViewportNode::new(camera),
    ));
    commands.spawn((
        DespawnOnExit(Demo::MeshShaders),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(34.0, 34.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.23))),
    ));
    commands.spawn((
        DespawnOnExit(Demo::MeshShaders),
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Resource)]
struct ColumnsPipeline {
    id: CachedRenderPipelineId,
    layout: BindGroupLayoutDescriptor,
}

/// Queues the task + mesh + fragment pipeline, exactly like a vertex + fragment one.
fn queue_pipeline(
    device: Res<RenderDevice>,
    adapter: Res<RenderAdapterInfo>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    mut commands: Commands,
) {
    if check_support(&device, &adapter).is_err() {
        return;
    }

    let shader = asset_server.load::<Shader>(SHADER);
    let layout = BindGroupLayoutDescriptor::new(
        "mesh_shader_columns_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::TASK | ShaderStages::MESH | ShaderStages::FRAGMENT,
            (
                uniform_buffer::<GlobalsUniform>(false),
                uniform_buffer::<ViewUniform>(true),
            ),
        ),
    );

    let id = pipeline_cache.queue_mesh_pipeline(MeshPipelineDescriptor {
        label: Some("mesh_shader_columns_pipeline".into()),
        layout: vec![layout.clone()],
        immediate_size: 0,
        task: Some(TaskState {
            shader: shader.clone(),
            entry_point: Some("task".into()),
            ..default()
        }),
        mesh: MeshState {
            shader: shader.clone(),
            entry_point: Some("mesh".into()),
            ..default()
        },
        primitive: default(),
        depth_stencil: Some(DepthStencilState {
            format: CORE_3D_DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            // Bevy uses reversed depth: nearer fragments have greater values.
            depth_compare: Some(CompareFunction::GreaterEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: default(),
        fragment: Some(FragmentState {
            shader,
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: TARGET_FORMAT,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(ColumnsPipeline { id, layout });
}

/// Runs right after the opaque pass of the demo's camera and draws on top of it.
fn draw_columns(
    view: ViewQuery<
        (&ViewTarget, &ViewDepthStencilTexture, &ViewUniformOffset),
        With<MeshShaderView>,
    >,
    columns: Option<Res<ColumnsPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    globals: Res<GlobalsBuffer>,
    mut render_context: RenderContext,
) {
    let Some(columns) = columns else {
        return;
    };
    let (Some(pipeline), Some(view_binding), Some(globals_binding)) = (
        pipeline_cache.get_render_pipeline(columns.id),
        view_uniforms.uniforms.binding(),
        globals.buffer.binding(),
    ) else {
        return;
    };
    let (target, depth, view_uniform_offset) = view.into_inner();

    let bind_group = render_context.render_device().create_bind_group(
        "mesh_shader_columns_bind_group",
        &pipeline_cache.get_bind_group_layout(&columns.layout),
        &BindGroupEntries::sequential((globals_binding, view_binding)),
    );

    // A mesh pipeline is a `RenderPipeline`, so it is used in an ordinary render pass.
    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("mesh_shader_columns_pass"),
        color_attachments: &[Some(target.get_color_attachment())],
        depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[view_uniform_offset.offset]);
    // Instead of `draw(vertices, instances)`: dispatch one task shader workgroup, which in turn
    // launches the mesh shader workgroups that emit the geometry.
    pass.draw_mesh_tasks(1, 1, 1);
}
