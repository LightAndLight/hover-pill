use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{hover::HoverEvent, reset::ResetEvent};

#[derive(Clone, Copy, Component)]
pub struct Controlled {
    pub rotating: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl Controlled {
    pub fn new() -> Self {
        Controlled {
            rotating: false,
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }

    pub fn reset(&mut self) {
        let Controlled {
            rotating,
            forward,
            backward,
            left,
            right,
        } = self;

        *rotating = false;
        *forward = false;
        *backward = false;
        *left = false;
        *right = false;
    }
}

impl Default for Controlled {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component)]
pub struct Forward {
    pub value: Vec3,
}

#[derive(Component)]
pub struct Speed {
    pub value: f32,
}

fn handle_movement(keys: Res<Input<KeyCode>>, mut query: Query<&mut Controlled>) {
    for mut controlled in query.iter_mut() {
        if keys.just_pressed(KeyCode::W) {
            controlled.forward = true;
        }

        if keys.just_released(KeyCode::W) {
            controlled.forward = false;
        }

        if keys.just_pressed(KeyCode::A) {
            controlled.left = true;
        }

        if keys.just_released(KeyCode::A) {
            controlled.left = false;
        }

        if keys.just_pressed(KeyCode::S) {
            controlled.backward = true;
        }

        if keys.just_released(KeyCode::S) {
            controlled.backward = false;
        }

        if keys.just_pressed(KeyCode::D) {
            controlled.right = true;
        }

        if keys.just_released(KeyCode::D) {
            controlled.right = false;
        }
    }
}

fn handle_jump(keys: Res<Input<KeyCode>>, mut hover_event: EventWriter<HoverEvent>) {
    if keys.just_pressed(KeyCode::Space) {
        hover_event.send(HoverEvent::Start);
    }

    if keys.just_released(KeyCode::Space) {
        hover_event.send(HoverEvent::Stop);
    }
}

fn handle_rotate(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut query: Query<&mut Controlled>,
) {
    for mouse_button_event in mouse_button_events.iter() {
        if let MouseButton::Right = mouse_button_event.button {
            for mut controlled in query.iter_mut() {
                match mouse_button_event.state {
                    ButtonState::Pressed => {
                        controlled.rotating = true;
                    }
                    ButtonState::Released => {
                        controlled.rotating = false;
                    }
                };
            }
        }
    }
}

fn hide_cursor(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut query: Query<&mut Controlled>,
) {
    let mut window = windows.get_single_mut().unwrap();
    for controlled in query.iter_mut() {
        window.cursor.visible = !controlled.rotating;
    }
}

pub fn handle_reset(keys: Res<Input<KeyCode>>, mut reset_event: EventWriter<ResetEvent>) {
    if keys.just_pressed(KeyCode::R) {
        reset_event.send(ResetEvent)
    }
}

#[derive(Resource)]
pub struct ControlsConfig {
    pub enabled: bool,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
struct ActiveSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
struct PassiveSet;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlsConfig>()
            .configure_set(ActiveSet.run_if(|config: Res<ControlsConfig>| config.enabled))
            .add_systems(
                (handle_movement, handle_jump, handle_rotate, handle_reset).in_set(ActiveSet),
            )
            .add_system(hide_cursor.after(handle_rotate).in_set(PassiveSet));
    }
}
