pub mod jni;
pub mod logger;

pub use jni::Java_com_playground_MainActivity_nativeInit;
pub use logger::android_log_init;