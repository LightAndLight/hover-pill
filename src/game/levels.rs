use std::f32::consts::PI;

use bevy::prelude::*;

use crate::level::{Level, LevelItem, WallType};

pub const WORLD_BOX_SIZE: f32 = 14.0;

pub fn tutorial_1() -> Level {
    let level = Level {
        next_level: Some("tutorial_2".into()),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: Some(vec![
            "w - move forward".into(),
            "s - move backward".into(),
            "a - move left".into(),
            "d - move right".into(),
            "right click and drag - look around".into(),
        ]),
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: Vec3::new(0.0, 0.0, WORLD_BOX_SIZE / 6.0),
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, 2.0 * WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                position: (WORLD_BOX_SIZE - WORLD_BOX_SIZE / 3.0) * Vec3::Z,
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    };

    let file = std::fs::File::create("assets/levels/tutorial_1.json").unwrap();
    serde_json::to_writer_pretty(file, &level).unwrap();
    level
}

pub fn tutorial_2() -> Level {
    let level = Level {
        next_level: Some("tutorial_3".into()),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: Some(vec!["space - hover".into()]),
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: Vec3::new(0.0, 0.0, 2.0 * WORLD_BOX_SIZE / 3.0),
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                position: Vec3::new(0.0, 0.0, 3.0 * WORLD_BOX_SIZE / 3.0),
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    };

    let file = std::fs::File::create("assets/levels/tutorial_2.json").unwrap();
    serde_json::to_writer_pretty(file, &level).unwrap();
    level
}

pub fn tutorial_3() -> Level {
    let level = Level {
        next_level: Some("level_1".into()),
        player_start: 3.0 * Vec3::Y,
        initial_overlay: Some(vec!["red - avoid".into()]),
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                position: Vec3::new(0.0, 0.0, WORLD_BOX_SIZE / 3.0),
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                position: Vec3::new(0.0, 0.0, 2.0 * WORLD_BOX_SIZE / 3.0),
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
        ],
    };

    let file = std::fs::File::create("assets/levels/tutorial_3.json").unwrap();
    serde_json::to_writer_pretty(file, &level).unwrap();
    level
}

pub fn level_1() -> Level {
    let level = Level {
        next_level: None,
        player_start: Vec3::new(0.0, 1.0, 0.0),
        initial_overlay: None,
        structure: vec![
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                position: Vec3::new(
                    -WORLD_BOX_SIZE / 3.0,
                    WORLD_BOX_SIZE / 2.0,
                    WORLD_BOX_SIZE / 2.0,
                ),
                rotation: Quat::from_rotation_x(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                position: Vec3::new(
                    WORLD_BOX_SIZE / 3.0,
                    WORLD_BOX_SIZE / 2.0,
                    WORLD_BOX_SIZE / 2.0,
                ),
                rotation: Quat::from_rotation_x(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                position: Vec3::new(0.0, WORLD_BOX_SIZE / 6.0, WORLD_BOX_SIZE / 2.0),
                rotation: Quat::from_rotation_x(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Avoid,
                position: Vec3::new(
                    0.0,
                    WORLD_BOX_SIZE / 6.0 + 2.0 * WORLD_BOX_SIZE / 3.0,
                    WORLD_BOX_SIZE / 2.0,
                ),
                rotation: Quat::from_rotation_x(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Goal,
                position: (WORLD_BOX_SIZE + 0.05) * Vec3::Y,
                rotation: Quat::default(),
                size: Vec2::new(WORLD_BOX_SIZE / 3.0, WORLD_BOX_SIZE / 3.0),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: (WORLD_BOX_SIZE / 2.0) * -Vec3::Z + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                rotation: Quat::from_rotation_x(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: (WORLD_BOX_SIZE / 2.0) * Vec3::X + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                rotation: Quat::from_rotation_z(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: (WORLD_BOX_SIZE / 2.0) * -Vec3::X + (WORLD_BOX_SIZE / 2.0) * Vec3::Y,
                rotation: Quat::from_rotation_z(PI / 2.0),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::Wall {
                wall_type: WallType::Neutral,
                position: WORLD_BOX_SIZE * Vec3::Y,
                rotation: Quat::default(),
                size: Vec2::new(WORLD_BOX_SIZE, WORLD_BOX_SIZE),
            },
            LevelItem::FuelBall {
                position: Vec3::new(2.0, 2.0, 2.0),
            },
        ],
    };

    let file = std::fs::File::create("assets/levels/level_1.json").unwrap();
    serde_json::to_writer_pretty(file, &level).unwrap();
    level
}
