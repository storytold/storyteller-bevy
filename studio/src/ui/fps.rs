use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub(super) struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, layout)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
struct FramerateCounter;

fn layout(mut cmd: Commands) {
    cmd.spawn((
        FramerateCounter,
        TextBundle::from_section(
            "0 fps",
            TextStyle {
                color: Color::GREEN,
                font_size: 14.,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            right: Val::Px(16.),
            bottom: Val::Px(16.),
            ..default()
        }),
    ));
}

fn update(
    time: Res<Time>,
    diag: Res<DiagnosticsStore>,
    mut counter: Query<&mut Text, With<FramerateCounter>>,
) {
    if let Some(fps_diag) = diag.get(FrameTimeDiagnosticsPlugin::FPS) {
        let fps = if let Some(fps_smoothed) = fps_diag.smoothed() {
            fps_smoothed
        } else {
            1. / time.delta_seconds_f64()
        };

        counter.single_mut().sections[0].value = format!("{fps:.2} fps");
    }
}
