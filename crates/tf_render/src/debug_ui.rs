use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{
    AlignItems, BackgroundColor, BorderColor, BorderRadius, Display, FlexDirection, JustifyContent, Node,
    Overflow, PositionType, UiRect, Val, ZIndex,
};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use tf_simulation::DebugControlsState;

#[derive(Component, Debug, Clone, Copy, Default)]
struct DebugOverlayRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
struct DebugConsoleText;

#[derive(Component, Debug, Clone, Copy, Default)]
struct DebugKeyHelpText;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_ui)
            .add_systems(Update, (toggle_debug_mode_input, update_debug_ui));
    }
}

fn setup_debug_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(33.333),
                display: Display::None,
                flex_direction: FlexDirection::Row,
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(200),
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.92)),
            DebugOverlayRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::right(Val::Px(1.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BorderColor(Color::srgba(0.75, 0.82, 0.90, 0.25)),
            ))
            .with_children(|left| {
                left.spawn((
                    Text::new("DEBUG CONSOLE"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.89, 0.92, 0.97)),
                ));

                left.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.01, 0.01, 0.02, 0.65)),
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|console| {
                    console.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.79, 0.84, 0.90)),
                        DebugConsoleText,
                    ));
                });
            });

            root.spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.35)),
            ))
            .with_children(|right| {
                right.spawn((
                    Text::new("DEBUG KEYS"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.89, 0.92, 0.97)),
                ));

                right.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.80, 0.86, 0.93)),
                    DebugKeyHelpText,
                ));
            });
        });
}

fn toggle_debug_mode_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugControlsState>,
) {
    if keyboard.just_pressed(KeyCode::KeyC)
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        debug_state.enabled = !debug_state.enabled;
        if debug_state.enabled {
            debug_state.push_console_line("Debug mode enabled");
        } else {
            debug_state.push_console_line("Debug mode disabled");
        }
    }

    if !debug_state.enabled {
        return;
    }

    if keyboard.just_pressed(KeyCode::BracketRight) {
        debug_state.zero_wind_enabled = !debug_state.zero_wind_enabled;
        if debug_state.zero_wind_enabled {
            debug_state.push_console_line("] pressed: zero wind field enabled (0.0, 0.0)");
        } else {
            debug_state.push_console_line("] pressed: zero wind field disabled");
        }
    }
}

fn update_debug_ui(
    debug_state: Res<DebugControlsState>,
    mut root_q: Query<&mut Node, With<DebugOverlayRoot>>,
    mut console_q: Query<&mut Text, (With<DebugConsoleText>, Without<DebugKeyHelpText>)>,
    mut key_help_q: Query<&mut Text, (With<DebugKeyHelpText>, Without<DebugConsoleText>)>,
) {
    let Ok(mut root_node) = root_q.get_single_mut() else {
        return;
    };

    root_node.display = if debug_state.enabled {
        Display::Flex
    } else {
        Display::None
    };

    let Ok(mut console_text) = console_q.get_single_mut() else {
        return;
    };

    let mut lines: Vec<String> = debug_state
        .console_lines()
        .map(|line| {
            let max_width = 80;
            if line.len() > max_width {
                format!("{}…", &line[..max_width])
            } else {
                line.to_string()
            }
        })
        .collect();
    if lines.is_empty() {
        lines.push("No debug output yet".to_string());
    }
    let keep = 10;
    if lines.len() > keep {
        lines = lines.split_off(lines.len() - keep);
    }
    console_text.0 = lines.join("\n");

    let Ok(mut key_help_text) = key_help_q.get_single_mut() else {
        return;
    };

    key_help_text.0 = format!(
        "Shift+C  | Toggle debug overlay       [{}]\n]        | Toggle wind to zero [{}]",
        on_off(debug_state.enabled),
        on_off(debug_state.zero_wind_enabled)
    );
}

fn on_off(enabled: bool) -> &'static str {
    if enabled {
        "ENABLED"
    } else {
        "disabled"
    }
}
