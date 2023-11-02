mod lexer;
use jni::{ JNIEnv, objects::{ JClass, JString, JObject, JByteArray }, sys::jobject };

#[no_mangle]
pub extern "system" fn compile<'l>(mut env: JNIEnv<'l>, class: JClass<'l>, code: JString<'l>) -> jobject {
  let Ok(bytes) = env.new_byte_array(3) else {
    return JObject::null().into_raw()
  };
  let Ok(_) = env.set_byte_array_region(&bytes, 0, &[1, 2, 3]) else {
    return JObject::null().into_raw()
  };
  bytes.into_raw()
}
