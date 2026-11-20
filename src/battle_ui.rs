use bevy::prelude::*;

use crate::{
    Adventure, GardenFocus, Notice,
    adventure::commit,
    controller::FocusEnabled,
    duel::{Card, Chapter, Emblem, Intent, Outcome, Ration},
    farm_ui::FarmMenu,
    garden::{Garden, SavePath},
    homestead::Product,
    session::{Checkpoint, Session},
    sound::Cue,
    ui::{self, UiFonts},
    waywatch::Trial,
    world::Gardener,
};

#[derive(Component)]
pub struct DuelPane;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DuelButton {
    Play(usize),
    End,
    Retreat,
    Stay,
    Pack,
    Eat(Ration),
}

#[derive(Component)]
pub enum DuelText {
    Level,
    Chapter,
    EnemyName,
    Friend,
    Foe,
    Intent,
    Energy,
    Piles,
    Rule,
    Pack,
    RationDetail(Ration),
    Message,
    Name(usize),
    Effects(usize),
    Cost(usize),
    End,
}

#[derive(Component)]
pub struct CardPicture(usize);

#[derive(Component)]
pub struct HealthFill(bool);

#[derive(Component)]
pub struct FlyingCard;
#[derive(Component)]
pub struct CombatImpact;
#[derive(Component)]
pub struct ImpactSymbol;

#[derive(Clone, Copy)]
struct Feedback {
    at: f32,
    card: Option<Card>,
    from: Vec2,
    to: Vec2,
    amount: u16,
    kind: Emblem,
    opponent: bool,
    sounded: bool,
}

#[derive(Resource, Default)]
pub struct CombatMotion(Option<Feedback>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Guardian {
    Trial(Trial),
    Dummy,
}

#[derive(Component)]
pub struct GuardianMotion(pub f32);

pub fn draw_trial_guardian(
    mut commands: Commands,
    assets: Res<AssetServer>,
    menu: Res<FarmMenu>,
    mut shown: Local<Option<(Guardian, Entity)>>,
) {
    let guardian = menu.battle().and_then(|battle| {
        if battle.training {
            Some(Guardian::Dummy)
        } else {
            battle.trial.map(Guardian::Trial)
        }
    });
    if shown.as_ref().map(|(id, _)| *id) == guardian {
        return;
    }
    if let Some((_, entity)) = shown.take() {
        commands.entity(entity).despawn();
    }
    if let Some(guardian) = guardian {
        let (model, point, ground) = match guardian {
            Guardian::Trial(trial) => {
                let point = trial.watch.point() + Vec2::new(-0.91, 1.86);
                (trial.enemy().model(), point, crate::world::exterior_ground(point))
            }
            Guardian::Dummy => ("training_dummy", crate::adventure::HOME + Vec2::new(1.89, -0.38), 0.06),
        };
        let actor = commands
            .spawn((
                SceneRoot(assets.load(format!("models/{model}.glb#Scene0"))),
                Transform::from_xyz(point.x, ground, point.y),
                GuardianMotion(ground),
            ))
            .id();
        *shown = Some((guardian, actor));
    }
}

pub fn animate_guardian(
    time: Res<Time>,
    menu: Res<FarmMenu>,
    motion: Res<CombatMotion>,
    mut guardian: Query<(&GuardianMotion, &mut Transform)>,
) {
    let now = time.elapsed_secs();
    let pulse = motion.0.filter(|fx| fx.opponent && menu.battle().is_some()).map_or(0., |fx| {
        let start = fx.at + if fx.card.is_some() { 0.21 } else { 0.08 };
        let progress = ((now - start) / 0.31).clamp(0., 1.);
        (std::f32::consts::PI * progress).sin()
    });
    for (ground, mut transform) in &mut guardian {
        transform.translation.y = ground.0 + 0.022 * (now * 3.5).sin().abs() + 0.16 * pulse;
        transform.rotation = Quat::from_rotation_y(-0.26) * Quat::from_rotation_z(-0.19 * pulse);
    }
}

#[derive(Resource)]
pub struct DeckIcons([Handle<Image>; Card::ALL.len()]);

impl FromWorld for DeckIcons {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self(Card::ALL.map(|card| assets.load(card.portrait())))
    }
}

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            DuelPane,
            crate::controller::Panel::Duel,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(22.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.10, 0.12, 0.13)),
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
                for (heading, header, field, bar) in [
                    ("Tanuki", None, DuelText::Friend, Some(true)),
                    (
                        "Dämmergänge",
                        Some(DuelText::Chapter),
                        DuelText::Level,
                        None,
                    ),
                    (
                        "Hüter",
                        Some(DuelText::EnemyName),
                        DuelText::Foe,
                        Some(false),
                    ),
                ] {
                    top.spawn((
                        Node {
                            min_width: Val::Px(142.),
                            max_width: Val::Percent(36.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.),
                            padding: UiRect::axes(Val::Px(16.), Val::Px(12.)),
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BackgroundColor(ui::cream()),
                        BorderColor(ui::stroke()),
                        BorderRadius::all(Val::Px(15.)),
                        ui::shadow(),
                    ))
                    .with_children(|panel| {
                        let mut title = panel.spawn(fonts.title(heading, 23.));
                        if let Some(header) = header {
                            title.insert(header);
                        }
                        panel.spawn((field, fonts.bold("", 15.)));
                        if let Some(player) = bar {
                            panel
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.),
                                        height: Val::Px(7.),
                                        ..default()
                                    },
                                    BorderRadius::all(Val::Px(4.)),
                                    BackgroundColor(Color::srgb_u8(224, 205, 165)),
                                ))
                                .with_child((
                                    HealthFill(player),
                                    Node {
                                        width: Val::Percent(100.),
                                        height: Val::Percent(100.),
                                        ..default()
                                    },
                                    BorderRadius::all(Val::Px(4.)),
                                    BackgroundColor(if player {
                                        Color::srgb_u8(86, 120, 75)
                                    } else {
                                        Color::srgb_u8(155, 81, 63)
                                    }),
                                ));
                        }
                    });
                }
            });
            veil.spawn((
                Node {
                    width: Val::Percent(100.),
                    max_width: Val::Px(1090.),
                    align_self: AlignSelf::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.),
                    padding: UiRect::all(Val::Px(13.)),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(42, 65, 52)),
                BorderColor(Color::srgb_u8(119, 139, 102)),
                BorderRadius::all(Val::Px(20.)),
                ui::shadow(),
            ))
            .with_children(|table| {
                table
                    .spawn(Node {
                        column_gap: Val::Px(8.),
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        for ration in Ration::ALL {
                            row.spawn((DuelButton::Eat(ration), ui::dialog_button(165.)))
                                .with_children(|item| {
                                    item.spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    })
                                    .with_children(|label| {
                                        label.spawn(fonts.button(ration.dish().name(), 13.));
                                        label.spawn((
                                            DuelText::RationDetail(ration),
                                            fonts.button("", 11.),
                                        ));
                                    });
                                });
                        }
                    });
                table
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: Val::Px(16.),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|status| {
                        status.spawn((DuelText::Intent, fonts.button("", 16.)));
                        status.spawn((DuelText::Energy, fonts.button("", 15.)));
                    });
                table
                    .spawn(Node {
                        column_gap: Val::Px(7.),
                        row_gap: Val::Px(7.),
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|hand| {
                        for index in 0..8 {
                            hand.spawn((
                                DuelButton::Play(index),
                                Button,
                                FocusEnabled(true),
                                Node {
                                    width: Val::Px(121.),
                                    max_width: Val::Percent(12.0),
                                    min_height: Val::Px(146.),
                                    padding: UiRect::all(Val::Px(5.)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexStart,
                                    row_gap: Val::Px(2.),
                                    border: UiRect::all(Val::Px(2.)),
                                    ..default()
                                },
                                BorderColor(Color::srgb_u8(165, 132, 91)),
                                BackgroundColor(ui::cream()),
                                BorderRadius::all(Val::Px(9.)),
                                ui::shadow(),
                            ))
                            .with_children(|card| {
                                card.spawn(Node {
                                    min_height: Val::Px(61.),
                                    ..default()
                                })
                                .with_children(|top| {
                                    top.spawn((
                                        CardPicture(index),
                                        ImageNode::default(),
                                        Node {
                                            width: Val::Percent(100.),
                                            height: Val::Px(61.),
                                            ..default()
                                        },
                                    ));
                                    top.spawn((
                                        Node {
                                            position_type: PositionType::Absolute,
                                            top: Val::Px(4.),
                                            left: Val::Px(4.),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            min_width: Val::Px(24.),
                                            min_height: Val::Px(24.),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BorderRadius::all(Val::Px(12.)),
                                        BackgroundColor(Color::srgb_u8(45, 69, 62)),
                                        BorderColor(Color::srgb_u8(243, 206, 133)),
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
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|feedback| {
                            feedback.spawn((DuelText::Rule, fonts.button("", 13.)));
                            feedback.spawn((DuelText::Message, fonts.button("", 14.)));
                            feedback.spawn((DuelText::Piles, fonts.button("", 13.)));
                        });
                        row.spawn(Node {
                            column_gap: Val::Px(7.),
                            ..default()
                        })
                        .with_children(|actions| {
                            actions
                                .spawn((DuelButton::Pack, ui::dialog_button(104.)))
                                .with_child((DuelText::Pack, fonts.button("Proviant", 13.)));
                            actions
                                .spawn((DuelButton::Retreat, ui::dialog_button(123.)))
                                .with_child(fonts.button("Zurückziehen", 13.));
                            actions
                                .spawn((DuelButton::Stay, ui::dialog_button(106.)))
                                .with_child(fonts.button("Bleiben", 14.));
                            actions
                                .spawn((DuelButton::End, ui::dialog_button(143.)))
                                .with_child((DuelText::End, fonts.button("Zug beenden", 14.)));
                        });
                    });
            });
            veil.spawn((
                FlyingCard,
                ImageNode::default(),
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ZIndex(3),
                ui::shadow(),
            ));
            veil.spawn((
                CombatImpact,
                fonts.title("", 39.),
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    min_width: Val::Px(110.),
                    padding: UiRect::axes(Val::Px(12.), Val::Px(4.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderRadius::all(Val::Px(16.)),
                ZIndex(4),
            )).with_child((
                ImpactSymbol, ImageNode::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-28.), top: Val::Px(9.),
                    width: Val::Px(32.), height: Val::Px(32.),
                    ..default()
                },
            ));
        });
}

pub fn input(
    session: Res<Session>,
    keys: Res<ButtonInput<KeyCode>>,
    controller: Res<crate::controller::Controller>,
    choices: Query<(&DuelButton, &Interaction), Changed<Interaction>>,
    mut menu: ResMut<FarmMenu>,
    mut adventure: ResMut<Adventure>,
    mut focus: ResMut<GardenFocus>,
    mut player: Single<(&mut Gardener, &mut Transform)>,
    mut garden: ResMut<Garden>,
    path: Res<SavePath>,
    mut notice: ResMut<Notice>,
    time: Res<Time>,
    mut motion: ResMut<CombatMotion>,
    places: Query<(&DuelButton, &GlobalTransform)>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut sounds: EventWriter<Cue>,
) {
    if !session.playing() {
        return;
    }
    let Some(battle) = menu.battle() else {
        return;
    };
    let press = choices
        .iter()
        .find_map(|(button, pressed)| (*pressed == Interaction::Pressed).then_some(*button));
    let keyboard = if keys.just_pressed(KeyCode::Escape) {
        Some(if battle.pack_open {
            DuelButton::Pack
        } else if battle.outcome == Outcome::Won {
            DuelButton::End
        } else if battle.retreat_pending {
            DuelButton::Stay
        } else {
            DuelButton::Retreat
        })
    } else if controller.provisions && (battle.pack_open || !battle.training && !battle.ration_used
        && Ration::ALL.into_iter().any(|ration| garden.stock(Product::Prepared(ration.dish())) > 0))
    {
        Some(DuelButton::Pack)
    } else if controller.end_turn || keys.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
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
    let battle = menu.battle_mut().expect("open battle");
    let now = time.elapsed_secs();
    let old = (battle.enemy_hp, battle.enemy_guard, battle.hp, battle.guard);
    let mut played = None;
    let (player, transform) = &mut *player;
    if battle.outcome == Outcome::Running {
        match (battle.retreat_pending, button) {
            (false, DuelButton::Retreat) => {
                battle.retreat_pending = true;
                return;
            }
            (true, DuelButton::Stay) => {
                battle.retreat_pending = false;
                return;
            }
            (true, DuelButton::End) => {}
            (true, _) => return,
            _ => {}
        }
    }
    match (battle.outcome, button) {
        (Outcome::Won, DuelButton::End | DuelButton::Retreat) => {
            adventure.focus = None;
            menu.back();
            return;
        }
        (Outcome::Lost, DuelButton::End | DuelButton::Retreat)
        | (Outcome::Running, DuelButton::End)
            if battle.training && (battle.retreat_pending || battle.outcome == Outcome::Lost) =>
        {
            menu.back();
            return;
        }
        (Outcome::Lost, DuelButton::End | DuelButton::Retreat)
        | (Outcome::Running, DuelButton::End)
            if battle.retreat_pending || battle.outcome == Outcome::Lost =>
        {
            let (area, position) = if battle.trial.is_some() {
                (adventure.area, player.position)
            } else {
                adventure.escape_duel(&mut garden, player, transform)
            };
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
        (Outcome::Running, DuelButton::Pack) if !battle.ration_used => {
            battle.pack_open = !battle.pack_open;
            return;
        }
        (Outcome::Running, DuelButton::Eat(ration)) if battle.pack_open => {
            let stock = Product::Prepared(ration.dish());
            if garden.stock(stock) > 0 && battle.eat(ration) {
                garden.take_stock(stock, 1);
                commit(
                    &garden,
                    &path,
                    &mut notice,
                    now,
                    battle.message.clone(),
                    None,
                    None,
                );
            }
            return;
        }
        (Outcome::Running, DuelButton::End) if battle.pack_open => {
            battle.pack_open = false;
            return;
        }
        (Outcome::Running, DuelButton::Play(index)) if !battle.pack_open => {
            let card = battle.hand.get(index).copied();
            if battle.play(index) {
                played = card;
            } else {
                return;
            }
        }
        (Outcome::Running, DuelButton::End) => battle.end_turn(),
        _ => return,
    }
    let strike = old.0.saturating_sub(battle.enemy_hp) + old.1.saturating_sub(battle.enemy_guard);
    let hurt = old.2.saturating_sub(battle.hp);
    let mend = battle.hp.saturating_sub(old.2);
    let shield = battle.guard.saturating_sub(old.3);
    let (kind, amount, opponent) = if hurt > 0 {
        (Emblem::Strike, hurt, false)
    } else if strike > 0 {
        (Emblem::Strike, strike, true)
    } else if mend > 0 {
        (Emblem::Heal, mend, false)
    } else if shield > 0 {
        (Emblem::Guard, shield, false)
    } else {
        (played.map_or(Emblem::Focus, Card::emblem), 0, false)
    };
    let from = window.single().ok()
        .and_then(|window| {
            let (_, t) = places.iter().find(|(id, _)| **id == button)?;
            let size = Vec2::new(window.physical_width() as f32, window.physical_height() as f32);
            Some((100. * t.translation().xy() / size).clamp(Vec2::ZERO, Vec2::splat(100.)))
        })
        .unwrap_or(Vec2::new(50., 76.));
    motion.0 = Some(Feedback {
        at: now, card: played, from,
        to: if opponent { Vec2::new(71., 39.) } else { Vec2::new(29., 40.) },
        amount, kind, opponent, sounded: false,
    });
    if played.is_some() {
        sounds.write(Cue::CardSlide);
    } else {
        sounds.write(Cue::Turn);
    }
    if battle.outcome != Outcome::Won {
        return;
    }
    let prize = if battle.training {
        if let Some(prologue) = &mut garden.prologue {
            prologue.sparred = true;
        }
        Some("Übung geschafft · Du kannst am Binder dein Deck zusammenstellen.".into())
    } else if let Some(trial) = battle.trial {
        trial.reward(&mut garden)
    } else if let Some((prize, card)) = garden.rpg.win(battle.level, battle.hp) {
        let coins = 7 + battle.level * 2;
        garden.farm.coins = garden.farm.coins.saturating_add(coins);
        let mut reward = format!("Gewonnen · +1 {} · +{coins} Münzen", prize.name());
        if let Some(card) = card {
            reward.push_str(&format!(" · Karte „{}“", card.name()));
        }
        Some(reward)
    } else {
        None
    };
    if let Some(reward) = prize {
        battle.message.clone_from(&reward);
        commit(&garden, &path, &mut notice, now, reward, None, None);
    }
}

pub fn animate_feedback(
    time: Res<Time>,
    menu: Res<FarmMenu>,
    session: Res<Session>,
    icons: Res<DeckIcons>,
    assets: Res<AssetServer>,
    mut motion: ResMut<CombatMotion>,
    mut flying: Single<(&mut Node, &mut ImageNode), With<FlyingCard>>,
    mut impact: Single<(&mut Node, &mut Text, &mut TextColor, &mut BackgroundColor),
        (With<CombatImpact>, Without<FlyingCard>)>,
    mut symbol: Single<&mut ImageNode, (With<ImpactSymbol>, Without<FlyingCard>)>,
    mut sounds: EventWriter<Cue>,
) {
    let (fly_node, art) = &mut *flying;
    let (hit_node, number, ink, halo) = &mut *impact;
    fly_node.display = Display::None;
    hit_node.display = Display::None;
    if menu.battle().is_none() || !session.playing() {
        motion.0 = None;
        return;
    }
    let Some(fx) = &mut motion.0 else { return };
    let age = time.elapsed_secs() - fx.at;
    if let Some(card) = fx.card.filter(|_| age < 0.36) {
        let t = (age / 0.36).clamp(0., 1.);
        let ease = 1. - (1. - t).powi(3);
        let center = fx.from.lerp(fx.to, ease) - Vec2::Y * (std::f32::consts::PI*t).sin()*12.;
        let width = 106. + 40. * (std::f32::consts::PI * t).sin();
        fly_node.display = Display::Flex;
        fly_node.left = Val::Percent(center.x);
        fly_node.top = Val::Percent(center.y);
        fly_node.margin = UiRect::left(Val::Px(-width*0.5));
        fly_node.width = Val::Px(width);
        fly_node.height = Val::Px(width*0.60);
        art.image = icons.0[card as usize].clone();
        art.color = Color::srgba(1., 1., 1., (1.45-t).min(1.));
    }
    let local = age - if fx.card.is_some() { 0.23 } else { 0.08 };
    if local < 0. {
        return;
    }
    if !fx.sounded {
        fx.sounded = true;
        let cue = match fx.kind {
            Emblem::Strike if fx.amount == 0 => Cue::CardGuard,
            Emblem::Strike => Cue::CardHit,
            Emblem::Guard => Cue::CardGuard,
            Emblem::Heal | Emblem::Spore => Cue::CardBloom,
            Emblem::Focus => Cue::CardGuard,
        };
        sounds.write(cue);
    }
    if local > 0.62 {
        motion.0 = None;
        return;
    }
    let t = local / 0.62;
    let fade = (1. - t).sqrt();
    let (label, color) = match fx.kind {
        Emblem::Strike if fx.amount > 0 =>
            (format!("−{}", fx.amount), if fx.opponent { (1., 0.77, 0.48) } else { (1., 0.56, 0.45) }),
        Emblem::Guard => (format!("+{} Schild", fx.amount), (0.69, 0.87, 1.)),
        Emblem::Heal => (format!("+{} Leben", fx.amount), (0.72, 1., 0.66)),
        Emblem::Spore => ("Sporen".into(), (0.83, 1., 0.64)),
        _ if fx.card.is_some() => ("Karte gespielt".into(), (1., 0.85, 0.58)),
        _ => ("Zugwechsel".into(), (1., 0.85, 0.58)),
    };
    hit_node.display = Display::Flex;
    hit_node.left = Val::Percent(fx.to.x);
    hit_node.top = Val::Percent(fx.to.y - 9.*t);
    hit_node.margin = UiRect::left(Val::Px(-55.));
    number.0 = label;
    ink.0 = Color::srgba(color.0, color.1, color.2, fade);
    halo.0 = Color::srgba(0.13, 0.20, 0.19, 0.76*fade);
    symbol.image = assets.load(fx.kind.texture());
    symbol.color = Color::srgba(1., 1., 1., fade);
}

pub fn update(
    menu: Res<FarmMenu>,
    session: Res<Session>,
    icons: Res<DeckIcons>,
    time: Res<Time>,
    garden: Res<Garden>,
    controller: Res<crate::controller::Controller>,
    mut veil: Single<&mut Node, With<DuelPane>>,
    mut fields: Query<(&DuelText, &mut Text)>,
    mut pictures: Query<(&CardPicture, &mut ImageNode)>,
    mut bars: Query<(&HealthFill, &mut Node), (Without<DuelPane>, Without<DuelButton>)>,
    mut buttons: Query<
        (&DuelButton, &Interaction, &mut Node, &mut BackgroundColor, &mut FocusEnabled),
        Without<DuelPane>,
    >,
) {
    let Some(battle) = menu.battle().filter(|_| session.playing()) else {
        veil.display = Display::None;
        return;
    };
    veil.display = Display::Flex;
    let ease = (1. - (-time.delta_secs()*13.).exp()).clamp(0., 1.);
    for (HealthFill(player), mut node) in &mut bars {
        let (current, max) = if *player { (battle.hp, battle.max_hp) }
            else { (battle.enemy_hp, battle.enemy_max) };
        let target = 100. * f32::from(current) / f32::from(max);
        let Val::Percent(previous) = node.width else { continue };
        let next = previous + (target-previous)*ease;
        if (target-previous).abs() > 0.001 {
            node.width = Val::Percent(if (target-next).abs() < 0.12 { target } else { next });
        }
    }
    if menu.is_changed() || session.is_changed() || controller.is_changed() {
        for (CardPicture(i), mut picture) in &mut pictures {
            if let Some(card) = battle.hand.get(*i) {
                let icon = &icons.0[*card as usize];
                if picture.image != *icon {
                    picture.image = icon.clone();
                }
            }
        }
        for (field, mut text) in &mut fields {
            let value = match *field {
                DuelText::Level => battle.trial.map_or_else(
                    || format!("Ebene {} · Zug {}", battle.level, battle.round),
                    |trial| format!("Prüfung {} · Zug {}", trial.rank + 1, battle.round),
                ),
                DuelText::Chapter if battle.training => "Hofübung".into(),
                DuelText::Chapter => battle
                    .trial
                    .map_or_else(
                        || Chapter::at(battle.level).name(),
                        |trial| trial.watch.name(),
                    )
                    .into(),
                DuelText::EnemyName if battle.training => "Strohwächter".into(),
                DuelText::EnemyName => battle.enemy.name().into(),
                DuelText::Friend => format!(
                    "{} / {} Leben · {} Schild",
                    battle.hp, battle.max_hp, battle.guard
                ),
                DuelText::Foe => format!(
                    "{} / {} Leben · {} Rüstung",
                    battle.enemy_hp, battle.enemy_max, battle.enemy_guard
                ),
                DuelText::Intent if battle.outcome == Outcome::Won => "Begegnung gewonnen".into(),
                DuelText::Intent if battle.outcome == Outcome::Lost => {
                    "Die Rastglocke ruft dich zurück".into()
                }
                DuelText::Intent if battle.retreat_pending => {
                    "Zum sicheren Ort zurückkehren?".into()
                }
                DuelText::Intent => match battle.intent() {
                    intent @ Intent::Attack(raw) => format!(
                        "Angriff {raw} · {} Schaden nach Schutz",
                        battle.incoming(intent)
                    ),
                    intent @ Intent::Flurry(raw) => format!(
                        "Doppelschlag 2 × {raw} · {} Schaden nach Schutz",
                        battle.incoming(intent)
                    ),
                    Intent::Brace(shield) => format!("Deckt sich · {shield} Rüstung"),
                    Intent::Mend(amount) => format!("Sammelt Kräfte · bis zu {amount} Leben"),
                    Intent::Charge { guard, damage } => {
                        format!("Lädt auf · {guard} Rüstung · droht mit {damage} Schaden")
                    }
                    Intent::Stagger => "Unterbrochen · Der aufgeladene Schlag fällt aus".into(),
                },
                DuelText::Energy if battle.outcome == Outcome::Running => {
                    format!("Kraft {}/{}", battle.energy, battle.energy_limit())
                }
                DuelText::Energy => String::new(),
                DuelText::Piles
                    if battle.outcome == Outcome::Running && !battle.retreat_pending =>
                {
                    let (draw, trash) = battle.piles();
                    let mut help = format!("{draw} im Zugstapel · {trash} abgelegt");
                    if controller.active {
                        help.push_str("\nA · Spielen   Y · Zug beenden   B · Zurück");
                        if !battle.training && !battle.ration_used
                            && Ration::ALL.into_iter().any(|r| garden.stock(Product::Prepared(r.dish())) > 0)
                        {
                            help.push_str("   X · Proviant");
                        }
                    }
                    help
                }
                DuelText::Piles => String::new(),
                DuelText::Rule if battle.pack_open => {
                    format!("Ein Proviant pro Kampf · {}", battle.enemy.provision_hint())
                }
                DuelText::RationDetail(ration) => {
                    let stock = garden.stock(Product::Prepared(ration.dish()));
                    let basic = format!("{} · ×{stock}", ration.effect());
                    match ration.matchup(battle.enemy) {
                        Some(bonus) => format!("{basic}\n{bonus}"),
                        None => basic,
                    }
                }
                DuelText::Rule => {
                    let conditions = battle.conditions();
                    if conditions.is_empty() {
                        battle.trial.filter(|_| battle.outcome == Outcome::Running)
                            .map_or("", |trial| trial.watch.rule()).into()
                    } else {
                        conditions
                    }
                },
                DuelText::Pack => if battle.pack_open {
                    "Schließen"
                } else {
                    "Proviant"
                }
                .into(),
                DuelText::Message => battle.message.clone(),
                DuelText::Name(i) => battle.hand.get(i).map_or("", |card| card.name()).into(),
                DuelText::Cost(i) => battle
                    .hand
                    .get(i)
                    .map_or_else(String::new, |card| card.cost().to_string()),
                DuelText::Effects(i) => battle
                    .hand
                    .get(i)
                    .map_or_else(String::new, |&card| battle.card_description(card)),
                DuelText::End if battle.retreat_pending => "Rückzug bestätigen".into(),
                DuelText::End if battle.training && battle.outcome != Outcome::Running => "Zum Kartenbinder".into(),
                DuelText::End => match battle.outcome {
                    Outcome::Running if controller.active => "Y · Zug beenden",
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
    }
    for (&button, &interaction, mut node, mut color, mut focus) in &mut buttons {
        let (visible, enabled) = match button {
            DuelButton::Play(i) => (
                battle.outcome == Outcome::Running
                    && !battle.pack_open
                    && !battle.retreat_pending
                    && i < battle.hand.len(),
                battle
                    .hand
                    .get(i)
                    .is_some_and(|card| card.cost() <= battle.energy),
            ),
            DuelButton::Retreat => (
                battle.outcome == Outcome::Running && !battle.retreat_pending,
                true,
            ),
            DuelButton::Stay => (battle.retreat_pending, true),
            DuelButton::Pack => (
                battle.outcome == Outcome::Running
                    && !battle.retreat_pending
                    && !battle.ration_used
                    && !battle.training
                    && Ration::ALL
                        .into_iter()
                        .any(|ration| garden.stock(Product::Prepared(ration.dish())) > 0),
                true,
            ),
            DuelButton::Eat(ration) => (
                battle.pack_open && garden.stock(Product::Prepared(ration.dish())) > 0,
                true,
            ),
            DuelButton::End => (true, true),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        focus.0 = visible && enabled;
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
