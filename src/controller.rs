use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Controller {
    pub walk: Vec2,
    pub zoom: f32,
    pub map: bool,
    pub act: bool,
    #[cfg(target_os = "android")]
    pointer: Option<Vec2>,
    #[cfg(target_os = "android")]
    pressed_at: Option<Vec2>,
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

    use super::Controller;
    use crate::{farm_ui::FarmMenu, session::Session, town::Dialog};

    const KEYS: [u32; 11] = [
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
    ];
    const POINTER_ID: u64 = u64::MAX;
    static AXES: [AtomicU32; 6] = [const { AtomicU32::new(0) }; 6];
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
                    let alternate = |main, other| {
                        let value = read(main);
                        if value.abs() > 0.03 {
                            value
                        } else {
                            read(other)
                        }
                    };
                    let axes = [
                        read(ndk::AMOTION_EVENT_AXIS_X),
                        read(ndk::AMOTION_EVENT_AXIS_Y),
                        alternate(ndk::AMOTION_EVENT_AXIS_Z, ndk::AMOTION_EVENT_AXIS_RX),
                        alternate(ndk::AMOTION_EVENT_AXIS_RZ, ndk::AMOTION_EVENT_AXIS_RY),
                        read(ndk::AMOTION_EVENT_AXIS_HAT_X),
                        read(ndk::AMOTION_EVENT_AXIS_HAT_Y),
                    ];
                    for (slot, value) in AXES.iter().zip(axes) {
                        slot.store(value.to_bits(), Relaxed);
                    }
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
            BorderColor(Color::srgb_u8(255, 226, 128)),
            BackgroundColor(Color::srgba(0.11, 0.17, 0.12, 0.44)),
        ));
    }

    fn dead_zone(axis: Vec2) -> Vec2 {
        if !axis.is_finite() {
            return Vec2::ZERO;
        }
        let speed = ((axis.length() - 0.19) / 0.81).clamp(0., 1.);
        axis.normalize_or_zero() * speed
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
        session: Res<Session>,
        menu: Res<FarmMenu>,
        dialog: Res<Dialog>,
        buttons: Query<(&ComputedNode, &GlobalTransform, &InheritedVisibility), With<Button>>,
        mut closing: Local<bool>,
        mut previous_hat: Local<Vec2>,
        mut was_modal: Local<bool>,
    ) {
        let (entity, window) = *window;
        let mut gesture = |position, phase| {
            touch.write(TouchInput {
                phase,
                position,
                window: entity,
                force: None,
                id: POINTER_ID,
            });
        };
        let unfocused = focus.read().any(|event| !event.focused);
        let suspending = lifecycle
            .read()
            .any(|event| matches!(event, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
        if unfocused || suspending {
            if let Some(position) = controller.pressed_at {
                gesture(position, TouchPhase::Canceled);
            }
            *controller = Controller::default();
            reset_input();
        }
        if *closing {
            keyboard.write(escape(entity, ButtonState::Released));
            *closing = false;
        }
        let keys = PRESSED.swap(0, Relaxed);
        let held = HELD.load(Relaxed) | keys;
        let hit = |index| keys & (1_u32 << index) != 0_u32;
        let down = |index| f32::from(u8::from(held & (1_u32 << index) != 0_u32));
        let [x, y, look_x, look_y, hat_x, hat_y] =
            std::array::from_fn(|i| f32::from_bits(AXES[i].load(Relaxed)));
        controller.walk = dead_zone(Vec2::new(x, -y));
        controller.act = hit(6);
        controller.map = hit(7);
        controller.zoom = 0.21 * f32::from(i8::from(hit(9)) - i8::from(hit(8)));
        if hit(5) || hit(10) {
            keyboard.write(escape(entity, ButtonState::Pressed));
            *closing = true;
        }
        let size = Vec2::new(window.width(), window.height());
        let modal = !session.playing() || menu.open() || dialog.open();
        let entered_modal = modal && !*was_modal;
        *was_modal = modal;
        let hat = Vec2::new(hat_x, hat_y);
        let dpad = Vec2::new(down(3) - down(2), down(1) - down(0));
        let stick = Vec2::new(look_x, look_y);
        let direction = dead_zone(if modal { stick } else { stick + hat + dpad });
        let tap = |i| f32::from(u8::from(hit(i)));
        let step = Vec2::new(tap(3) - tap(2), tap(1) - tap(0))
            + if hat != *previous_hat {
                hat
            } else {
                Vec2::ZERO
            };
        *previous_hat = hat;
        if modal
            && (step != Vec2::ZERO
                || hit(4) && controller.pointer.is_none()
                || entered_modal && controller.pointer.is_some())
        {
            let visible = buttons.iter().filter_map(|(button, transform, visible)| {
                if !visible.get() || button.size().min_element() < 4. {
                    return None;
                }
                let at = transform.translation().truncate() / window.scale_factor();
                at.cmpge(Vec2::ZERO)
                    .all()
                    .then_some(at)
                    .filter(|p| p.cmple(size).all())
            });
            let from = if entered_modal {
                None
            } else {
                controller.pointer
            };
            if let Some(point) = focus_step(visible, from, step) {
                controller.pointer = Some(point);
            }
        }
        let movement = time.delta_secs().clamp(0.01, 0.04) * 610. * direction;
        if direction != Vec2::ZERO || hit(4) {
            let center = controller.pointer.unwrap_or(size / 2.);
            controller.pointer = Some((center + movement).clamp(Vec2::ZERO, size));
        }
        if let Some(position) = controller.pressed_at.take() {
            gesture(position, TouchPhase::Ended);
            if hit(4) {
                PRESSED.fetch_or(1 << 4, Relaxed);
            }
        } else if hit(4)
            && let Some(position) = controller.pointer
        {
            gesture(position, TouchPhase::Started);
            controller.pressed_at = Some(position);
        }
        if let Some(position) = controller.pointer {
            cursor.display = Display::Flex;
            cursor.left = Val::Px(position.x / ui_scale.0 - 9.5);
            cursor.top = Val::Px(position.y / ui_scale.0 - 9.5);
        } else {
            cursor.display = Display::None;
        }
    }

    fn focus_step(
        buttons: impl Iterator<Item = Vec2>,
        from: Option<Vec2>,
        step: Vec2,
    ) -> Option<Vec2> {
        let axis = step.normalize_or_zero();
        buttons
            .filter_map(|point| {
                let Some(from) = from else {
                    return Some((point, point.y * 4096. + point.x));
                };
                let offset = point - from;
                let along = axis.dot(offset);
                let perpendicular = axis.perp_dot(offset).abs();
                (along > 7.).then_some((point, along + perpendicular * 2.35))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(point, _)| point)
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
