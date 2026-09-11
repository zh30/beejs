#include "napi.h"

static napi_value Hello(napi_env env, napi_callback_info info) {
  (void)info;
  napi_value result;
  napi_create_string_utf8(env, "world", 5, &result);
  return result;
}

napi_value napi_register_module_v1(napi_env env, napi_value exports) {
  napi_value fn;
  napi_create_function(env, "hello", 5, Hello, 0, &fn);
  napi_set_named_property(env, exports, "hello", fn);
  return exports;
}
