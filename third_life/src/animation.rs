use bevy::prelude::*;

use crate::worlds::config::SpriteConfig;

pub struct ThirdLifeAnimationPlugin;

impl Plugin for ThirdLifeAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprites);
    }
}



#[derive(Bundle)]
pub struct ColonyAnimationBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    animation_index: AnimationIndex,
    sprite_size: SpriteSize,
    animation_timer: AnimationTimer,
}

impl ColonyAnimationBundle {
    pub fn new(
        name: String,
        (world_position_x, world_position_y): (isize, isize),
        sprite_sheet: Handle<Image>,
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        sprite: SpriteConfig,
    ) -> Self {
        let num_frames = sprite.frames();
        let (frames_layout_x, frames_layout_y) = sprite.frames_layout();
        let (shape_x, shape_y) = sprite.shape();
        let animation_timer = sprite.animation_timer();
        let layout = TextureAtlasLayout::from_grid(
            Vec2::new(shape_x as f32, shape_y as f32), 
            frames_layout_x, frames_layout_y,
            None, None
        );
        let sprite_size = SpriteSize(layout.size / layout.len() as f32);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let animation_index = AnimationIndex::new(num_frames);
        Self { 
            sprite_sheet_bundle: SpriteSheetBundle {
                transform: Transform::from_xyz(
                    world_position_x as f32, world_position_y as f32, 0.
                ).with_scale(Vec3::splat(2.)),
                texture: sprite_sheet,
                atlas: TextureAtlas { 
                    layout: texture_atlas_layout,
                    index: animation_index.first
                },
                ..default()
            },
            animation_index,
            sprite_size,
            animation_timer: AnimationTimer(Timer::from_seconds(
                    animation_timer,
                    TimerMode::Repeating
            ))
        }
    }
}


#[derive(Component)]
pub struct AnimationIndex {
    pub first: usize,
    pub last: usize,
}

impl AnimationIndex {
    pub fn new(last: usize) -> Self {
        Self { first: 0, last: last - 1 }
    }
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct SpriteSize(pub Vec2);


fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndex, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (index, mut timer, mut atlas) in &mut query {
        if timer.0.tick(time.delta()).finished() {
            atlas.index = if atlas.index == index.last {
                index.first
            } else {
                atlas.index + 1
            };
        }
    }
}

