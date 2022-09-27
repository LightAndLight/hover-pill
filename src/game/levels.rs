use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    level::{Level, LevelItem, WallType},
    ui,
};

pub const WORLD_BOX_SIZE: f32 = 14.0;

pub fn tutorial_1() -> Level {
    Level {
        next_level: Some(tutorial_2),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: Some(ui::tutorial::display_tutorial_1),
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_xyz(0.0, 0.0, WORLD_BOX_SIZE / 6.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, 2.0 * WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                transform: Transform::from_translation(
                    (WORLD_BOX_SIZE - WORLD_BOX_SIZE / 3.0) * Vec3::Z,
                ),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    }
}

pub fn tutorial_2() -> Level {
    Level {
        next_level: Some(tutorial_3),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: Some(ui::tutorial::display_tutorial_2),
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::identity(),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_xyz(0.0, 0.0, 2.0 * WORLD_BOX_SIZE / 3.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                transform: Transform::from_xyz(0.0, 0.0, 3.0 * WORLD_BOX_SIZE / 3.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    }
}

pub fn tutorial_3() -> Level {
    Level {
        next_level: Some(level_1),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: None,
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::identity(),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                transform: Transform::from_xyz(0.0, 0.0, WORLD_BOX_SIZE / 3.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                transform: Transform::from_xyz(0.0, 0.0, 2.0 * WORLD_BOX_SIZE / 3.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    }
}

pub fn level_1() -> Level {
    Level {
        next_level: None,
        player_start: Vec3::new(0.0, 1.0, 0.0),
        initial_overlay: None,
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::default(),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * Vec3::Z - (WORLD_BOX_SIZE / 3.0) * Vec3::X
                            + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * Vec3::Z
                            + (WORLD_BOX_SIZE / 3.0) * Vec3::X
                            + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * Vec3::Z + (WORLD_BOX_SIZE / 6.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * Vec3::Z
                            + (WORLD_BOX_SIZE / 6.0 + 2.0 * WORLD_BOX_SIZE / 3.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                transform: Transform::from_translation((WORLD_BOX_SIZE + 0.05) * Vec3::Y),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * -Vec3::Z + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_rotation(Quat::from_rotation_z(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * Vec3::X + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_rotation(Quat::from_rotation_z(PI / 2.0))
                    .with_translation(
                        (WORLD_BOX_SIZE / 2.0) * -Vec3::X + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                    ),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                transform: Transform::from_translation(WORLD_BOX_SIZE * Vec3::Y),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::FuelBall {
                position: Vec3::new(2.0, 2.0, 2.0),
            },
        ],
    }
}
