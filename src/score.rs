use crate::loading::FontAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct ScorePlugin;

#[derive(Resource)]
pub struct Score {
    pub value: i32,
}

#[derive(Component)]
struct ScoreBoard;

impl Default for Score {
    fn default() -> Self {
        Score { value: 0 }
    }
}

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_system(setup_score.in_schedule(OnEnter(GameState::Playing)))
            .add_system(update_score.in_set(OnUpdate(GameState::Playing)));
    }
}

fn setup_score(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands.spawn((
        TextBundle::from_section(
            "Score: 0",
            TextStyle {
                font: font_assets.fira_sans.clone(),
                font_size: 40.,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        ),
        ScoreBoard,
    ));
}

fn update_score(score: Res<Score>, mut query: Query<(&ScoreBoard, &mut Text)>) {
    let (_, mut text) = query.single_mut();

    text.sections[0].value = format!("Score: {}", score.value);
}
