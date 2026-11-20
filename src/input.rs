use bevy::{
    input::{
        ButtonState,
        mouse::{MouseButtonInput, MouseWheel},
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::{AppLifecycle, PrimaryWindow, WindowFocused},
};

use crate::{
    Controls, Ready,
    session::Session,
    town::Dialog,
    ui::{HudLayout, Utility},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pointer {
    Finger(u64),
    Mouse,
}

#[derive(Clone, Copy)]
struct Tap {
    id: Pointer,
    start: Vec2,
    action_button: bool,
    utility: Option<Utility>,
}

struct Stick {
    id: Pointer,
    anchor: Vec2,
    position: Vec2,
}

#[derive(Resource, Default)]
pub struct Gesture {
    stick: Option<Stick>,
    tap: Option<Tap>,
    pub stick_axis: Vec2,
    fingers: Vec<(u64, Vec2)>,
}

impl Gesture {
    pub fn action_held(&self) -> bool {
        self.tap.is_some_and(|tap| tap.action_button)
    }

    fn press(&mut self, pointer: Pointer, at: Vec2, layout: HudLayout) {
        if layout.on_stick(at) {
            if self.stick.is_none() {
                self.stick = Some(Stick {
                    id: pointer,
                    anchor: at,
                    position: at,
                });
            }
        } else if self.tap.is_none()
            && (layout.utility(at).is_some() || layout.on_action(at) || !layout.on_other_ui(at))
        {
            self.tap = Some(Tap {
                id: pointer,
                start: at,
                action_button: layout.on_action(at),
                utility: layout.utility(at),
            });
        }
    }

    fn moved(&mut self, pointer: Pointer, at: Vec2) {
        if let Some(stick) = &mut self.stick
            && stick.id == pointer
        {
            stick.position = at;
        }
    }

    fn release(
        &mut self,
        pointer: Pointer,
        at: Option<Vec2>,
        layout: HudLayout,
        controls: &mut Controls,
    ) {
        if self.stick.as_ref().is_some_and(|stick| stick.id == pointer) {
            self.stick = None;
        }
        if self.tap.is_some_and(|tap| tap.id == pointer) {
            let tap = self.tap.take().unwrap();
            if let Some(at) = at.filter(|p| p.distance(tap.start) < 22.0 * layout.scale) {
                if let Some(command) = tap.utility {
                    if layout.utility(at) == Some(command) {
                        command.apply(controls);
                    }
                } else if tap.action_button {
                    controls.act |= layout.on_action(at);
                } else if !layout.on_stick(at) && !layout.on_action(at) && !layout.on_other_ui(at) {
                    controls.pick_at = Some(tap.start);
                    controls.finger_pick = matches!(pointer, Pointer::Finger(id) if id != u64::MAX);
                }
            }
        }
    }
}

pub(crate) fn stick_curve(axis: Vec2, dead_zone: f32) -> Vec2 {
    if !axis.is_finite() {
        return Vec2::ZERO;
    }
    let length = axis.length();
    let amount = ((length - dead_zone) / (1. - dead_zone)).clamp(0., 1.);
    axis.normalize_or_zero() * (amount * (0.73 + 0.27 * amount))
}

pub fn collect(
    mut touch: EventReader<TouchInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut wheel: EventReader<MouseWheel>,
    mut lifecycle: EventReader<AppLifecycle>,
    mut focus: EventReader<WindowFocused>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    layout: Res<HudLayout>,
    ready: Res<Ready>,
    session: Res<Session>,
    dialog: Res<Dialog>,
    overlays: (Res<crate::farm_ui::FarmMenu>, Res<crate::wheel::Wheel>, Res<crate::night::Night>, Res<crate::decorate::Layout>),
    controller: Res<crate::controller::Controller>,
    mut was_live: Local<bool>,
    mut gesture: ResMut<Gesture>,
    mut controls: ResMut<Controls>,
) {
    *controls = Controls::default();
    let (window_entity, window) = *window;
    let lost_focus = focus
        .read()
        .any(|event| event.window == window_entity && !event.focused)
        || lifecycle
            .read()
            .any(|event| matches!(event, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
    let (farm_menu, tool_wheel, night, decor) = overlays;
    let live = session.playing() && !dialog.open() && !farm_menu.open()
        && !tool_wheel.open() && !night.active() && !decor.open() && ready.0;
    let just_opened = live && !*was_live;
    *was_live = live;
    if lost_focus || !live || just_opened {
        *gesture = Gesture::default();
        touch.clear();
        mouse.clear();
        wheel.clear();
        return;
    }
    for event in touch.read().filter(|event| event.window == window_entity) {
        let pointer = Pointer::Finger(event.id);
        match event.phase {
            TouchPhase::Started => {
                gesture.press(pointer, event.position, *layout);
                if !layout.on_stick(event.position)
                    && !layout.on_other_ui(event.position)
                    && !layout.on_action(event.position)
                {
                    gesture.fingers.push((event.id, event.position));
                    if gesture.fingers.len() >= 2 {
                        gesture.tap = None;
                    }
                }
            }
            TouchPhase::Moved => {
                gesture.moved(pointer, event.position);
                let span = |points: &[(u64, Vec2)]| {
                    points.get(1).map(|p| p.1.distance(points[0].1).max(16.))
                };
                let previous = span(&gesture.fingers);
                if let Some(finger) = gesture.fingers.iter_mut().find(|p| p.0 == event.id) {
                    finger.1 = event.position;
                }
                if let (Some(before), Some(after)) = (previous, span(&gesture.fingers)) {
                    controls.zoom += (after / before).ln();
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                gesture.fingers.retain(|p| p.0 != event.id);
                gesture.release(
                    pointer,
                    (event.phase == TouchPhase::Ended).then_some(event.position),
                    *layout,
                    &mut controls,
                );
            }
        }
    }
    controls.zoom += wheel
        .read()
        .filter(|e| e.window == window_entity)
        .map(|e| match e.unit {
            bevy::input::mouse::MouseScrollUnit::Line => e.y * 0.12,
            bevy::input::mouse::MouseScrollUnit::Pixel => e.y * 0.006,
        })
        .sum::<f32>();
    for event in mouse
        .read()
        .filter(|event| event.button == MouseButton::Left && event.window == window_entity)
    {
        match event.state {
            ButtonState::Pressed => {
                if let Some(at) = window.cursor_position() {
                    gesture.press(Pointer::Mouse, at, *layout);
                }
            }
            ButtonState::Released => gesture.release(
                Pointer::Mouse,
                window.cursor_position(),
                *layout,
                &mut controls,
            ),
        }
    }
    if let Some(at) = window.cursor_position() {
        gesture.moved(Pointer::Mouse, at);
    }
    let axis = gesture.stick.as_ref().map_or(Vec2::ZERO, |stick| {
        let direction = (stick.position - stick.anchor) / (41. * layout.scale);
        stick_curve(direction * Vec2::new(1., -1.), 0.12)
    });
    gesture.stick_axis = axis;
    let right = keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let left = keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let forward = keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let back = keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let keyboard = Vec2::new(
        i32::from(right) as f32 - i32::from(left) as f32,
        i32::from(forward) as f32 - i32::from(back) as f32,
    );
    controls.move_axis = if axis != Vec2::ZERO {
        axis
    } else if controller.walk != Vec2::ZERO {
        controller.walk
    } else {
        keyboard.clamp_length_max(1.)
    };
    controls.act |= controller.act;
    controls.pad_action = controller.act || controller.fertilizer;
    controls.pad_cycle = controller.cycle;
    controls.cancel = controller.back;
    controls.task |= controller.task;
    controls.inventory |= controller.inventory;
    controls.choice |= controller.seeds;
    controls.fertilize |= controller.fertilizer;
    controls.wheel |= controller.wheel;
    controls.map |= controller.map;
    controls.zoom += controller.zoom;
    controls.cycle = keys.just_pressed(KeyCode::Tab);
    for (key, action) in [
        (KeyCode::Equal, Utility::In),
        (KeyCode::Minus, Utility::Out),
        (KeyCode::KeyM, Utility::Map),
        (KeyCode::KeyE, Utility::Choice),
        (KeyCode::KeyQ, Utility::Task),
        (KeyCode::KeyI, Utility::Inventory),
        (KeyCode::KeyF, Utility::Fertilize),
        (KeyCode::KeyR, Utility::FieldTool),
    ] {
        if keys.just_pressed(key) {
            action.apply(&mut controls);
        }
    }
    controls.act |= keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
}
