//! Node.js `diagnostics_channel` module implementation.
use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_diagnostics_channel_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let script_code = r#"
    (function() {
        class Channel {
            constructor(name) {
                this.name = name;
                this._subscribers = [];
            }
            get hasSubscribers() {
                return this._subscribers.length > 0;
            }
            subscribe(subscription) {
                if (typeof subscription === 'function' && !this._subscribers.includes(subscription)) {
                    this._subscribers.push(subscription);
                }
            }
            unsubscribe(subscription) {
                const idx = this._subscribers.indexOf(subscription);
                if (idx !== -1) {
                    this._subscribers.splice(idx, 1);
                }
            }
            publish(message) {
                for (let i = 0; i < this._subscribers.length; i++) {
                    try {
                        this._subscribers[i](message, this.name);
                    } catch (e) {
                        if (typeof process !== 'undefined' && typeof process.nextTick === 'function') {
                            process.nextTick(() => { throw e; });
                        } else {
                            setTimeout(() => { throw e; }, 0);
                        }
                    }
                }
            }
        }

        const _channels = new Map();
        function channel(name) {
            if (typeof name !== 'string' && typeof name !== 'symbol') {
                throw new TypeError('Channel name must be a string or symbol');
            }
            let ch = _channels.get(name);
            if (!ch) {
                ch = new Channel(name);
                _channels.set(name, ch);
            }
            return ch;
        }

        function hasSubscribers(name) {
            const ch = _channels.get(name);
            return ch ? ch.hasSubscribers : false;
        }

        function subscribe(name, subscription) {
            return channel(name).subscribe(subscription);
        }

        function unsubscribe(name, subscription) {
            return channel(name).unsubscribe(subscription);
        }

        class TracingChannel {
            constructor(nameOrChannels) {
                if (typeof nameOrChannels === 'string') {
                    this.start = channel(`${nameOrChannels}:start`);
                    this.end = channel(`${nameOrChannels}:end`);
                    this.asyncStart = channel(`${nameOrChannels}:asyncStart`);
                    this.asyncEnd = channel(`${nameOrChannels}:asyncEnd`);
                    this.error = channel(`${nameOrChannels}:error`);
                } else if (typeof nameOrChannels === 'object' && nameOrChannels !== null) {
                    this.start = nameOrChannels.start || channel('tracing:start');
                    this.end = nameOrChannels.end || channel('tracing:end');
                    this.asyncStart = nameOrChannels.asyncStart || channel('tracing:asyncStart');
                    this.asyncEnd = nameOrChannels.asyncEnd || channel('tracing:asyncEnd');
                    this.error = nameOrChannels.error || channel('tracing:error');
                }
            }
            get hasSubscribers() {
                return !!(
                    (this.start && this.start.hasSubscribers) ||
                    (this.end && this.end.hasSubscribers) ||
                    (this.asyncStart && this.asyncStart.hasSubscribers) ||
                    (this.asyncEnd && this.asyncEnd.hasSubscribers) ||
                    (this.error && this.error.hasSubscribers)
                );
            }
            subscribe(subscribers) {
                if (!subscribers) return;
                if (subscribers.start && this.start) this.start.subscribe(subscribers.start);
                if (subscribers.end && this.end) this.end.subscribe(subscribers.end);
                if (subscribers.asyncStart && this.asyncStart) this.asyncStart.subscribe(subscribers.asyncStart);
                if (subscribers.asyncEnd && this.asyncEnd) this.asyncEnd.subscribe(subscribers.asyncEnd);
                if (subscribers.error && this.error) this.error.subscribe(subscribers.error);
            }
            unsubscribe(subscribers) {
                if (!subscribers) return;
                if (subscribers.start && this.start) this.start.unsubscribe(subscribers.start);
                if (subscribers.end && this.end) this.end.unsubscribe(subscribers.end);
                if (subscribers.asyncStart && this.asyncStart) this.asyncStart.unsubscribe(subscribers.asyncStart);
                if (subscribers.asyncEnd && this.asyncEnd) this.asyncEnd.unsubscribe(subscribers.asyncEnd);
                if (subscribers.error && this.error) this.error.unsubscribe(subscribers.error);
            }
            traceSync(fn, context = {}, thisArg, ...args) {
                if (this.start) this.start.publish(context);
                try {
                    const result = fn.apply(thisArg, args);
                    context.result = result;
                    return result;
                } catch (err) {
                    context.error = err;
                    if (this.error) this.error.publish(context);
                    throw err;
                } finally {
                    if (this.end) this.end.publish(context);
                }
            }
            tracePromise(fn, context = {}, thisArg, ...args) {
                if (this.start) this.start.publish(context);
                try {
                    const p = fn.apply(thisArg, args);
                    return Promise.resolve(p).then(
                        (result) => {
                            context.result = result;
                            if (this.end) this.end.publish(context);
                            return result;
                        },
                        (err) => {
                            context.error = err;
                            if (this.error) this.error.publish(context);
                            if (this.end) this.end.publish(context);
                            throw err;
                        }
                    );
                } catch (err) {
                    context.error = err;
                    if (this.error) this.error.publish(context);
                    if (this.end) this.end.publish(context);
                    throw err;
                }
            }
            traceCallback(fn, position = 0, context = {}, thisArg, ...args) {
                if (this.start) this.start.publish(context);
                const origCb = args[position];
                const wrappedCb = (...cbArgs) => {
                    if (cbArgs[0]) {
                        context.error = cbArgs[0];
                        if (this.error) this.error.publish(context);
                    } else {
                        context.result = cbArgs[1];
                    }
                    if (this.end) this.end.publish(context);
                    if (typeof origCb === 'function') {
                        return origCb.apply(this, cbArgs);
                    }
                };
                args[position] = wrappedCb;
                try {
                    return fn.apply(thisArg, args);
                } catch (err) {
                    context.error = err;
                    if (this.error) this.error.publish(context);
                    if (this.end) this.end.publish(context);
                    throw err;
                }
            }
        }

        function tracingChannel(nameOrChannels) {
            return new TracingChannel(nameOrChannels);
        }

        const dc = {
            Channel,
            channel,
            hasSubscribers,
            subscribe,
            unsubscribe,
            tracingChannel,
            TracingChannel,
        };
        dc.default = dc;
        return dc;
    })();
    "#;

    let source = v8::String::new(scope, script_code)
        .ok_or_else(|| anyhow::anyhow!("Failed to create diagnostics_channel bootstrap source"))?;
    let script = v8::Script::compile(scope, source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to compile diagnostics_channel bootstrap"))?;
    let dc_obj = script
        .run(scope)
        .ok_or_else(|| anyhow::anyhow!("Failed to run diagnostics_channel bootstrap"))?;

    let global = context.global(scope);
    let key = v8::String::new(scope, "diagnostics_channel").unwrap();
    global.set(scope, key.into(), dc_obj);

    Ok(())
}
