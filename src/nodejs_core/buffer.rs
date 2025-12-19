//! Node.js Buffer模块实现
//! 高性能二进制数据处理

use anyhow::Result;
use rusty_v8 as v8;

/// 设置Buffer API
pub fn setup_buffer_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // 创建Buffer构造函数
    let buffer_constructor = v8::FunctionTemplate::new(scope, buffer_constructor_callback);

    // 添加静态方法
    // Buffer.from()
    let from_func = v8::FunctionTemplate::new(scope, buffer_from_callback);
    let from_instance = from_func.get_function(scope).unwrap();
    buffer_constructor.set_on_instance(scope, v8::String::new(scope, "from").unwrap().into(), from_instance.into());

    // Buffer.alloc()
    let alloc_func = v8::FunctionTemplate::new(scope, buffer_alloc_callback);
    let alloc_instance = alloc_func.get_function(scope).unwrap();
    buffer_constructor.set_on_instance(scope, v8::String::new(scope, "alloc").unwrap().into(), alloc_instance.into());

    // Buffer.concat()
    let concat_func = v8::FunctionTemplate::new(scope, buffer_concat_callback);
    let concat_instance = concat_func.get_function(scope).unwrap();
    buffer_constructor.set_on_instance(scope, v8::String::new(scope, "concat").unwrap().into(), concat_instance.into());

    // Buffer.byteLength()
    let byte_length_func = v8::FunctionTemplate::new(scope, buffer_byte_length_callback);
    let byte_length_instance = byte_length_func.get_function(scope).unwrap();
    buffer_constructor.set_on_instance(scope, v8::String::new(scope, "byteLength").unwrap().into(), byte_length_instance.into());

    // Buffer.isBuffer()
    let is_buffer_func = v8::FunctionTemplate::new(scope, buffer_is_buffer_callback);
    let is_buffer_instance = is_buffer_func.get_function(scope).unwrap();
    buffer_constructor.set_on_instance(scope, v8::String::new(scope, "isBuffer").unwrap().into(), is_buffer_instance.into());

    // 创建Buffer函数实例
    let buffer_func = buffer_constructor.get_function(scope).unwrap();

    // 添加实例方法
    // toString()
    let to_string_func = v8::FunctionTemplate::new(scope, buffer_to_string_callback);
    buffer_constructor.set_prototype_property_initializer_callback(
        scope,
        v8::String::new(scope, "toString").unwrap().into(),
        to_string_func,
    );

    // toJSON()
    let to_json_func = v8::FunctionTemplate::new(scope, buffer_to_json_callback);
    buffer_constructor.set_prototype_property_initializer_callback(
        scope,
        v8::String::new(scope, "toJSON").unwrap().into(),
        to_json_func,
    );

    // fill()
    let fill_func = v8::FunctionTemplate::new(scope, buffer_fill_callback);
    buffer_constructor.set_prototype_property_initializer_callback(
        scope,
        v8::String::new(scope, "fill").unwrap().into(),
        fill_func,
    );

    // slice()
    let slice_func = v8::FunctionTemplate::new(scope, buffer_slice_callback);
    buffer_constructor.set_prototype_property_initializer_callback(
        scope,
        v8::String::new(scope, "slice").unwrap().into(),
        slice_func,
    );

    // length 属性
    let length_getter = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let this = args.this();
        let length_key = v8::String::new(scope, "_length").unwrap();
        let length = this.get(scope, length_key.into()).unwrap_or(v8::Integer::new(scope, 0).into());
        _rv.set(length.into());
    });

    buffer_constructor.set_prototype_property_accessor(
        scope,
        v8::String::new(scope, "length").unwrap().into(),
        length_getter,
    );

    // 设置Buffer到全局
    let global = context.global(scope);
    let buffer_key = v8::String::new(scope, "Buffer").unwrap();
    global.set(scope, buffer_key.into(), buffer_func.into());

    Ok(())
}

fn buffer_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let size = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let buffer = v8::ArrayBuffer::new(scope, size);
    let buffer_view = unsafe {
        std::slice::from_raw_parts_mut(
            buffer.buffer().data() as *mut u8,
            size
        )
    };
    buffer_view.fill(0);

    // 设置length属性
    let length_key = v8::String::new(scope, "_length").unwrap();
    let length_key_val = v8::Integer::new(scope, size as i32).into();

    buffer.set(scope, length_key.into(), length_key_val);;

    retval.set(buffer.into());
}

fn buffer_from_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let arg = args.get(0);

    if arg.is_string() {
        // Buffer.from(string)
        let string = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let encoding = args
            .get(1)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| "utf8".to_string());

        let bytes = match encoding.as_str() {
            "utf8" | "utf-8" => string.as_bytes().to_vec(),
            "hex" => hex::decode(&string).unwrap_or_default(),
            "base64" => base64::decode(&string).unwrap_or_default(),
            "latin1" => string.chars().map(|c| c as u8).collect(),
            _ => string.as_bytes().to_vec(),
        };

        let buffer = v8::ArrayBuffer::new(scope, bytes.len());
        let backing_store = buffer.backing_store();
        unsafe {
            std::slice::from_raw_parts_mut(
                backing_store.data() as *mut u8,
                bytes.len()
            )
        }.copy_from_slice(&bytes);

        let length_key = v8::String::new(scope, "_length").unwrap();
        buffer.set(scope, length_key.into(), v8::Integer::new(scope, bytes.len() as i32).into());

        retval.set(buffer.into());
    } else if arg.is_array() {
        let arr = v8::Local::<v8::Array>::try_from(arg).unwrap();
        // Buffer.from(array)
        let length = arr.length() as usize;
        let mut bytes = vec![0u8; length];

        for i in 0..length {
            if let Some(val) = arr.get_index(scope, i) {
                if let Some(int) = val.to_integer(scope) {
                    bytes[i] = int.value() as u8;
                }
            }
        }

        let buffer = v8::ArrayBuffer::new(scope, length);
        unsafe {
            std::slice::from_raw_parts_mut(
                buffer.buffer().data() as *mut u8,
                length
            )
        }.copy_from_slice(&bytes);

        let length_key = v8::String::new(scope, "_length").unwrap();
        let length_key_val = v8::Integer::new(scope, length as i32).into();

        buffer.set(scope, length_key.into(), length_key_val);;

        retval.set(buffer.into());
    } else {
        // 默认返回空buffer
        let buffer = v8::ArrayBuffer::new(scope, 0);
        retval.set(buffer.into());
    }
}

fn buffer_alloc_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let size = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let fill_value = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as u8;

    let buffer = v8::ArrayBuffer::new(scope, size);
    unsafe {
        let backing_store = buffer.backing_store();
        std::slice::from_raw_parts_mut(
            backing_store.data() as *mut u8,
            size
        )
    }.fill(fill_value);

    let length_key = v8::String::new(scope, "_length").unwrap();
    let length_key_val = v8::Integer::new(scope, size as i32).into();

    buffer.set(scope, length_key.into(), length_key_val);;

    retval.set(buffer.into());
}

fn buffer_concat_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let list = args.get(0);
    let total_length = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    if list.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(list) {
            let mut combined_data = Vec::new();
            let mut calculated_length = 0;

            for i in 0..arr.length() {
                if let Ok(buf) = v8::Local::<v8::Array>::try_from(arr.get_index(scope, i).unwrap()) {
                    let length_key = v8::String::new(scope, "_length").unwrap();
                    if let Some(len_val) = buf.get(scope, length_key.into()).and_then(|v| v.to_integer(scope)) {
                        let len = len_val.value() as usize;
                        calculated_length += len;

                        // 复制数据
                        unsafe {
                            let backing_store = buf.backing_store();
                            let data_ptr = backing_store.data() as *const u8;
                            let data_slice = std::slice::from_raw_parts(data_ptr, len);
                            combined_data.extend_from_slice(data_slice);
                        }
                    }
                }
            }

            let target_length = if total_length > 0 { total_length } else { calculated_length };
            let buffer = v8::ArrayBuffer::new(scope, target_length);

            unsafe {
                let backing_store = buffer.backing_store();
                std::slice::from_raw_parts_mut(
                    backing_store.data() as *mut u8,
                    target_length
                )
            }.copy_from_slice(&combined_data[..target_length]);

            let length_key = v8::String::new(scope, "_length").unwrap();
            let length_key_val = v8::Integer::new(scope, target_length as i32).into();

            buffer.set(scope, length_key.into(), length_key_val);;

            retval.set(buffer.into());
        }
    } else {
        retval.set(v8::null(scope).into());
    }
}

fn buffer_byte_length_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let string = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let encoding = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "utf8".to_string());

    let byte_length = match encoding.as_str() {
        "utf8" | "utf-8" => string.as_bytes().len(),
        "hex" => string.len() / 2,
        "base64" => (string.len() * 3) / 4,
        "latin1" => string.len(),
        _ => string.as_bytes().len(),
    };

    retval.set(v8::Integer::new(scope, byte_length as i32).into());
}

fn buffer_is_buffer_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let value = args.get(0);
    let is_buffer = value.is_array_buffer();

    retval.set(v8::Boolean::new(scope, is_buffer).into());
}

fn buffer_to_string_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let encoding = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "utf8".to_string());

    let start = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let end = args
        .get(2)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, -1))
        .value() as usize;

    let length_key = v8::String::new(scope, "_length").unwrap();
    let buffer_length = this
        .get(scope, length_key.into())
        .and_then(|v| v.to_integer(scope).map(|i| i.value() as usize))
        .unwrap_or(0);

    let actual_end = if end == -1 { buffer_length } else { end.min(buffer_length) };
    let actual_start = start.min(buffer_length);

    if actual_start >= actual_end {
        retval.set(v8::String::new(scope, "").unwrap().into());
        return;
    }

    unsafe {
        let data_ptr = this.to_object(scope).unwrap().buffer().unwrap().data() as *const u8;
        let data_slice = std::slice::from_raw_parts(data_ptr, buffer_length);

        let result = match encoding.as_str() {
            "utf8" | "utf-8" => {
                String::from_utf8_lossy(&data_slice[actual_start..actual_end]).to_string()
            }
            "hex" => hex::encode(&data_slice[actual_start..actual_end]),
            "base64" => base64::encode(&data_slice[actual_start..actual_end]),
            "latin1" => {
                let bytes = &data_slice[actual_start..actual_end];
                bytes.iter().map(|&b| b as char).collect::<String>()
            }
            _ => String::from_utf8_lossy(&data_slice[actual_start..actual_end]).to_string(),
        };

        retval.set(v8::String::new(scope, &result).unwrap().into());
    }
}

fn buffer_to_json_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();

    let length_key = v8::String::new(scope, "_length").unwrap();
    let buffer_length = this
        .get(scope, length_key.into())
        .and_then(|v| v.to_integer(scope).map(|i| i.value() as usize))
        .unwrap_or(0);

    unsafe {
        let data_ptr = this.to_object(scope).unwrap().buffer().unwrap().data() as *const u8;
        let data_slice = std::slice::from_raw_parts(data_ptr, buffer_length);

        // 创建JSON数组
        let json_array = v8::Array::new(scope, buffer_length);
        for i in 0..buffer_length {
            json_array.set_index(scope, i, v8::Integer::new(scope, data_slice[i] as i32).into());
        }

        retval.set(json_array.into());
    }
}

fn buffer_fill_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let value = args.get(0);

    let start = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let end = args
        .get(2)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, -1))
        .value() as usize;

    let length_key = v8::String::new(scope, "_length").unwrap();
    let buffer_length = this
        .get(scope, length_key.into())
        .and_then(|v| v.to_integer(scope).map(|i| i.value() as usize))
        .unwrap_or(0);

    let actual_end = if end == -1 { buffer_length } else { end.min(buffer_length) };
    let actual_start = start.min(buffer_length);

    let fill_value = if value.is_number() {
        value.to_integer(scope).unwrap().value() as u8
    } else if value.is_string() {
        let string = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
        string.chars().next().unwrap_or('\0') as u8
    } else {
        0
    };

    unsafe {
        let data_ptr = this.to_object(scope).unwrap().buffer().unwrap().data() as *mut u8;
        let data_slice = std::slice::from_raw_parts_mut(data_ptr, buffer_length);
        for i in actual_start..actual_end {
            data_slice[i] = fill_value;
        }
    }

    retval.set(this.into());
}

fn buffer_slice_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let start = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let end = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, -1))
        .value() as usize;

    let length_key = v8::String::new(scope, "_length").unwrap();
    let buffer_length = this
        .get(scope, length_key.into())
        .and_then(|v| v.to_integer(scope).map(|i| i.value() as usize))
        .unwrap_or(0);

    let actual_end = if end == -1 { buffer_length } else { end.min(buffer_length) };
    let actual_start = start.min(buffer_length);
    let slice_length = if actual_end > actual_start { actual_end - actual_start } else { 0 };

    let new_buffer = v8::ArrayBuffer::new(scope, slice_length);

    if slice_length > 0 {
        unsafe {
            let old_data_ptr = this.to_object(scope).unwrap().buffer().unwrap().data() as *const u8;
            let old_data_slice = std::slice::from_raw_parts(old_data_ptr, buffer_length);

            let new_data_ptr = new_buffer.buffer().data() as *mut u8;
            let new_data_slice = std::slice::from_raw_parts_mut(new_data_ptr, slice_length);

            new_data_slice.copy_from_slice(&old_data_slice[actual_start..actual_end]);
        }
    }

    let length_key = v8::String::new(scope, "_length").unwrap();
    let length_key_val = v8::Integer::new(scope, slice_length as i32).into();

    new_buffer.set(scope, length_key.into(), length_key_val);;

    retval.set(new_buffer.into());
}
