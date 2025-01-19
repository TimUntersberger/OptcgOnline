use bevy::prelude::*;

#[derive(Default, Component)]
#[require(Sprite, Transform, RightClickable, LeftClickable)]
pub(crate) struct Card {
    pub(crate) asset: &'static str,
}

#[derive(Component, Default)]
pub(crate) enum EntityKind {
    #[default]
    Card,
}

#[derive(Component, Default)]
pub(crate) struct RightClickable;

#[derive(Component, Default)]
pub(crate) struct Player1;

#[derive(Component, Default)]
pub(crate) struct Player2;

#[derive(Component, Default)]
pub(crate) struct LeaderCard;

#[derive(Component, Default)]
pub(crate) struct LeftClickable;

#[derive(Component, Default)]
pub(crate) struct Draggable;

#[derive(Component, Default)]
pub(crate) struct InPlay;

#[derive(Component, Default)]
pub(crate) struct InTrash;

#[derive(Component, Default)]
pub(crate) struct InLifeFaceDown;

#[derive(Component, Default)]
pub(crate) struct InLifeFaceUp;

#[derive(Component, Default)]
pub(crate) struct Tapped;

#[derive(Component, Default)]
pub(crate) struct DonCard;

#[derive(Component, Default)]
pub(crate) struct CharacterCard;

#[derive(Debug)]
pub(crate) enum PlayAction {
    PlayCharacter { card_entity: Entity },
    TapCharacter { card_entity: Entity },
    UntapCharacter { card_entity: Entity },
}