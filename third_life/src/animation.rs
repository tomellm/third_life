use bevy::prelude::*;

pub struct ThirdLifeAnimationPlugin;

impl Plugin for ThirdLifeAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprites);
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

