// Node.js StringDecoder 模块实现
// 字符流解码支持

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_string_decoder_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    let js_code = r#"
    (function() {
        class StringDecoder {
            constructor(encoding = 'utf8') {
                this.encoding = (encoding || 'utf8').toLowerCase();
                if (this.encoding === 'utf-8') this.encoding = 'utf8';
                this._buffer = [];
                this._decoder = new TextDecoder(this.encoding);
            }

            write(buffer) {
                if (typeof buffer === 'string') return buffer;
                let bytes = [];
                if (buffer instanceof Uint8Array) {
                    bytes = Array.from(buffer);
                } else if (buffer && buffer.buffer instanceof ArrayBuffer) {
                    const u8 = new Uint8Array(buffer.buffer, buffer.byteOffset || 0, buffer.byteLength || buffer.length || 0);
                    bytes = Array.from(u8);
                } else if (Array.isArray(buffer)) {
                    bytes = buffer.slice();
                } else {
                    return '';
                }

                const allBytes = this._buffer.concat(bytes);
                this._buffer = [];

                if (allBytes.length === 0) return '';

                // For UTF-8, detect trailing incomplete multibyte sequence
                if (this.encoding === 'utf8') {
                    let incompleteBytes = 0;
                    for (let check = 1; check <= 4 && check <= allBytes.length; check++) {
                        const b = allBytes[allBytes.length - check];
                        if ((b & 0x80) === 0) {
                            break;
                        } else if ((b & 0xC0) === 0xC0) {
                            let needed = 0;
                            if ((b & 0xE0) === 0xC0) needed = 2;
                            else if ((b & 0xF0) === 0xE0) needed = 3;
                            else if ((b & 0xF8) === 0xF0) needed = 4;

                            if (check < needed) {
                                incompleteBytes = check;
                            }
                            break;
                        }
                    }

                    if (incompleteBytes > 0) {
                        this._buffer = allBytes.slice(allBytes.length - incompleteBytes);
                        const emitBytes = allBytes.slice(0, allBytes.length - incompleteBytes);
                        if (emitBytes.length === 0) return '';
                        return this._decoder.decode(new Uint8Array(emitBytes));
                    }
                }

                return this._decoder.decode(new Uint8Array(allBytes));
            }

            end(buffer) {
                let res = '';
                if (buffer) {
                    res += this.write(buffer);
                }
                if (this._buffer.length > 0) {
                    res += this._decoder.decode(new Uint8Array(this._buffer));
                    this._buffer = [];
                }
                return res;
            }
        }

        const stringDecoderModule = {
            StringDecoder,
            default: StringDecoder
        };

        globalThis.__string_decoder = stringDecoderModule;
        globalThis.string_decoder = stringDecoderModule;
    })();
    "#;

    let script_source = v8::String::new(scope, js_code).unwrap();
    if let Some(script) = v8::Script::compile(scope, script_source, None) {
        let _ = script.run(scope);
    }

    let string_decoder_key = v8::String::new(scope, "__string_decoder").unwrap();
    if let Some(sd_val) = global.get(scope, string_decoder_key.into()) {
        let sd_key = v8::String::new(scope, "string_decoder").unwrap();
        global.set(scope, sd_key.into(), sd_val);
    }

    Ok(())
}
