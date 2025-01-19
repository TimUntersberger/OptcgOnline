use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::{input_just_pressed, input_pressed},
    log::{Level, LogPlugin},
    prelude::*,
    scene::ron::de,
    state::commands,
    window::{PresentMode, WindowTheme},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_simple_text_input::{
    TextInput, TextInputSubmitEvent, TextInputSystem, TextInputTextColor, TextInputTextFont,
};
use card_spawner::{CardSpawnerOpenEvent, CardSpawnerPlugin};
use components::*;
use ui::text_input;

mod card_spawner;
mod components;
mod ui;

const CARD_SIZE_RATIO: (f32, f32) = (1.0, 1.4);
const CARD_SIZE: f32 = 120.0;
const DRAGGING_ENABLED: bool = false;
const CARD_SIZE_2D: Vec2 = Vec2::new(CARD_SIZE_RATIO.0 * CARD_SIZE, CARD_SIZE_RATIO.1 * CARD_SIZE);
const CHARACTER_CARD_TEMPLATE: Card = Card {
    asset: "character.png",
};

#[derive(Resource, Default)]
struct PlayActionQueue(Vec<PlayAction>);

#[derive(Resource, Default)]
struct State {
    cursor_position: Vec2,
}

fn main() {
    App::new()
        .init_resource::<State>()
        .init_resource::<PlayActionQueue>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "I am a window!".into(),
                        name: Some("bevy.app".into()),
                        resolution: (1920., 1080.).into(),
                        present_mode: PresentMode::AutoVsync,
                        // Tells Wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        // This will spawn an invisible window
                        // The window will be made visible in the make_visible() system after 3 frames.
                        // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
                    custom_layer: |_| None,
                }),
        )
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(text_input::TextInputPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(CardSpawnerPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_components_system.run_if(input_just_pressed(KeyCode::F2)),
                fetch_cursor_position_system,
                debug_remove_character.run_if(input_just_pressed(KeyCode::Backspace)),
                organize_in_play_characters_system.run_if(input_just_pressed(KeyCode::F3)),
            ),
        )
        .run();
}

fn on_drag_move(
    trigger: Trigger<Pointer<Drag>>,
    mut transforms: Query<&mut Transform, With<Draggable>>,
) {
    let mut transform = transforms.get_mut(trigger.entity()).unwrap();

    transform.translation.x += trigger.delta.x;
    transform.translation.y -= trigger.delta.y;
}

fn on_right_click_tap(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut entities: Query<(&EntityKind, &mut Transform, Option<&Tapped>), With<RightClickable>>,
) {
    if trigger.button == PointerButton::Secondary {
        if let Ok((kind, mut transform, maybe_tapped)) = entities.get_mut(trigger.entity()) {
            match kind {
                EntityKind::Card => {
                    match maybe_tapped {
                        Some(_) => {
                            transform.rotation = Quat::IDENTITY;
                            commands.entity(trigger.entity()).remove::<Tapped>();
                        },
                        None => {
                            transform.rotate(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2));
                            commands.entity(trigger.entity()).insert(Tapped);
                        },
                    }
                }
            }
        }
    }
}

fn debug_remove_character(
    mut commands: Commands,
    mut state: ResMut<State>,
    query: Query<(Entity, &Transform, &Sprite, &EntityKind), With<Card>>,
) {
    for (entity, transform, sprite, kind) in &query {
        if is_cursor_over_sprite(state.cursor_position, sprite, transform) {
            match kind {
                EntityKind::Card => {
                    commands.entity(entity).despawn();
                }
            }

            break;
        }
    }
}

fn debug_components_system(world: &World, query: Query<Entity>) {
    for entity in &query {
        debug!(
            "{0:#?}",
            world
                .inspect_entity(entity)
                .map(|info| info.name())
                .collect::<Vec<_>>()
        );
    }
}

fn organize_in_play_characters_system(
    mut state: ResMut<State>,
    mut char_query: Query<
        (&mut Transform, &Sprite, Option<&Tapped>),
        (With<InPlay>, With<CharacterCard>, Without<DonCard>),
    >,
    mut don_query: Query<
        (&mut Transform, &Sprite, Option<&Tapped>),
        (With<InPlay>, With<DonCard>, Without<CharacterCard>),
    >,
) {
    debug!("Organizing cards");
    let slot_width = CARD_SIZE_2D.y;

    let mut char_cards = char_query.iter_mut().collect::<Vec<_>>();
    for i in 0..char_cards.len() {
        let left_most = -((char_cards.len() - 1) as f32 * slot_width / 2.0);

        debug!(
            "Setting card at {i} to coordinate x = {0}",
            left_most + (slot_width * i as f32)
        );
        char_cards[i].0.translation.x = left_most + (slot_width * i as f32);
        char_cards[i].0.translation.y = 0.0;
    }

    let mut don_cards = don_query.iter_mut().collect::<Vec<_>>();
    let slot_width = CARD_SIZE_2D.y / 2.0;
    for i in 0..don_cards.len() {
        let left_most = -((don_cards.len() - 1) as f32 * slot_width / 2.0);

        debug!(
            "Setting card at {i} to coordinate x = {0}",
            left_most + (slot_width * i as f32)
        );
        don_cards[i].0.translation.x = left_most + (slot_width * i as f32);
        don_cards[i].0.translation.y = CARD_SIZE_2D.y * 1.5;
    }
}

fn is_cursor_over_sprite(cursor_position: Vec2, sprite: &Sprite, transform: &Transform) -> bool {
    if let Some(custom_size) = sprite.custom_size {
        let left = transform.translation.x - custom_size.x / 2.0;
        let right = transform.translation.x + custom_size.x / 2.0;
        let top = transform.translation.y - custom_size.y / 2.0;
        let bottom = transform.translation.y + custom_size.y / 2.0;
        let inside_horizontal = cursor_position.x >= left && cursor_position.x <= right;
        let inside_vertical = cursor_position.y >= top && cursor_position.y <= bottom;

        return inside_horizontal && inside_vertical;
    } else {
        return false;
    }
}

// fn handle_right_click_system(
//     state: ResMut<State>,
//     mut queue: ResMut<PlayActionQueue>,
//     query: Query<
//         (Entity, &Transform, &Sprite, &EntityKind, Option<&TapState>),
//         With<RightClickable>,
//     >,
// ) {
//     debug!(
//         "Handling right click! Found {0} clickable entities",
//         query.iter().len()
//     );
//     for (entity, transform, sprite, kind, tap_state) in &query {
//         if is_cursor_over_sprite(state.cursor_position, sprite, transform) {
//             match kind {
//                 EntityKind::Card => {
//                     debug!("Card got right clicked!");
//                     queue.0.push(match tap_state.unwrap() {
//                         TapState::Tapped => PlayAction::UntapCharacter {
//                             card_entity: entity,
//                         },
//                         TapState::Untapped => PlayAction::TapCharacter {
//                             card_entity: entity,
//                         },
//                     });
//                 }
//             }

//             break;
//         }
//     }
// }

fn fetch_cursor_position_system(
    mut state: ResMut<State>,
    mut evr_cursor: EventReader<CursorMoved>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    for ev in evr_cursor.read() {
        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, ev.position)
            .ok()
            .map(|ray| ray.origin.truncate())
        {
            //debug!("Cursor moved to {world_position}");
            state.cursor_position = world_position;
        }
    }
}

fn spawn_card(commands: &mut Commands, asset: Handle<Image>, x: f32, y: f32) -> Entity {
    commands
        .spawn(Card::default())
        .insert(EntityKind::Card)
        .insert(CharacterCard)
        .insert(Sprite {
            image: asset,
            custom_size: Some(CARD_SIZE_2D),
            ..default()
        })
        .insert(Transform::from_xyz(x, y, 0.0))
        .id()
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut action_queue: ResMut<PlayActionQueue>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            image: asset_server.load("playsheet_black.png"),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, -250.0, -1.0),
            scale: Vec3::new(1.2, 0.45, 1.0),
            ..default()
        },
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load("playsheet_black.png"),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 250.0, -1.0),
            scale: Vec3::new(1.2, 0.45, 1.0),
            ..default()
        },
    ));

    for i in 0..10 {
        action_queue.0.push(PlayAction::PlayCharacter {
            card_entity: commands
                .spawn(Card::default())
                .insert(EntityKind::Card)
                .insert(Player1)
                .insert(DonCard)
                .insert(Draggable)
                .insert(Sprite {
                    image: asset_server.load("don.png"),
                    custom_size: Some(CARD_SIZE_2D),
                    ..default()
                })
                .insert(Transform::from_xyz(-100.0, -100.0, 0.0))
                .observe(on_drag_move)
                .observe(on_right_click_tap)
                .id(),
        });
    }

    // for i in 0..10 {
    //     action_queue.0.push(PlayAction::PlayCharacter {
    //         card_entity: commands
    //             .spawn(Card::default())
    //             .insert(EntityKind::Card)
    //             .insert(Player2)
    //             .insert(InPlay)
    //             .insert(DonCard)
    //             .insert(Sprite {
    //                 image: asset_server.load("don.png"),
    //                 custom_size: Some(CARD_SIZE_2D),
    //                 ..default()
    //             })
    //             .insert(Transform::from_xyz(0.0, 0.0, 0.0))
    //             .id()
    //     });
    // }
}
