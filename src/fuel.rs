use bevy::prelude::*;

#[derive(Component)]
pub struct Fuel {
    pub value: f32,
}

pub struct FuelChanged {
    pub new_value: f32,
}

pub fn subtract_fuel(fuel: &mut Fuel, amount: f32, fuel_changed: &mut EventWriter<FuelChanged>) {
    fuel.value = (fuel.value - amount).clamp(0.0, 1.0);
    fuel_changed.send(FuelChanged {
        new_value: fuel.value,
    });
}

pub fn add_fuel(fuel: &mut Fuel, amount: f32, fuel_changed: &mut EventWriter<FuelChanged>) {
    fuel.value = (fuel.value + amount).clamp(0.0, 1.0);
    fuel_changed.send(FuelChanged {
        new_value: fuel.value,
    });
}

pub struct FuelPlugin;

impl Plugin for FuelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FuelChanged>();
    }
}
