use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    fuel::{subtract_fuel, Fuel, FuelChanged},
    jump::JumpImpulse,
};

pub enum HoverEvent {
    Start,
    Stop,
}

#[derive(Component)]
pub struct Hovering {
    pub value: bool,
}

pub fn start_hover(
    fuel: &Fuel,
    hovering: &mut Hovering,
    external_impulse: &mut ExternalImpulse,
    jump_impulse: &JumpImpulse,
    external_force: &mut ExternalForce,
) {
    if fuel.value > 0.0 {
        hovering.value = true;
        external_impulse.impulse = jump_impulse.value;
        external_force.force = 12. * Vec3::Y;
    }
}

pub fn end_hover(
    hovering: &mut Hovering,
    external_impulse: &mut ExternalImpulse,
    external_force: &mut ExternalForce,
) {
    hovering.value = false;
    external_impulse.impulse = Vec3::ZERO;
    external_force.force = Vec3::ZERO;
}

pub fn handle_hover_events(
    mut events: EventReader<HoverEvent>,
    mut query: Query<(
        &mut Hovering,
        &Fuel,
        &JumpImpulse,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
) {
    for event in events.iter() {
        match event {
            HoverEvent::Start => {
                for (mut hovering, fuel, jump_impulse, mut external_impulse, mut external_force) in
                    query.iter_mut()
                {
                    start_hover(
                        fuel,
                        &mut hovering,
                        &mut external_impulse,
                        jump_impulse,
                        &mut external_force,
                    );
                }
            }
            HoverEvent::Stop => {
                for (mut hovering, _, _, mut external_impulse, mut external_force) in
                    query.iter_mut()
                {
                    end_hover(&mut hovering, &mut external_impulse, &mut external_force);
                }
            }
        }
    }
}

pub fn use_fuel_to_hover(
    time: Res<Time>,
    mut query: Query<(
        &mut Hovering,
        &mut Fuel,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    for (mut hovering, mut fuel, mut external_impulse, mut external_force) in &mut query {
        if hovering.value {
            subtract_fuel(&mut fuel, time.delta_seconds() * 0.1, &mut fuel_changed);

            if fuel.value <= 0. {
                end_hover(&mut hovering, &mut external_impulse, &mut external_force)
            }
        }
    }
}

pub struct HoverPlugin;

impl Plugin for HoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HoverEvent>();
    }
}
