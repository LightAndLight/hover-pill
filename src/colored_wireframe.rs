use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
            DynamicUniformBuffer, PipelineCache, PolygonMode, RenderPipelineDescriptor,
            ShaderStages, SpecializedMeshPipeline, SpecializedMeshPipelineError,
            SpecializedMeshPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, VisibleEntities},
        Extract, RenderApp, RenderStage,
    },
};

#[derive(Debug, Clone, Default, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct ColoredWireframeConfig {
    // TODO: make this do something?
    pub enabled: bool,
}

#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
pub struct ColoredWireframe {
    pub color: Color,
}

pub const WIREFRAME_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 327186210400322572);

struct ColoredWireframePipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for ColoredWireframePipeline {
    fn from_world(render_world: &mut World) -> Self {
        ColoredWireframePipeline {
            mesh_pipeline: render_world.resource::<MeshPipeline>().clone(),
            shader: WIREFRAME_SHADER_HANDLE.typed(),
            bind_group_layout: render_world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("colored_wireframe_bind_group_layout_descriptor"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
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

        debug_assert_eq!(descriptor.layout.as_ref().unwrap().len(), 2);
        descriptor
            .layout
            .as_mut()
            .unwrap()
            .push(self.bind_group_layout.clone());

        Ok(descriptor)
    }
}

#[derive(Component)]
pub struct WireframeColorDynamicUniformIndex {
    value: u32,
}

fn extract_wireframes(mut commands: Commands, query: Extract<Query<(Entity, &ColoredWireframe)>>) {
    for (entity, wireframe) in query.iter() {
        commands.get_or_spawn(entity).insert(*wireframe);
    }
}

#[derive(Default)]
struct WireframeColorUniform {
    buffer: DynamicUniformBuffer<Color>,
}

pub struct ColoredWireframeBindGroup {
    value: BindGroup,
}

fn prepare_wireframes(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<ColoredWireframePipeline>,
    query: Query<(Entity, &ColoredWireframe)>,
    mut color_uniform: ResMut<WireframeColorUniform>,
) {
    color_uniform.buffer.clear();

    for (entity, colored_wireframe) in &query {
        let color_index = color_uniform.buffer.push(colored_wireframe.color);

        trace!("entity {:?} has color_index {:?}", entity, color_index);
        commands
            .get_or_spawn(entity)
            .insert(WireframeColorDynamicUniformIndex { value: color_index });
    }

    color_uniform
        .buffer
        .write_buffer(&render_device, &render_queue);

    /*
    Is it costly to re-create the `BindGroup` like this?

    To avoid re-creation, I'd need to know whether `DynamicUniformBuffer::write_buffer`
    has allocated a new buffer or reused the old one. It reallocates when the number
    of items exceed's the buffer's capacity.
    */
    if let Some(resource) = color_uniform.buffer.binding() {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource,
            }],
            label: Some("colored_wireframe_bind_group"),
            layout: &pipeline.bind_group_layout,
        });

        commands.insert_resource(ColoredWireframeBindGroup { value: bind_group });
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_wireframes(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    wireframe_pipeline: Res<ColoredWireframePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ColoredWireframePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    material_meshes_query: Query<
        (Entity, &Handle<Mesh>, &MeshUniform),
        With<WireframeColorDynamicUniformIndex>,
    >,
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

        visible_entities
            .entities
            .iter()
            .filter_map(|visible_entity| material_meshes_query.get(*visible_entity).ok())
            .for_each(add_render_phase);
    }
}

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
                .init_resource::<WireframeColorUniform>()
                .add_system_to_stage(RenderStage::Extract, extract_wireframes)
                .add_system_to_stage(RenderStage::Prepare, prepare_wireframes)
                .add_system_to_stage(RenderStage::Queue, queue_wireframes);
        }
    }
}

pub struct SetColorBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetColorBindGroup<I> {
    type Param = (
        SRes<ColoredWireframeBindGroup>,
        SQuery<Read<WireframeColorDynamicUniformIndex>>,
    );

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (colored_wireframe_bind_group, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let wireframe_color_dynamic_uniform_index = query.get(item).unwrap();

        trace!(
            "wireframe_color_dynamic_uniform_index: {:?}",
            wireframe_color_dynamic_uniform_index.value
        );
        pass.set_bind_group(
            I,
            &colored_wireframe_bind_group.into_inner().value,
            &[wireframe_color_dynamic_uniform_index.value],
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
