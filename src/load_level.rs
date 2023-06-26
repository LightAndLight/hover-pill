use bevy::prelude::*;

use crate::{
    fuel::FuelChanged,
    level::Level,
    level_editor,
    pause::PauseEvent,
    player,
    ui::{self, UI},
    GameState,
};

pub struct LoadEvent {
    pub path: String,
}

#[derive(Resource)]
pub struct CurrentLevel {
    pub path: String,
    pub handle: Handle<Level>,
    pub level: Level,
    created: bool,
}

#[derive(Component)]
pub enum InCurrentLevel {
    NoLocation,
    LevelItem(usize),
}

#[derive(Resource)]
pub struct LoadingLevel {
    pub path: String,
    pub handle: Handle<Level>,
}

fn start_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut load_events: EventReader<LoadEvent>,
) {
    if let Some(LoadEvent { path }) = load_events.iter().last() {
        trace!("start_loading: {:?}", path);

        let handle = asset_server.load(path);
        commands.insert_resource(LoadingLevel {
            path: path.clone(),
            handle,
        });
    }
}

fn finish_loading(
    mut commands: Commands,
    assets: Res<Assets<Level>>,
    loading_level: Res<LoadingLevel>,
) {
    if let Some(level) = assets.get(&loading_level.handle) {
        trace!("finish_loading");

        commands.remove_resource::<LoadingLevel>();

        commands.insert_resource(CurrentLevel {
            path: loading_level.path.clone(),
            handle: loading_level.handle.clone(),
            level: level.clone(),
            created: false,
        });
    }
}

fn hotreload(
    mut commands: Commands,
    assets: Res<Assets<Level>>,
    mut asset_event: EventReader<AssetEvent<Level>>,
    current_level: Res<CurrentLevel>,
) {
    for event in asset_event.iter() {
        debug!("level asset event: {:?}", event);

        if let AssetEvent::Modified {
            handle: modified_handle,
        } = event
        {
            debug!("asset modified: {:?}", modified_handle);

            if modified_handle == &current_level.handle {
                if let Some(level) = assets.get(&current_level.handle) {
                    commands.insert_resource(CurrentLevel {
                        path: current_level.path.clone(),
                        handle: current_level.handle.clone(),
                        level: level.clone(),
                        created: false,
                    });
                }
            }
        }
    }
}

fn setup_current_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<State<GameState>>,
    mut ui: ResMut<UI>,
    mut fuel_changed: EventWriter<FuelChanged>,
    mut current_level: ResMut<CurrentLevel>,
    mut pause_event: EventWriter<PauseEvent>,
    in_current_level_query: Query<Entity, With<InCurrentLevel>>,
) {
    if !current_level.created {
        trace!("clearing level entities");

        for entity in in_current_level_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 10000.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::PI / 3.5,
                )),
                ..default()
            },
            InCurrentLevel::NoLocation,
        ));

        current_level
            .level
            .structure
            .iter()
            .enumerate()
            .for_each(|(index, item)| {
                item.spawn(&mut commands, &mut meshes, &mut materials)
                    .insert(InCurrentLevel::LevelItem(index));
            });

        match state.0 {
            GameState::MainMenu => panic!("setup_current_level called in GameState::MainMenu"),
            GameState::Playing => {
                player::spawn_player(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    Transform::from_translation(current_level.level.player_start),
                    Some(&mut fuel_changed),
                )
                .insert(InCurrentLevel::NoLocation);

                if let Some(overlay_text) = &current_level.level.initial_overlay {
                    pause_event.send(PauseEvent::Pause);

                    ui::overlay::level_overview::display(
                        &asset_server,
                        &mut commands,
                        &mut ui,
                        overlay_text,
                    );
                }
            }
            GameState::Testing => {}
            GameState::Editing => {
                level_editor::spawn_player_token(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    current_level.level.player_start,
                );
            }
        }

        current_level.created = true;
    }
}

pub struct LoadLevelPlugin;

impl Plugin for LoadLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadEvent>()
            .add_system(start_loading)
            .add_system(finish_loading.run_if(resource_exists::<LoadingLevel>()))
            .add_system(hotreload.run_if(resource_exists::<CurrentLevel>()))
            .add_system(
                setup_current_level
                    /*
                    Adding this to `CoreSet::PreUpdate` ensures it runs before systems that are added
                    as normal, such as `level_editor::annotate_level_items`. If
                    `setup_current_level` and `level_editor::annotate_level_items` run
                    concurrently then it's possible to annotate entities that have just been
                    deleted. They must be run sequentially, but in what order? It's hard toa
                    decide and seems arbitrary. I don't want to expose these systems to allow
                    explicit use of `before`/`after`. Putting them in separate base sets seems
                    like a hacky way to resolve the problem.
                    */
                    .in_base_set(CoreSet::PreUpdate)
                    .run_if(resource_exists::<CurrentLevel>()),
            );
    }
}
