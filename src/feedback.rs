use bevy::{prelude::*, ui::FocusPolicy};

use crate::{Notice, night::Night, ui::UiFonts};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Success,
    Problem,
}

pub fn make(tone: Tone, message: String, now: f32) -> Notice {
    Notice {
        message,
        until: now + Notice::DURATION,
        toast: Some(tone),
        ..default()
    }
}

#[derive(Component)]
pub struct Frame;
#[derive(Component)]
pub struct Message;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            Frame,
            GlobalZIndex(160),
            FocusPolicy::Pass,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: Val::Percent(22.),
                right: Val::Percent(22.),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_child((
            Message,
            fonts.button("", 16.),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                max_width: Val::Px(465.),
                padding: UiRect::axes(Val::Px(18.), Val::Px(9.)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(14.)),
            FocusPolicy::Pass,
        ));
}

pub fn update(
    time: Res<Time>,
    notice: Res<Notice>,
    night: Res<Night>,
    mut frame: Single<&mut Node, (With<Frame>, Without<Message>)>,
    mut line: Single<(&mut Text, &mut BackgroundColor, &mut TextColor), With<Message>>,
) {
    let remaining = notice.until - time.elapsed_secs();
    let Some(tone) = notice.toast.filter(|_| remaining > 0. && !night.active()) else {
        frame.display = Display::None;
        return;
    };
    frame.display = Display::Flex;
    let age = (Notice::DURATION - remaining).max(0.);
    let settle = (1. - age / 0.16).clamp(0., 1.);
    let alpha = (remaining / 0.27).clamp(0., 1.) * (1. - settle * settle);
    frame.top = Val::Px(79. - 7. * settle);
    let (label, background, ink) = &mut *line;
    if label.0 != notice.message {
        label.0.clone_from(&notice.message);
    }
    let rgb = match tone {
        Tone::Success => (0.13, 0.30, 0.23),
        Tone::Problem => (0.40, 0.25, 0.18),
    };
    background.0 = Color::srgba(rgb.0, rgb.1, rgb.2, alpha * 0.97);
    ink.0 = Color::srgba(1., 0.96, 0.83, alpha);
}
