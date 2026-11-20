use bevy::prelude::*;

use crate::{
    Adventure, GardenFocus, Notice,
    adventure::commit,
    duel::{Emblem, Intent, Outcome},
    farm_ui::FarmMenu,
    garden::{Garden, SavePath},
    session::{Checkpoint, Session},
    ui::{self, UiFonts},
    world::Gardener,
};

#[derive(Component)]
pub struct DuelPane;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DuelButton {
    Play(usize),
    End,
    Retreat,
}

#[derive(Component)]
pub enum DuelText {
    Level,
    Friend,
    Foe,
    Intent,
    Message,
    Name(usize),
    Effects(usize),
    Cost(usize),
    End,
}

#[derive(Component)]
pub struct CardPicture(usize);

#[derive(Resource)]
pub struct DeckIcons([Handle<Image>; Emblem::ALL.len()]);

impl FromWorld for DeckIcons {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self(Emblem::ALL.map(|emblem| assets.load(emblem.texture())))
    }
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            DuelPane,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(17.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.09, 0.10, 0.17, 0.23)),
            GlobalZIndex(91),
        ))
        .with_children(|veil| {
            veil.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(12.),
                ..default()
            })
            .with_children(|top| {
                for (heading, field) in [
                    ("Tanuki", DuelText::Friend),
                    ("Dämmergänge", DuelText::Level),
                    ("Hüter", DuelText::Foe),
                ] {
                    top.spawn((
                        Node {
                            min_width: Val::Px(142.),
                            max_width: Val::Percent(37.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.),
                            padding: UiRect::axes(Val::Px(14.), Val::Px(11.)),
                            ..default()
                        },
                        BackgroundColor(ui::cream()),
                        BorderRadius::all(Val::Px(15.)),
                        ui::shadow(),
                    ))
                    .with_children(|panel| {
                        panel.spawn(fonts.title(heading, 20.));
                        panel.spawn((field, fonts.bold("", 14.)));
                    });
                }
            });
            veil.spawn((
                Node {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(7.),
                    padding: UiRect::all(Val::Px(12.)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.22, 0.24, 0.20, 0.94)),
                BorderRadius::all(Val::Px(19.)),
                ui::shadow(),
            ))
            .with_children(|table| {
                table.spawn((DuelText::Intent, fonts.button("", 16.)));
                table
                    .spawn(Node {
                        column_gap: Val::Px(7.),
                        row_gap: Val::Px(7.),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|hand| {
                        for index in 0..8 {
                            hand.spawn((
                                DuelButton::Play(index),
                                Button,
                                Node {
                                    width: Val::Px(121.),
                                    min_height: Val::Px(115.),
                                    padding: UiRect::all(Val::Px(8.)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    row_gap: Val::Px(2.),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BorderColor(Color::srgb_u8(166, 148, 109)),
                                BackgroundColor(ui::cream()),
                                BorderRadius::all(Val::Px(9.)),
                            ))
                            .with_children(|card| {
                                card.spawn(Node {
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|top| {
                                    top.spawn((
                                        CardPicture(index),
                                        ImageNode::default(),
                                        Node {
                                            width: Val::Px(34.),
                                            height: Val::Px(34.),
                                            ..default()
                                        },
                                    ));
                                    top.spawn((
                                        Node {
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            min_width: Val::Px(23.),
                                            min_height: Val::Px(23.),
                                            ..default()
                                        },
                                        BorderRadius::all(Val::Px(12.)),
                                        BackgroundColor(Color::srgb_u8(73, 101, 76)),
                                    ))
                                    .with_child((DuelText::Cost(index), fonts.button("", 13.)));
                                });
                                card.spawn((DuelText::Name(index), fonts.title("", 15.)));
                                card.spawn((DuelText::Effects(index), fonts.body("", 12.)));
                            });
                        }
                    });
                table
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(9.),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((DuelText::Message, fonts.button("", 14.)));
                        row.spawn(Node {
                            column_gap: Val::Px(7.),
                            ..default()
                        })
                        .with_children(|actions| {
                            actions
                                .spawn((DuelButton::Retreat, ui::dialog_button(123.)))
                                .with_child(fonts.button("Zurückziehen", 13.));
                            actions
                                .spawn((DuelButton::End, ui::dialog_button(143.)))
                                .with_child((DuelText::End, fonts.button("Zug beenden", 14.)));
                        });
                    });
            });
        });
}

pub fn input(
    session: Res<Session>,
    keys: Res<ButtonInput<KeyCode>>,
    choices: Query<(&DuelButton, &Interaction), Changed<Interaction>>,
    mut menu: ResMut<FarmMenu>,
    mut adventure: ResMut<Adventure>,
    mut focus: ResMut<GardenFocus>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
    mut notice: ResMut<Notice>,
    time: Res<Time>,
) {
    if !session.playing() {
        return;
    }
    let Some(battle) = menu.battle_mut() else {
        return;
    };
    let press = choices
        .iter()
        .find_map(|(button, pressed)| (*pressed == Interaction::Pressed).then_some(*button));
    let keyboard = if keys.just_pressed(KeyCode::Escape) {
        Some(if battle.outcome == Outcome::Won {
            DuelButton::End
        } else {
            DuelButton::Retreat
        })
    } else if keys.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
        Some(DuelButton::End)
    } else {
        [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
        ]
        .into_iter()
        .position(|key| keys.just_pressed(key))
        .map(DuelButton::Play)
    };
    let Some(button) = press.or(keyboard) else {
        return;
    };
    let now = time.elapsed_secs();
    let (player, transform) = &mut *player;
    match (battle.outcome, button) {
        (Outcome::Won, DuelButton::End | DuelButton::Retreat) => {
            adventure.focus = None;
            menu.close();
            return;
        }
        (Outcome::Lost, DuelButton::End | DuelButton::Retreat) | (_, DuelButton::Retreat) => {
            let (area, position) = adventure.escape_duel(&mut garden, player, transform);
            garden.checkpoint = Some(Checkpoint::new(area, position));
            *focus = GardenFocus::default();
            menu.close();
            commit(
                &garden,
                &path,
                &mut notice,
                now,
                "Du bist an einem sicheren Ort.".into(),
                None,
                None,
            );
            return;
        }
        (Outcome::Running, DuelButton::Play(index)) => {
            battle.play(index);
        }
        (Outcome::Running, DuelButton::End) => battle.end_turn(),
        _ => return,
    }
    if battle.outcome == Outcome::Won
        && let Some((prize, card)) = garden.rpg.win(battle.level, battle.hp)
    {
        let coins = 7 + battle.level * 2;
        garden.farm.coins = garden.farm.coins.saturating_add(coins);
        let mut reward = format!("Gewonnen · +1 {} · +{coins} Münzen", prize.name());
        if let Some(card) = card {
            reward.push_str(&format!(" · Karte „{}“", card.name()));
        }
        battle.message.clone_from(&reward);
        commit(&garden, &path, &mut notice, now, reward, None, None);
    }
}

pub fn update(
    menu: Res<FarmMenu>,
    session: Res<Session>,
    icons: Res<DeckIcons>,
    mut veil: Single<&mut Node, With<DuelPane>>,
    mut fields: Query<(&DuelText, &mut Text)>,
    mut pictures: Query<(&CardPicture, &mut ImageNode)>,
    mut buttons: Query<
        (&DuelButton, &Interaction, &mut Node, &mut BackgroundColor),
        Without<DuelPane>,
    >,
) {
    let Some(battle) = menu.battle().filter(|_| session.playing()) else {
        veil.display = Display::None;
        return;
    };
    veil.display = Display::Flex;
    for (CardPicture(i), mut picture) in &mut pictures {
        if let Some(card) = battle.hand.get(*i) {
            let icon = &icons.0[card.emblem() as usize];
            if picture.image != *icon {
                picture.image = icon.clone();
            }
        }
    }
    for (field, mut text) in &mut fields {
        let value = match *field {
            DuelText::Level => format!("Ebene {} · Zug {}", battle.level, battle.round),
            DuelText::Friend => format!(
                "{} / {} Leben · {} Schild",
                battle.hp, battle.max_hp, battle.guard
            ),
            DuelText::Foe => format!(
                "{} · {} / {} Leben · {} Schild",
                battle.enemy.name(),
                battle.enemy_hp,
                battle.enemy_max,
                battle.enemy_guard
            ),
            DuelText::Intent if battle.outcome == Outcome::Won => "Begegnung gewonnen".into(),
            DuelText::Intent if battle.outcome == Outcome::Lost => {
                "Die Rastglocke ruft dich zurück".into()
            }
            DuelText::Intent => {
                let plan = match battle.intent() {
                    Intent::Attack(damage) => {
                        format!(
                            "Angriff {damage} · {} Schaden nach Schutz",
                            battle.incoming(damage)
                        )
                    }
                    Intent::Brace(shield) => format!("{} Schild", shield),
                };
                format!("Kraft {} · Absicht des Hüters · {plan}", battle.energy)
            }
            DuelText::Message => battle.message.clone(),
            DuelText::Name(i) => battle.hand.get(i).map_or("", |card| card.name()).into(),
            DuelText::Cost(i) => battle
                .hand
                .get(i)
                .map_or_else(String::new, |card| card.cost().to_string()),
            DuelText::Effects(i) => battle
                .hand
                .get(i)
                .map_or_else(String::new, |card| card.description()),
            DuelText::End => match battle.outcome {
                Outcome::Running => "Zug beenden",
                Outcome::Won => "Weiter erkunden",
                Outcome::Lost => "Zurück zur Rast",
            }
            .into(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (&button, &interaction, mut node, mut color) in &mut buttons {
        let (visible, enabled) = match button {
            DuelButton::Play(i) => (
                battle.outcome == Outcome::Running && i < battle.hand.len(),
                battle
                    .hand
                    .get(i)
                    .is_some_and(|card| card.cost() <= battle.energy),
            ),
            DuelButton::Retreat => (battle.outcome == Outcome::Running, true),
            DuelButton::End => (true, true),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        color.0 = match button {
            DuelButton::Play(_) if !enabled => Color::srgb_u8(202, 196, 176),
            DuelButton::Play(_) if interaction != Interaction::None => {
                Color::srgb_u8(210, 222, 173)
            }
            DuelButton::Play(_) => ui::cream(),
            _ => ui::button_color(interaction, enabled),
        };
    }
}
