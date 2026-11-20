use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Session,
    Farm,
    Dialogue,
    Duel,
    Cinematic,
    #[cfg(target_os = "android")]
    Wheel,
    #[cfg(target_os = "android")]
    Rest,
    #[cfg(target_os = "android")]
    Decor,
}

#[derive(Component)]
pub struct FocusEnabled(pub bool);

#[derive(Resource, Default)]
pub struct Controller {
    pub walk: Vec2,
    pub zoom: f32,
    pub map: bool,
    pub act: bool,
    pub back: bool,
    pub cycle: i8,
    pub task: bool,
    pub inventory: bool,
    pub seeds: bool,
    pub fertilizer: bool,
    pub wheel: bool,
    pub accept: bool,
    pub radial: Vec2,
    pub rotate: bool,
    pub store: bool,
    pub step: Vec2,
    pub active: bool,
    pub end_turn: bool,
    pub page_step: i8,
    pub provisions: bool,
    #[cfg(target_os = "android")]
    pointer: Option<Vec2>,
    #[cfg(target_os = "android")]
    focused: Option<Entity>,
    #[cfg(target_os = "android")]
    release: Option<Vec2>,
}

#[cfg(any(target_os = "android", test))]
fn focus_step(
    buttons: impl Iterator<Item = (Entity, Vec2, bool)>,
    from: Option<Vec2>,
    step: Vec2,
) -> Option<Entity> {
    let axis = step.normalize_or_zero();
    buttons
        .filter_map(|(entity, point, close)| {
            let Some(from) = from else {
                let score = point.y * 4. + point.x * 0.01 + if close { 10_000. } else { 0. };
                return Some((entity, score));
            };
            let offset = point - from;
            let along = axis.dot(offset);
            let sideways = axis.perp_dot(offset).abs();
            (along > 7.).then_some((entity, along + sideways * 2.5))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(entity, _)| entity)
}

#[cfg(target_os = "android")]
mod android {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

    use bevy::{
        input::{
            ButtonState,
            keyboard::{Key, KeyboardInput},
            touch::{TouchInput, TouchPhase},
        },
        prelude::*,
        ui::FocusPolicy,
        window::{AppLifecycle, PrimaryWindow, WindowFocused},
    };
    use ndk_sys as ndk;

    use super::{Controller, FocusEnabled, Panel, focus_step};
    use crate::{
        Adventure,
        farm_ui::{Choice, FarmMenu, FarmPage},
        night::Night,
        session::Session,
        town::{Dialog, DialogueButton},
        wheel::Wheel,
    };

    const KEYS: [u32; 15] = [
        ndk::AKEYCODE_DPAD_UP,
        ndk::AKEYCODE_DPAD_DOWN,
        ndk::AKEYCODE_DPAD_LEFT,
        ndk::AKEYCODE_DPAD_RIGHT,
        ndk::AKEYCODE_BUTTON_A,
        ndk::AKEYCODE_BUTTON_B,
        ndk::AKEYCODE_BUTTON_X,
        ndk::AKEYCODE_BUTTON_Y,
        ndk::AKEYCODE_BUTTON_L1,
        ndk::AKEYCODE_BUTTON_R1,
        ndk::AKEYCODE_BUTTON_START,
        ndk::AKEYCODE_BUTTON_SELECT,
        ndk::AKEYCODE_BUTTON_L2,
        ndk::AKEYCODE_BUTTON_R2,
        ndk::AKEYCODE_BUTTON_THUMBR,
    ];
    const POINTER_ID: u64 = u64::MAX;
    static AXES: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
    static HELD: AtomicU32 = AtomicU32::new(0);
    static PRESSED: AtomicU32 = AtomicU32::new(0);
    unsafe extern "C" {
        fn __real_AInputQueue_getEvent(
            queue: *mut ndk::AInputQueue,
            output: *mut *mut ndk::AInputEvent,
        ) -> i32;
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn __wrap_AInputQueue_getEvent(
        queue: *mut ndk::AInputQueue,
        output: *mut *mut ndk::AInputEvent,
    ) -> i32 {
        loop {
            // Winit provides the live queue and its output slot. Finish each event we consume.
            let status = unsafe { __real_AInputQueue_getEvent(queue, output) };
            if status < 0 {
                return status;
            }
            let event = unsafe { *output };
            if !unsafe { consume_controller_event(event) } {
                return status;
            }
            unsafe { ndk::AInputQueue_finishEvent(queue, event, 1) };
        }
    }

    unsafe fn consume_controller_event(event: *mut ndk::AInputEvent) -> bool {
        let source = unsafe { ndk::AInputEvent_getSource(event) } as u32;
        let includes = |kind| source & kind == kind;
        let kind = unsafe { ndk::AInputEvent_getType(event) } as u32;
        match kind {
            ndk::AINPUT_EVENT_TYPE_KEY
                if includes(ndk::AINPUT_SOURCE_GAMEPAD) || includes(ndk::AINPUT_SOURCE_DPAD) =>
            {
                let code = unsafe { ndk::AKeyEvent_getKeyCode(event) } as u32;
                let Some(index) = KEYS.iter().position(|&key| key == code) else {
                    return false;
                };
                let bit = 1_u32 << index;
                match unsafe { ndk::AKeyEvent_getAction(event) } as u32 {
                    ndk::AKEY_EVENT_ACTION_DOWN => {
                        if HELD.fetch_or(bit, Relaxed) & bit == 0 {
                            PRESSED.fetch_or(bit, Relaxed);
                        }
                    }
                    ndk::AKEY_EVENT_ACTION_UP => {
                        HELD.fetch_and(!bit, Relaxed);
                    }
                    _ => {}
                }
                true
            }
            ndk::AINPUT_EVENT_TYPE_MOTION if includes(ndk::AINPUT_SOURCE_JOYSTICK) => {
                let action = unsafe { ndk::AMotionEvent_getAction(event) } as u32
                    & ndk::AMOTION_EVENT_ACTION_MASK;
                if action == ndk::AMOTION_EVENT_ACTION_MOVE
                    && unsafe { ndk::AMotionEvent_getPointerCount(event) } > 0
                {
                    let read = |axis: u32| unsafe {
                        ndk::AMotionEvent_getAxisValue(event, axis as i32, 0)
                    };
                    let axes = [
                        read(ndk::AMOTION_EVENT_AXIS_X),
                        read(ndk::AMOTION_EVENT_AXIS_Y),
                        read(ndk::AMOTION_EVENT_AXIS_Z),
                        read(ndk::AMOTION_EVENT_AXIS_RZ),
                        read(ndk::AMOTION_EVENT_AXIS_HAT_X),
                        read(ndk::AMOTION_EVENT_AXIS_HAT_Y),
                        read(ndk::AMOTION_EVENT_AXIS_LTRIGGER)
                            .max(read(ndk::AMOTION_EVENT_AXIS_BRAKE)),
                        read(ndk::AMOTION_EVENT_AXIS_RTRIGGER)
                            .max(read(ndk::AMOTION_EVENT_AXIS_GAS)),
                    ];
                    for (i, (slot, value)) in AXES.iter().zip(axes).enumerate() {
                        let previous = f32::from_bits(slot.swap(value.to_bits(), Relaxed));
                        // Keep taps that begin and end between two render frames.
                        let new_direction = value.abs() > 0.5 && (value - previous).abs() > 0.5;
                        let key = match i {
                            4 if new_direction => Some(if value < 0. { 2 } else { 3 }),
                            5 if new_direction => Some(if value < 0. { 0 } else { 1 }),
                            6 | 7 if value > 0.55 && previous <= 0.55 => Some(i + 6),
                            _ => None,
                        };
                        if let Some(key) = key {
                            PRESSED.fetch_or(1 << key, Relaxed);
                        }
                    }
                } else if action == ndk::AMOTION_EVENT_ACTION_CANCEL {
                    reset_input();
                }
                true
            }
            _ => false,
        }
    }

    fn reset_input() {
        for axis in &AXES {
            axis.store(0, Relaxed);
        }
        HELD.store(0, Relaxed);
        PRESSED.store(0, Relaxed);
    }

    #[derive(Component)]
    pub struct ControllerCursor;

    pub fn setup(mut commands: Commands) {
        commands.spawn((
            ControllerCursor,
            FocusPolicy::Pass,
            GlobalZIndex(150),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Px(19.),
                height: Val::Px(19.),
                border: UiRect::all(Val::Px(3.)),
                ..default()
            },
            BorderRadius::all(Val::Px(12.)),
            BorderColor(Color::srgb_u8(240, 168, 58)),
            BackgroundColor(Color::NONE),
        ));
    }

    fn dead_zone(axis: Vec2) -> Vec2 {
        if !axis.is_finite() {
            return Vec2::ZERO;
        }
        let speed = ((axis.length() - 0.19) / 0.81).clamp(0., 1.);
        axis.normalize_or_zero() * speed
    }

    #[derive(Default)]
    pub struct LastInput {
        release_escape: bool,
        panel: Option<(Panel, Option<FarmPage>)>,
        direction: Vec2,
        repeat_at: f32,
        triggers: (bool, bool),
        check_device_at: f32,
    }

    #[derive(bevy::ecs::system::SystemParam)]
    pub struct FocusButtons<'w, 's> {
        buttons: Query<
            'w,
            's,
            (
                Entity,
                &'static ComputedNode,
                &'static GlobalTransform,
                &'static InheritedVisibility,
                Option<&'static FocusEnabled>,
                Option<&'static Choice>,
                Option<&'static DialogueButton>,
            ),
            With<Button>,
        >,
        parents: Query<'w, 's, &'static ChildOf>,
        panels: Query<'w, 's, &'static Panel>,
    }

    pub fn poll(
        time: Res<Time>,
        mut controller: ResMut<Controller>,
        window: Single<(Entity, &Window), With<PrimaryWindow>>,
        mut focus: EventReader<WindowFocused>,
        mut lifecycle: EventReader<AppLifecycle>,
        mut touch: EventWriter<TouchInput>,
        mut keyboard: EventWriter<KeyboardInput>,
        mut cursor: Single<&mut Node, With<ControllerCursor>>,
        ui_scale: Res<UiScale>,
        layers: (Res<Session>, Res<FarmMenu>, Res<Dialog>, Res<Wheel>, Res<Night>, Res<Adventure>, Res<crate::decorate::Layout>),
        targets: FocusButtons,
        mut last: Local<LastInput>,
    ) {
        let (session, menu, dialog, wheel, night, adventure, decor) = layers;
        let (entity, window) = *window;
        let mut send_touch = |position, phase| {
            touch.write(TouchInput {
                phase,
                position,
                window: entity,
                force: None,
                id: POINTER_ID,
            });
        };
        let unfocused = focus.read().any(|event| !event.focused);
        let suspended = lifecycle
            .read()
            .any(|event| matches!(event, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
        if last.release_escape {
            keyboard.write(escape(entity, ButtonState::Released));
            last.release_escape = false;
        }
        let removed = if time.elapsed_secs() >= last.check_device_at {
            last.check_device_at = time.elapsed_secs() + 0.25;
            crate::mobile::input_removed()
        } else {
            false
        };
        if unfocused || suspended || removed {
            if let Some(position) = controller.release.take() {
                send_touch(position, TouchPhase::Canceled);
            }
            *controller = Controller::default();
            *last = LastInput::default();
            reset_input();
            cursor.display = Display::None;
            return;
        }
        let keys = PRESSED.swap(0, Relaxed);
        let held = HELD.load(Relaxed) | keys;
        let hit = |i: u32| keys & (1_u32 << i) != 0;
        let down = |i: u32| f32::from(u8::from(held & (1_u32 << i) != 0));
        let [x, y, look_x, look_y, hat_x, hat_y, lt, rt] =
            std::array::from_fn(|i| f32::from_bits(AXES[i].load(Relaxed)));
        let walk = crate::input::stick_curve(Vec2::new(x, -y), 0.19);
        let look = dead_zone(Vec2::new(look_x, look_y));
        let pad = (Vec2::new(down(3) - down(2), down(1) - down(0)) + Vec2::new(hat_x, hat_y))
            .clamp(Vec2::splat(-1.), Vec2::ONE);
        let triggers = (lt > 0.55 || hit(12), rt > 0.55 || hit(13));
        controller.active |= keys != 0
            || walk != Vec2::ZERO
            || look != Vec2::ZERO
            || pad != Vec2::ZERO
            || triggers.0
            || triggers.1;
        let layer = if !session.playing() {
            Some(Panel::Session)
        } else if night.active() {
            Some(Panel::Rest)
        } else if wheel.open() {
            Some(Panel::Wheel)
        } else if decor.open() {
            Some(Panel::Decor)
        } else if menu.clip().is_some() {
            Some(Panel::Cinematic)
        } else if menu.battle().is_some() {
            Some(Panel::Duel)
        } else if dialog.open() {
            Some(Panel::Dialogue)
        } else if menu.open() {
            Some(Panel::Farm)
        } else {
            None
        };
        let hand = layer == Some(Panel::Duel)
            && menu.battle().is_some_and(crate::duel::Battle::hand_controls);
        let direct = hand || matches!(layer, Some(Panel::Wheel | Panel::Rest | Panel::Decor));
        let panel = layer.map(|layer| (layer, menu.page()));
        let changed_panel = panel != last.panel;
        if changed_panel {
            controller.focused = None;
            controller.pointer = None;
            last.direction = Vec2::ZERO;
        }
        last.panel = panel;
        let nav = if layer.is_some() && layer != Some(Panel::Wheel) && pad == Vec2::ZERO {
            walk * Vec2::new(1., -1.)
        } else {
            pad
        };
        let direction = if nav.length() < 0.46 {
            Vec2::ZERO
        } else if nav.x.abs() > nav.y.abs() {
            Vec2::X * nav.x.signum()
        } else {
            Vec2::Y * nav.y.signum()
        };
        let now = time.elapsed_secs();
        let is_new_direction = direction != last.direction || keys & 0b1111 != 0;
        let step = if direction != Vec2::ZERO
            && (is_new_direction || (layer.is_some() && now >= last.repeat_at))
        {
            last.repeat_at = now + if is_new_direction { 0.30 } else { 0.115 };
            direction
        } else {
            Vec2::ZERO
        };
        last.direction = direction;
        let world = layer.is_none();
        let size = Vec2::new(window.width(), window.height());
        controller.walk = if world { walk } else { Vec2::ZERO };
        controller.zoom = if world {
            0.21 * (f32::from(u8::from(hit(9))) - f32::from(u8::from(hit(8))))
        } else {
            0.
        };
        let pointer = world && adventure.map && adventure.area.outdoors();
        if !pointer {
            controller.pointer = None;
        }
        let accept = hit(4) && !(hit(5) || hit(10));
        controller.act = world && accept
            && (!pointer || (controller.pointer.is_none() && look == Vec2::ZERO));
        controller.accept = direct && accept;
        controller.step = step;
        controller.radial = if walk.length_squared() >= 0.26 {
            walk * Vec2::new(1., -1.)
        } else {
            look
        };
        controller.wheel = (world || layer == Some(Panel::Wheel)) && (hit(6) || hit(14));
        controller.map = world && hit(7);
        controller.back = (world || matches!(layer, Some(Panel::Wheel | Panel::Decor))) && hit(5);
        controller.rotate = layer == Some(Panel::Decor) && (hit(6) || hit(9));
        controller.store = layer == Some(Panel::Decor) && hit(7);
        controller.cycle = if world { step.x as i8 } else { 0 };
        controller.task = world && step.y < 0.;
        controller.inventory = world && (hit(11) || step.y > 0.);
        controller.seeds = world && triggers.0 && !last.triggers.0;
        controller.fertilizer = world && triggers.1 && !last.triggers.1;
        controller.end_turn = hand && hit(7);
        controller.provisions = layer == Some(Panel::Duel) && hit(6);
        controller.page_step = if layer == Some(Panel::Farm) {
            i8::from(hit(9)) - i8::from(hit(8))
        } else {
            0
        };
        last.triggers = triggers;
        if (hit(10) || hit(5) && !world) && !matches!(layer, Some(Panel::Wheel | Panel::Rest | Panel::Decor)) {
            keyboard.write(escape(entity, ButtonState::Pressed));
            last.release_escape = true;
        }
        if hit(10) && matches!(layer, Some(Panel::Wheel | Panel::Decor)) {
            controller.back = true;
        }
        if let Some(position) = controller.release.take() {
            send_touch(
                position,
                if changed_panel {
                    TouchPhase::Canceled
                } else {
                    TouchPhase::Ended
                },
            );
        }
        let selection = layer.filter(|_| controller.active && !direct).and_then(|layer| {
            let candidates: Vec<_> = targets
                .buttons
                .iter()
                .filter_map(|(entity, node, pose, visible, enabled, button, dialogue)| {
                    if !visible.get()
                        || enabled.is_some_and(|state| !state.0)
                        || node.size().min_element() < 4.
                        || !targets.parents.iter_ancestors(entity).any(|parent| {
                            targets
                                .panels
                                .get(parent)
                                .is_ok_and(|&owner| owner == layer)
                        })
                    {
                        return None;
                    }
                    let point = pose.translation().truncate() / window.scale_factor();
                    let bounds = node.size() / window.scale_factor();
                    let close = matches!(button, Some(Choice::Close))
                        || matches!(dialogue, Some(DialogueButton::Close));
                    (point.cmpge(Vec2::ZERO).all() && point.cmple(size).all())
                        .then_some((entity, point, bounds, close))
                })
                .collect();
            let current = candidates
                .iter()
                .find(|(id, _, _, _)| Some(*id) == controller.focused);
            if current.is_none() || step != Vec2::ZERO {
                controller.focused = focus_step(
                    candidates.iter().map(|&(id, p, _, close)| (id, p, close)),
                    current.map(|(_, p, _, _)| *p),
                    step,
                )
                .or(controller.focused.filter(|_| current.is_some()));
            }
            candidates
                .into_iter()
                .find(|(id, _, _, _)| Some(*id) == controller.focused)
                .map(|(_, point, bounds, _)| (point, bounds + Vec2::splat(8.)))
        });
        if look != Vec2::ZERO && pointer {
            let start = controller.pointer.unwrap_or(size / 2.);
            controller.pointer =
                Some((start + look * 610. * time.delta_secs().min(0.05)).clamp(Vec2::ZERO, size));
        }
        if world
            && (walk != Vec2::ZERO || controller.cycle != 0 || controller.back)
            && look == Vec2::ZERO
        {
            controller.pointer = None;
        }
        let mark = selection.or_else(|| {
            controller
                .pointer
                .filter(|_| world)
                .map(|p| (p, Vec2::splat(19.)))
        });
        if accept && !direct
            && let Some((position, _)) = mark
        {
            send_touch(position, TouchPhase::Started);
            controller.release = Some(position);
        }
        if let Some((position, bounds)) = mark {
            cursor.display = Display::Flex;
            let origin = (position - bounds / 2.) / ui_scale.0;
            cursor.left = Val::Px(origin.x);
            cursor.top = Val::Px(origin.y);
            cursor.width = Val::Px(bounds.x / ui_scale.0);
            cursor.height = Val::Px(bounds.y / ui_scale.0);
        } else {
            cursor.display = Display::None;
        }
    }

    pub fn hide_after_touch(
        mut touches: EventReader<TouchInput>,
        mut controller: ResMut<Controller>,
        mut cursor: Single<&mut Node, With<ControllerCursor>>,
    ) {
        if touches
            .read()
            .any(|event| event.id != POINTER_ID && event.phase == TouchPhase::Started)
        {
            controller.pointer = None;
            controller.focused = None;
            controller.active = false;
            cursor.display = Display::None;
        }
    }

    fn escape(window: Entity, state: ButtonState) -> KeyboardInput {
        KeyboardInput {
            key_code: KeyCode::Escape,
            logical_key: Key::Escape,
            state,
            text: None,
            repeat: false,
            window,
        }
    }
}

#[cfg(target_os = "android")]
pub use android::{hide_after_touch, poll, setup};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_focus_prefers_a_work_button_and_direction_stays_in_the_column() {
        let top = Entity::from_raw(1);
        let close = Entity::from_raw(2);
        let below = Entity::from_raw(3);
        let diagonal = Entity::from_raw(4);
        let buttons = [
            (close, Vec2::new(800., 40.), true),
            (top, Vec2::new(160., 180.), false),
            (below, Vec2::new(160., 280.), false),
            (diagonal, Vec2::new(270., 234.), false),
        ];
        assert_eq!(focus_step(buttons.into_iter(), None, Vec2::ZERO), Some(top));
        assert_eq!(
            focus_step(buttons.into_iter(), Some(Vec2::new(160., 180.)), Vec2::Y),
            Some(below)
        );
    }
}
