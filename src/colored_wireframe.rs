/*
use bevy_app::Plugin;
use bevy_asset::{load_internal_asset, Handle, HandleUntyped};
use bevy_core_pipeline::core_3d::Opaque3d;
use bevy_ecs::{prelude::*, reflect::ReflectComponent};
use bevy_pbr::MeshPipeline;
use bevy_pbr::{DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup};
use bevy_reflect::std_traits::ReflectDefault;
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::Extract;
use bevy_render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    mesh::{Mesh, MeshVertexBufferLayout},
    render_asset::RenderAssets,
    render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
    render_resource::{
        PipelineCache, PolygonMode, RenderPipelineDescriptor, Shader, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines,
    },
    view::{ExtractedView, Msaa, VisibleEntities},
    RenderApp, RenderStage,
};
use bevy_utils::tracing::error;
*/

use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshBindGroup, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup, SkinnedMeshJoints,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::DynamicUniformIndex,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, PipelineCache, PolygonMode,
            RenderPipelineDescriptor, SpecializedMeshPipeline, SpecializedMeshPipelineError,
            SpecializedMeshPipelines,
        },
        renderer::RenderDevice,
        view::{ExtractedView, VisibleEntities},
        Extract, RenderApp, RenderStage,
    },
};

pub const WIREFRAME_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 327186210400322572);

#[derive(Debug, Default)]
pub struct ColoredWireframePlugin;

impl Plugin for ColoredWireframePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            WIREFRAME_SHADER_HANDLE,
            "render/wireframe.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<ColoredWireframeConfig>()
            .init_resource::<ColoredWireframeConfig>()
            .add_plugin(ExtractResourcePlugin::<ColoredWireframeConfig>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, DrawWireframes>()
                .init_resource::<ColoredWireframePipeline>()
                .init_resource::<SpecializedMeshPipelines<ColoredWireframePipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_wireframes)
                .add_system_to_stage(RenderStage::Queue, queue_wireframes);
        }
    }
}

fn extract_wireframes(mut commands: Commands, query: Extract<Query<(Entity, &ColoredWireframe)>>) {
    for (entity, wireframe) in query.iter() {
        commands.get_or_spawn(entity).insert(*wireframe);
    }
}

/// Controls whether an entity should rendered in wireframe-mode if the [`WireframePlugin`] is enabled
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
pub struct ColoredWireframe {
    /// The wireframe color.
    color: Color,
}

#[derive(Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct ColoredWireframeConfig {
    /// Whether to show wireframes for all meshes. If `false`, only meshes with a [Wireframe] component will be rendered.
    pub global: Option<Color>,
}

pub struct ColoredWireframePipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
}
impl FromWorld for ColoredWireframePipeline {
    fn from_world(render_world: &mut World) -> Self {
        ColoredWireframePipeline {
            mesh_pipeline: render_world.resource::<MeshPipeline>().clone(),
            shader: WIREFRAME_SHADER_HANDLE.typed(),
        }
    }
}

impl SpecializedMeshPipeline for ColoredWireframePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone_weak();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone_weak();
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        descriptor.depth_stencil.as_mut().unwrap().bias.slope_scale = 1.0;
        Ok(descriptor)
    }
}

struct ColoredWireframeBindGroup(BindGroup);

#[allow(clippy::too_many_arguments)]
fn queue_wireframes(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    wireframe_config: Res<ColoredWireframeConfig>,
    wireframe_pipeline: Res<ColoredWireframePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ColoredWireframePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    mut material_meshes: ParamSet<(
        Query<(Entity, &Handle<Mesh>, &MeshUniform)>,
        Query<(Entity, &Handle<Mesh>, &MeshUniform), With<ColoredWireframe>>,
    )>,
    mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_custom = opaque_3d_draw_functions
        .read()
        .get_id::<DrawWireframes>()
        .unwrap();
    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);
    for (view, visible_entities, mut opaque_phase) in &mut views {
        let rangefinder = view.rangefinder3d();

        let add_render_phase =
            |(entity, mesh_handle, mesh_uniform): (Entity, &Handle<Mesh>, &MeshUniform)| {
                if let Some(mesh) = render_meshes.get(mesh_handle) {
                    let key = msaa_key
                        | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                    let pipeline_id = pipelines.specialize(
                        &mut pipeline_cache,
                        &wireframe_pipeline,
                        key,
                        &mesh.layout,
                    );
                    let pipeline_id = match pipeline_id {
                        Ok(id) => id,
                        Err(err) => {
                            error!("{}", err);
                            return;
                        }
                    };
                    opaque_phase.add(Opaque3d {
                        entity,
                        pipeline: pipeline_id,
                        draw_function: draw_custom,
                        distance: rangefinder.distance(&mesh_uniform.transform),
                    });
                }
            };

        match wireframe_config.global {
            Some(color) => {
                commands.insert_resource(ColoredWireframeBindGroup(
                    render_device.create_bind_group(&BindGroupDescriptor {
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: mesh_binding.clone(),
                        }],
                        label: Some("colored_wireframe_bind_group"),
                        layout: &mesh_pipeline.mesh_layout,
                    }),
                ));

                let query = material_meshes.p0();
                visible_entities
                    .entities
                    .iter()
                    .filter_map(|visible_entity| query.get(*visible_entity).ok())
                    .for_each(add_render_phase);
            }
            None => {
                let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: mesh_binding.clone(),
                    }],
                    label: Some("colored_wireframe_bind_group"),
                    layout: &mesh_pipeline.mesh_layout,
                });

                let query = material_meshes.p1();
                visible_entities
                    .entities
                    .iter()
                    .filter_map(|visible_entity| query.get(*visible_entity).ok())
                    .for_each(add_render_phase);
            }
        }
    }
}

pub struct SetColorBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetColorBindGroup<I> {
    type Param = (
        SRes<ColoredWireframeBindGroup>,
        SQuery<Read<DynamicUniformIndex<ColoredWireframe>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (colored_wireframe_bind_group, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let colored_wireframe_index = query.get(item).unwrap();
        pass.set_bind_group(
            I,
            &colored_wireframe_bind_group.into_inner(),
            &[colored_wireframe_index.index()],
        );
        RenderCommandResult::Success
    }
}

type DrawWireframes = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetColorBindGroup<2>,
    DrawMesh,
);
