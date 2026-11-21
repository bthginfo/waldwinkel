use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    Adventure, Garden, Notice, Ready, campaign,
    feedback::{self, Tone},
    harbor, regions,
    session::Session,
    story::Guide,
    town::{self, Npc},
    ui::{SafeInsets, UiFonts},
};

#[derive(Clone)]
struct Objective {
    title: &'static str,
    deliver_to: &'static str,
    materials: String,
    ready: bool,
    guide: Option<Guide>,
}

impl Objective {
    fn id(&self) -> (&'static str, &'static str) {
        (self.title, self.deliver_to)
    }
}

fn accepted(garden: &Garden) -> Vec<Objective> {
    let mut tasks = Vec::new();
    for npc in [Npc::Mara, Npc::Fiete, Npc::Bo] {
        if let Some((title, materials, ready)) = town::tracking(npc, garden) {
            tasks.push(Objective { title, materials, ready, deliver_to: npc.name(), guide: Some(Guide::Town(npc)) });
        }
    }
    for (area, npc) in regions::Frontier::ALL.into_iter().zip([Npc::Runa, Npc::Nele, Npc::Edda]) {
        if let Some((title, materials, ready)) = regions::tracking(area, garden) {
            tasks.push(Objective { title, materials, ready, deliver_to: npc.name(), guide: Some(Guide::Regional(area)) });
        }
    }
    if garden.harbor.started && garden.harbor.phase != harbor::Phase::Open {
        tasks.push(Objective {
            title: garden.harbor.phase.label(),
            deliver_to: "Bautisch am Fischersteg",
            materials: harbor::supplies(garden),
            ready: harbor::has_materials(garden),
            guide: Some(Guide::Pier),
        });
    }
    for bell in regions::Bell::ALL {
        let (materials, action, ready) = bell.describe(garden);
        if action == "Glocke reparieren" {
            tasks.push(Objective {
                title: "Wegglocke reparieren",
                deliver_to: bell.name(),
                materials,
                ready,
                guide: None,
            });
        }
    }
    if let Some(order) = garden.ledger.active() {
        tasks.push(Objective {
            title: order.name(),
            deliver_to: "Hafenaushang",
            materials: order.detail(garden),
            ready: order.can_work(garden),
            guide: None,
        });
    }
    tasks
}

#[derive(Default)]
pub struct Tracking {
    slot: Option<usize>,
    next_check: f32,
    tasks: Vec<Objective>,
    announced: Vec<(&'static str, &'static str)>,
}

impl Tracking {
    fn replace(&mut self, slot: usize, tasks: Vec<Objective>) {
        if self.slot != Some(slot) {
            self.slot = Some(slot);
            self.announced = tasks.iter().filter(|task| task.ready).map(Objective::id).collect();
        } else {
            // Drop a ready mark if the objective is completed or supplies were spent elsewhere.
            self.announced.retain(|&id| tasks.iter().any(|task| task.ready && task.id() == id));
        }
        self.tasks = tasks;
    }

    fn notification(&mut self) -> Option<String> {
        let task = self.tasks.iter().find(|task| task.ready && !self.announced.contains(&task.id()))?;
        self.announced.push(task.id());
        Some(format!("Abgabebereit · {} · {}", task.title, task.deliver_to))
    }
}

#[derive(Component)]
pub struct QuestPanel;

#[derive(Component)]
pub struct QuestLine(usize);

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>) {
    commands.spawn((
        QuestPanel,
        GlobalZIndex(3),
        FocusPolicy::Pass,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: Val::Px(232.),
            padding: UiRect::axes(Val::Px(12.), Val::Px(8.)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(7.),
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.25, 0.21, 0.93)),
        BorderRadius::all(Val::Px(13.)),
    )).with_children(|panel| {
        panel.spawn(fonts.button("Aufträge", 15.));
        for i in 0..2 {
            panel.spawn((QuestLine(i), fonts.button("", 13.), Node { display: Display::None, ..default() }));
        }
    });
}

pub fn update(
    time: Res<Time>,
    session: Res<Session>,
    ready: Res<Ready>,
    garden: Res<Garden>,
    adventure: Res<Adventure>,
    layers: (Res<crate::farm_ui::FarmMenu>, Res<crate::town::Dialog>, Res<crate::wheel::Wheel>,
        Res<crate::night::Night>, Res<crate::decorate::Layout>),
    window: Single<&Window, With<PrimaryWindow>>,
    insets: Res<SafeInsets>,
    scale: Res<UiScale>,
    mut tracking: Local<Tracking>,
    mut notice: ResMut<Notice>,
    mut panel: Single<&mut Node, (With<QuestPanel>, Without<QuestLine>)>,
    mut lines: Query<(&QuestLine, &mut Text, &mut Node), Without<QuestPanel>>,
) {
    let (menu, dialog, wheel, rest, layout) = layers;
    let Some(slot) = session.slot() else {
        tracking.slot = None;
        panel.display = Display::None;
        return;
    };
    let now = time.elapsed_secs();
    if tracking.slot != Some(slot) || now >= tracking.next_check {
        tracking.replace(slot, accepted(&garden));
        tracking.next_check = now + 0.15;
    }
    let exploring = ready.0 && session.playing() && !menu.open() && !dialog.open()
        && !wheel.open() && !rest.active() && !layout.open();
    if exploring && (notice.until <= now || notice.toast.is_none())
        && let Some(message) = tracking.notification()
    {
        *notice = feedback::make(Tone::Success, message, now);
    }
    let tablet = exploring && !adventure.map && adventure.area.outdoors()
        && window.width().min(window.height()) >= 590.;
    if !tablet {
        panel.display = Display::None;
        return;
    }

    let mut ordered: Vec<_> = tracking.tasks.iter().collect();
    ordered.sort_by_key(|task| (adventure.tracked.is_none() || task.guide != adventure.tracked, !task.ready));
    let mut labels: Vec<_> = ordered.into_iter().take(2).map(|task| {
        if task.ready {
            format!("{}\nAbgabebereit · {}", task.title, task.deliver_to)
        } else {
            format!("{}\n{}", task.title, task.materials)
        }
    }).collect();
    if labels.len() < 2 && !tracking.tasks.iter().any(|task| task.guide == Some(Guide::Pier)
        && campaign::next(&garden) == campaign::Step::Pier)
    {
        let step = campaign::next(&garden);
        if step != campaign::Step::Done {
            let title = step.title();
            let destination = step.target(&garden).map(|target| target.name());
            let goal = match step {
                campaign::Step::Watch(site) => site.action(&garden),
                campaign::Step::Gate(_) => "Zugang öffnen",
                campaign::Step::Echo(_) => "Klangstein stimmen",
                _ => destination.filter(|&name| name != title).unwrap_or("Im Journal ansehen"),
            };
            labels.push(format!("{}\n{goal}", step.title()));
        }
    }
    panel.display = if labels.is_empty() { Display::None } else { Display::Flex };
    let safe = |i: usize| insets.0[i] / window.scale_factor() / scale.0;
    panel.right = Val::Px(14. + crate::navigation_ui::MAP_WIDTH + 12. + 9. + safe(2));
    panel.top = Val::Px(85. + safe(1));
    for (QuestLine(i), mut text, mut node) in &mut lines {
        node.display = if let Some(label) = labels.get(*i) {
            if &text.0 != label {
                text.0.clone_from(label);
            }
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_a_readiness_toast_once_per_transition_and_baselines_loaded_saves() {
        let mut tracker = Tracking::default();
        let mut quest = Objective {
            title: "Für Maras Küche", deliver_to: "Mara", materials: String::new(),
            ready: true, guide: Some(Guide::Town(Npc::Mara)),
        };
        tracker.replace(0, vec![quest.clone()]);
        assert_eq!(tracker.notification(), None);
        quest.ready = false;
        tracker.replace(0, vec![quest.clone()]);
        assert_eq!(tracker.notification(), None);
        quest.ready = true;
        tracker.replace(0, vec![quest.clone()]);
        assert!(tracker.notification().unwrap().contains("Mara"));
        tracker.replace(0, vec![quest.clone()]);
        assert_eq!(tracker.notification(), None);
        tracker.replace(1, vec![quest]);
        assert_eq!(tracker.notification(), None);
    }
}
