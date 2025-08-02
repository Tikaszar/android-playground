use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_playground_MainActivity_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jstring {
    let path: String = env
        .get_string(&path)
        .expect("Couldn't get java string")
        .into();
    
    tracing::info!("Native init called with path: {}", path);
    
    let output = env
        .new_string(format!("Initialized at: {}", path))
        .expect("Couldn't create java string");
    
    output.into_raw()
}

pub fn android_log_init() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace),
    );
}