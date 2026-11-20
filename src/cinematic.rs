use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{
    adventure::Area,
    farm_ui::{Choice, FarmMenu},
    session::Session,
    ui::{self, UiFonts},
    world::{IslandCamera, camera_offset},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Arrival,
    Opening,
    Watches,
    Festival,
}

impl Clip {
    pub fn duration(self) -> f32 {
        match self {
            Self::Arrival => 13.5,
            Self::Opening => 9.,
            Self::Watches => 11.,
            Self::Festival => 14.,
        }
    }

    fn subtitle(self, time: f32) -> &'static str {
        match self {
            Self::Arrival if time < 4.5 => {
                "Bo · Willkommen in Waldwinkel. Ab heute gehört dieser Hof dir."
            }
            Self::Arrival if time < 9. => {
                "Bo · Vor der Hütte steht eine alte Werkbank. Neben der Veranda ist Platz für deine ersten Beete."
            }
            Self::Arrival => {
                "Bo · Leg dein erstes Beet an. Komm dann in den Hafen. Die Farnwacht braucht heute Abend wieder Licht."
            }
            Self::Watches if time < 5.5 => {
                "Juna · Als die Glocken schwiegen, wiesen diese fünf Lichter den Menschen den Heimweg."
            }
            Self::Watches => {
                "Juna · Jetzt hat die Insel wieder eine Karte. Und neben jedem Licht steht deine Geschichte."
            }
            Self::Opening if time < 4.5 => {
                "Bo · Der Fischersteg trägt wieder. Heute teilen wir hier draußen das Essen aus deinem Garten."
            }
            Self::Opening => {
                "Fiete · Siehst du die Laternen an der Seeseite? Dort kannst du jetzt Hafenhering angeln."
            }
            Self::Festival if time < 4.5 => {
                "Bo · Früher haben wir am Wasser gewartet, bis irgendwo ein Licht anging."
            }
            Self::Festival if time < 9. => {
                "Juna · Jetzt kennen wir wieder jede Stimme zwischen Moor und Hafen."
            }
            Self::Festival => {
                "Bo · Die Leuchte gehört dem ganzen Dorf. Dein Hof bleibt dein Zuhause."
            }
        }
    }

    fn framing(self, time: f32) -> (Vec3, f32) {
        let progress = (time / self.duration()).clamp(0., 1.);
        let t = progress * progress * (3. - 2. * progress);
        match self {
            Self::Arrival => (
                Vec3::new(-6.9, 1.65, 0.04).lerp(Vec3::new(0.8, 0.89, 3.12), t),
                25.0_f32.lerp(12.3, t),
            ),
            Self::Watches => (
                Vec3::new(52.2, 1.28, 7.22).lerp(Vec3::new(55.8, 0.88, 6.81), t),
                19.3_f32.lerp(10.8, t),
            ),
            Self::Opening => (
                Vec3::new(57.7, 0.38, 26.5).lerp(Vec3::new(58.08, 0.69, 32.5), t),
                16.2_f32.lerp(12.6, t),
            ),
            Self::Festival => (
                Vec3::new(51.5, 1.5, 15.8).lerp(Vec3::new(56.8, 0.95, 11.2), t),
                22.0_f32.lerp(13.2, t),
            ),
        }
    }
}

#[derive(Component)]
pub struct SubtitlePane;

#[derive(Component)]
pub struct Subtitle;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands
        .spawn((
            SubtitlePane,
            crate::controller::Panel::Cinematic,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                bottom: Val::Px(22.),
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.)),
                ..default()
            },
            GlobalZIndex(125),
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    width: Val::Px(680.),
                    max_width: Val::Percent(95.),
                    padding: UiRect::axes(Val::Px(18.), Val::Px(10.)),
                    column_gap: Val::Px(14.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(17.)),
                BackgroundColor(Color::srgba(0.11, 0.19, 0.16, 0.91)),
            ))
            .with_children(|panel| {
                let (text, font, _) = fonts.body("", 17.);
                panel.spawn((
                    Subtitle,
                    text,
                    font,
                    TextColor(Color::srgb_u8(255, 241, 213)),
                    Node {
                        flex_grow: 1.,
                        ..default()
                    },
                ));
                panel
                    .spawn((Choice::Close, ui::dialog_button(119.)))
                    .with_child(fonts.button("Überspringen", 14.));
            });
        });
}

pub fn show(
    session: Res<Session>,
    mut menu: ResMut<FarmMenu>,
    time: Res<Time>,
    mut pane: Single<&mut Node, With<SubtitlePane>>,
    mut subtitle: Single<&mut Text, With<Subtitle>>,
    mut camera: Single<(&mut Projection, &mut Transform), With<IslandCamera>>,
) {
    let Some((clip, elapsed)) = menu.clip().filter(|_| session.playing()) else {
        pane.display = Display::None;
        return;
    };
    pane.display = Display::Flex;
    let caption = clip.subtitle(elapsed);
    if subtitle.0 != caption {
        subtitle.0 = caption.into();
    }
    let (center, height) = clip.framing(elapsed);
    let (projection, pose) = &mut *camera;
    if let Projection::Orthographic(ortho) = &mut **projection {
        ortho.scaling_mode = ScalingMode::AutoMin {
            min_width: height * 1.30,
            min_height: height,
        };
    }
    **pose = Transform::from_translation(center + camera_offset(Area::Island))
        .looking_at(center, Vec3::Y);
    menu.advance_clip(time.delta_secs());
}
