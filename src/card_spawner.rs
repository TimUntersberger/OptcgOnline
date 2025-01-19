use bevy::{app::Plugin, prelude::*};
use bevy_simple_text_input::{
    TextInput, TextInputPlugin, TextInputSubmitEvent, TextInputSystem, TextInputTextColor, TextInputTextFont
};

#[derive(Event, Default)]
pub struct CardSpawnerOpenEvent;

#[derive(Event)]
pub struct CardSpawnerSelectedEvent;

#[derive(Component, Default)]
pub struct CardSpawnerModalUI;

pub struct CardSpawnerPlugin;

impl Plugin for CardSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CardSpawnerOpenEvent>()
            .add_event::<CardSpawnerSelectedEvent>()
            .add_systems(Update, (react_to_open_request_system));
    }
}

fn react_to_open_request_system(
    mut commands: Commands,
    mut events: EventReader<CardSpawnerOpenEvent>,
) {
    if let Some(event) = events.read().last() {
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Percent(80.0),
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(200.0, 200.0, 200.0)),
                )).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(200.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(Color::WHITE),
                        TextInput,
                        TextInputTextFont(TextFont {
                            font_size: 34.,
                            ..default()
                        }),
                        TextInputTextColor(TextColor(Color::BLACK)),
                    ));
                });
            });
    }
}
