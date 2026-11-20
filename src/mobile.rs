use bevy::{prelude::*, window::ANDROID_APP};
use jni::{
    Env, JValue, JavaVM,
    errors::{Error, Result},
    jni_sig, jni_str,
    objects::{JByteArray, JIntArray, JObject},
    refs::Global,
};

fn activity<T>(
    call: impl for<'local, 'activity> FnOnce(&mut Env<'local>, &JObject<'activity>) -> Result<T>,
) -> Result<T> {
    let app = ANDROID_APP
        .get()
        .ok_or(Error::NullPtr("Android Activity"))?;
    // AndroidApp owns the VM and the activity reference for this invocation.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let raw = app.activity_as_ptr().cast();
    vm.attach_current_thread(|env| {
        env.with_local_frame(8, |env| {
            let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&raw)? };
            call(env, activity.as_ref().as_ref())
        })
    })
}

pub fn open_document(slot: usize, data: Option<&[u8]>) -> Result<()> {
    activity(|env, activity| {
        let bytes = env.byte_array_from_slice(data.unwrap_or_default())?;
        env.call_method(
            activity,
            jni_str!("openSaveDocument"),
            jni_sig!("(IZ[B)V"),
            &[
                JValue::Int(slot as i32 + 1),
                data.is_some().into(),
                (&bytes).into(),
            ],
        )?;
        Ok(())
    })
}

pub fn document_result() -> Result<Option<Vec<u8>>> {
    activity(|env, activity| {
        let result = env
            .call_method(
                activity,
                jni_str!("takeSaveDocumentResult"),
                jni_sig!("()[B"),
                &[],
            )?
            .l()?;
        if result.is_null() {
            return Ok(None);
        }
        let bytes = env.cast_local::<JByteArray>(result)?;
        env.convert_byte_array(&bytes).map(Some)
    })
}

pub fn input_removed() -> bool {
    activity(|env, activity| {
        env.call_method(
            activity,
            jni_str!("takeInputRemoved"),
            jni_sig!(() -> boolean),
            &[],
        )?
        .z()
    })
    .unwrap_or(false)
}

pub fn cutout_insets() -> Option<[f32; 4]> {
    activity(|env, activity| {
        let result = env
            .call_method(
                activity,
                jni_str!("displayCutoutInsets"),
                jni_sig!("()[I"),
                &[],
            )?
            .l()?;
        let result = env.cast_local::<JIntArray>(result)?;
        let mut values = [0; 4];
        result.get_region(env, 0, &mut values)?;
        Ok(values.map(|value| value.max(0) as f32))
    })
    .ok()
}

pub fn refresh_insets(
    time: Res<Time>,
    mut again: Local<f32>,
    mut insets: ResMut<crate::ui::SafeInsets>,
) {
    if time.elapsed_secs() < *again {
        return;
    }
    *again = time.elapsed_secs() + 0.5;
    if let Some(next) = cutout_insets()
        && insets.0 != next
    {
        insets.0 = next;
    }
}
