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

#[derive(Resource, Default)]
pub struct Gesture {
    stick: Option<(Pointer, Vec2)>,
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
                self.stick = Some((pointer, at));
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
        if let Some((id, pos)) = &mut self.stick {
            if *id == pointer {
                *pos = at;
            }
        }
    }

    fn release(
        &mut self,
        pointer: Pointer,
        at: Option<Vec2>,
        layout: HudLayout,
        controls: &mut Controls,
    ) {
        if self.stick.is_some_and(|(id, _)| id == pointer) {
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
                    controls.pick_at = Some(at);
                }
            }
        }
    }
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
    let live = session.playing() && !dialog.open() && ready.0;
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
    let axis = gesture.stick.map_or(Vec2::ZERO, |(_, pos)| {
        let direction = (pos - layout.stick_center()) / (44.0 * layout.scale);
        let magnitude = ((direction.length() - 0.10) / 0.90).clamp(0., 1.);
        Vec2::new(direction.x, -direction.y).normalize_or_zero() * magnitude.powf(1.24)
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
    controls.move_axis = (axis + keyboard).clamp_length_max(1.0);
    controls.cycle = keys.just_pressed(KeyCode::Tab);
    for (key, action) in [
        (KeyCode::Equal, Utility::In),
        (KeyCode::Minus, Utility::Out),
        (KeyCode::KeyM, Utility::Map),
        (KeyCode::KeyE, Utility::Choice),
        (KeyCode::KeyQ, Utility::Task),
        (KeyCode::KeyI, Utility::Inventory),
    ] {
        if keys.just_pressed(key) {
            action.apply(&mut controls);
        }
    }
    controls.act |= keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
}
