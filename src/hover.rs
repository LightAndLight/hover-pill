use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    controls::Controlled,
    fuel::{subtract_fuel, Fuel, FuelChanged},
    jump::JumpImpulse,
};

pub fn start_hover(
    fuel: &Fuel,
    controlled: &mut Controlled,
    external_impulse: &mut ExternalImpulse,
    jump_impulse: &JumpImpulse,
    external_force: &mut ExternalForce,
) {
    debug!("start_hover: fuel: {:?}", fuel.value);
    if fuel.value > 0.0 {
        debug!("start_hover: hovering");
        controlled.hovering = true;

        external_impulse.impulse = jump_impulse.value;
        external_force.force = 12. * Vec3::Y;
    }
}

pub fn end_hover(
    controlled: &mut Controlled,
    external_impulse: &mut ExternalImpulse,
    external_force: &mut ExternalForce,
) {
    controlled.hovering = false;
    external_impulse.impulse = Vec3::ZERO;
    external_force.force = Vec3::ZERO;
}

pub fn use_fuel_to_hover(
    time: Res<Time>,
    mut query: Query<(
        &mut Controlled,
        &mut Fuel,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
    mut fuel_changed: EventWriter<FuelChanged>,
) {
    for (mut controlled, mut fuel, mut external_impulse, mut external_force) in &mut query {
        if controlled.hovering {
            subtract_fuel(&mut fuel, time.delta_seconds() * 0.1, &mut fuel_changed);

            if fuel.value <= 0. {
                end_hover(&mut controlled, &mut external_impulse, &mut external_force)
            }
        }
    }
}

pub struct HoverPlugin;

impl Plugin for HoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(use_fuel_to_hover);
    }
}
