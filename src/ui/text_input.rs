use std::mem;

use bevy::{
    input::{
        common_conditions::input_just_pressed,
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    text::{cosmic_text::ttf_parser::Width, TextBounds},
    window::PrimaryWindow,
};
use bevy_mod_picking::prelude::On;

#[derive(Component, Default)]
struct Focused;

#[derive(Component, Default)]
struct UIElement;

#[derive(Component, Default)]
#[require(UIElement)]
struct UITextInput;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (listen_keyboard_input_events));
    }
}

pub fn spawn(parent: &mut ChildBuilder) {
            parent.spawn((
                Text::new("TEST"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                Node {
                    height: Val::Px(24.0),
                    min_width: Val::Px(100.0),
                    border: UiRect::all(Val::Px(2.)),
                    padding: UiRect::all(Val::Px(20.)),
                    margin: UiRect::all(Val::Px(20.)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                UITextInput,
                BackgroundColor(Color::srgb(100.0, 0.0, 0.0)),
            ))
        .observe(on_input_click);
}

fn on_input_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    focused_input: Option<Single<(Entity), (With<UIElement>, With<Focused>)>>,
) {
    if let Some(focused_input) = focused_input {
        commands
            .entity(focused_input.into_inner())
            .remove::<Focused>();
    }

    commands.entity(click.target).insert(Focused);
}

fn listen_keyboard_input_events(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    maybe_input: Option<Single<(&mut Text), (With<Focused>)>>,
) {
    if maybe_input.is_none() {
        return;
    }

    let (mut text) = maybe_input.unwrap().into_inner();
    for ev in events.read() {
        // We don't care about key releases, only key presses
        if ev.state == ButtonState::Released {
            continue;
        }
        match &ev.logical_key {
            // Handle pressing Enter to finish the input
            Key::Enter => {
                println!("Text input: {:#?}", &*text);
                text.clear();
            }
            // Handle pressing Backspace to delete last char
            Key::Backspace => {
                text.pop();
            }
            // Handle key presses that produce text characters
            Key::Character(input) => {
                // Ignore any input that contains control (special) characters
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }
                text.push_str(&input);
            }
            _ => {}
        }
    }
}
