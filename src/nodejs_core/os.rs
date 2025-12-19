//! Node.js OS模块实现
//! 操作系统信息

use anyhow::Result;
use rusty_v8 as v8;
use std::env;

/// 设置OS API
pub fn setup_os_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let os_obj = v8::Object::new(scope);

    // arch
    let arch_func = v8::FunctionTemplate::new(scope, os_arch_callback);
    let arch_instance = arch_func.get_function(scope).unwrap();
    let arch_key = v8::String::new(scope, "arch").unwrap();
    os_obj.set(scope, arch_key.into(), arch_instance.into());

    // platform
    let platform_func = v8::FunctionTemplate::new(scope, os_platform_callback);
    let platform_instance = platform_func.get_function(scope).unwrap();
    let platform_key = v8::String::new(scope, "platform").unwrap();
    os_obj.set(scope, platform_key.into(), platform_instance.into());

    // type
    let type_func = v8::FunctionTemplate::new(scope, os_type_callback);
    let type_instance = type_func.get_function(scope).unwrap();
    let type_key = v8::String::new(scope, "type").unwrap();
    os_obj.set(scope, type_key.into(), type_instance.into());

    // release
    let release_func = v8::FunctionTemplate::new(scope, os_release_callback);
    let release_instance = release_func.get_function(scope).unwrap();
    let release_key = v8::String::new(scope, "release").unwrap();
    os_obj.set(scope, release_key.into(), release_instance.into());

    // hostname
    let hostname_func = v8::FunctionTemplate::new(scope, os_hostname_callback);
    let hostname_instance = hostname_func.get_function(scope).unwrap();
    let hostname_key = v8::String::new(scope, "hostname").unwrap();
    os_obj.set(scope, hostname_key.into(), hostname_instance.into());

    // loadavg
    let loadavg_func = v8::FunctionTemplate::new(scope, os_loadavg_callback);
    let loadavg_instance = loadavg_func.get_function(scope).unwrap();
    let loadavg_key = v8::String::new(scope, "loadavg").unwrap();
    os_obj.set(scope, loadavg_key.into(), loadavg_instance.into());

    // uptime
    let uptime_func = v8::FunctionTemplate::new(scope, os_uptime_callback);
    let uptime_instance = uptime_func.get_function(scope).unwrap();
    let uptime_key = v8::String::new(scope, "uptime").unwrap();
    os_obj.set(scope, uptime_key.into(), uptime_instance.into());

    // cpus
    let cpus_func = v8::FunctionTemplate::new(scope, os_cpus_callback);
    let cpus_instance = cpus_func.get_function(scope).unwrap();
    let cpus_key = v8::String::new(scope, "cpus").unwrap();
    os_obj.set(scope, cpus_key.into(), cpus_instance.into());

    // freemem
    let freemem_func = v8::FunctionTemplate::new(scope, os_freemem_callback);
    let freemem_instance = freemem_func.get_function(scope).unwrap();
    let freemem_key = v8::String::new(scope, "freemem").unwrap();
    os_obj.set(scope, freemem_key.into(), freemem_instance.into());

    // totalmem
    let totalmem_func = v8::FunctionTemplate::new(scope, os_totalmem_callback);
    let totalmem_instance = totalmem_func.get_function(scope).unwrap();
    let totalmem_key = v8::String::new(scope, "totalmem").unwrap();
    os_obj.set(scope, totalmem_key.into(), totalmem_instance.into());

    // homedir
    let homedir_func = v8::FunctionTemplate::new(scope, os_homedir_callback);
    let homedir_instance = homedir_func.get_function(scope).unwrap();
    let homedir_key = v8::String::new(scope, "homedir").unwrap();
    os_obj.set(scope, homedir_key.into(), homedir_instance.into());

    // tmpdir
    let tmpdir_func = v8::FunctionTemplate::new(scope, os_tmpdir_callback);
    let tmpdir_instance = tmpdir_func.get_function(scope).unwrap();
    let tmpdir_key = v8::String::new(scope, "tmpdir").unwrap();
    os_obj.set(scope, tmpdir_key.into(), tmpdir_instance.into());

    // tmpDir (别名)
    let tmpdir_alias_func = v8::FunctionTemplate::new(scope, os_tmpdir_callback);
    let tmpdir_alias_instance = tmpdir_alias_func.get_function(scope).unwrap();
    let tmpdir_alias_key = v8::String::new(scope, "tmpDir").unwrap();
    os_obj.set(scope, tmpdir_alias_key.into(), tmpdir_alias_instance.into());

    // networkInterfaces
    let network_interfaces_func = v8::FunctionTemplate::new(scope, os_network_interfaces_callback);
    let network_interfaces_instance = network_interfaces_func.get_function(scope).unwrap();
    let network_interfaces_key = v8::String::new(scope, "networkInterfaces").unwrap();
    os_obj.set(scope, network_interfaces_key.into(), network_interfaces_instance.into());

    // constants
    let constants_obj = v8::Object::new(scope);

    // OS常量
    constants_obj.set(scope, v8::String::new(scope, "UV_UDP_REUSEADDR").unwrap().into(), v8::Integer::new(scope, 4).into());

    // Signal常量
    let signals_obj = v8::Object::new(scope);
    signals_obj.set(scope, v8::String::new(scope, "SIGHUP").unwrap().into(), v8::Integer::new(scope, 1).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGINT").unwrap().into(), v8::Integer::new(scope, 2).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGQUIT").unwrap().into(), v8::Integer::new(scope, 3).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGILL").unwrap().into(), v8::Integer::new(scope, 4).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGTRAP").unwrap().into(), v8::Integer::new(scope, 5).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGABRT").unwrap().into(), v8::Integer::new(scope, 6).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGIOT").unwrap().into(), v8::Integer::new(scope, 6).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGBUS").unwrap().into(), v8::Integer::new(scope, 7).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGFPE").unwrap().into(), v8::Integer::new(scope, 8).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGKILL").unwrap().into(), v8::Integer::new(scope, 9).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGUSR1").unwrap().into(), v8::Integer::new(scope, 10).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGUSR2").unwrap().into(), v8::Integer::new(scope, 12).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGPIPE").unwrap().into(), v8::Integer::new(scope, 13).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGALRM").unwrap().into(), v8::Integer::new(scope, 14).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGTERM").unwrap().into(), v8::Integer::new(scope, 15).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGCHLD").unwrap().into(), v8::Integer::new(scope, 17).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGCLD").unwrap().into(), v8::Integer::new(scope, 17).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGCONT").unwrap().into(), v8::Integer::new(scope, 18).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGSTOP").unwrap().into(), v8::Integer::new(scope, 19).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGTSTP").unwrap().into(), v8::Integer::new(scope, 20).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGTTIN").unwrap().into(), v8::Integer::new(scope, 21).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGTTOU").unwrap().into(), v8::Integer::new(scope, 22).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGURG").unwrap().into(), v8::Integer::new(scope, 23).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGXCPU").unwrap().into(), v8::Integer::new(scope, 24).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGXFSZ").unwrap().into(), v8::Integer::new(scope, 25).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGVTALRM").unwrap().into(), v8::Integer::new(scope, 26).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGPROF").unwrap().into(), v8::Integer::new(scope, 27).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGWINCH").unwrap().into(), v8::Integer::new(scope, 28).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGIO").unwrap().into(), v8::Integer::new(scope, 29).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGPOLL").unwrap().into(), v8::Integer::new(scope, 29).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGPWR").unwrap().into(), v8::Integer::new(scope, 30).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGSYS").unwrap().into(), v8::Integer::new(scope, 31).into());
    signals_obj.set(scope, v8::String::new(scope, "SIGUNUSED").unwrap().into(), v8::Integer::new(scope, 31).into());

    constants_obj.set(scope, v8::String::new(scope, "signals").unwrap().into(), signals_obj.into());

    os_obj.set(scope, v8::String::new(scope, "constants").unwrap().into(), constants_obj.into());

    // EOL常量
    let eol = if cfg!(windows) { "\r\n" } else { "\n" };
    os_obj.set(scope, v8::String::new(scope, "EOL").unwrap().into(), v8::String::new(scope, eol).unwrap().into());

    // arch()的常量版本
    let arch_constants_obj = v8::Object::new(scope);
    arch_constants_obj.set(scope, v8::String::new(scope, "arm").unwrap().into(), v8::String::new(scope, "arm").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "arm64").unwrap().into(), v8::String::new(scope, "arm64").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "ia32").unwrap().into(), v8::String::new(scope, "ia32").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "mips").unwrap().into(), v8::String::new(scope, "mips").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "mipsel").unwrap().into(), v8::String::new(scope, "mipsel").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "ppc").unwrap().into(), v8::String::new(scope, "ppc").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "ppc64").unwrap().into(), v8::String::new(scope, "ppc64").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "riscv64").unwrap().into(), v8::String::new(scope, "riscv64").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "s390").unwrap().into(), v8::String::new(scope, "s390").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "s390x").unwrap().into(), v8::String::new(scope, "s390x").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "x64").unwrap().into(), v8::String::new(scope, "x64").unwrap().into());
    arch_constants_obj.set(scope, v8::String::new(scope, "x86").unwrap().into(), v8::String::new(scope, "x86").unwrap().into());

    os_obj.set(scope, v8::String::new(scope, "arch").unwrap().into(), arch_constants_obj.into());

    // platform()的常量版本
    let platform_constants_obj = v8::Object::new(scope);
    platform_constants_obj.set(scope, v8::String::new(scope, "aix").unwrap().into(), v8::String::new(scope, "aix").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "darwin").unwrap().into(), v8::String::new(scope, "darwin").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "freebsd").unwrap().into(), v8::String::new(scope, "freebsd").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "linux").unwrap().into(), v8::String::new(scope, "linux").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "openbsd").unwrap().into(), v8::String::new(scope, "openbsd").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "sunos").unwrap().into(), v8::String::new(scope, "sunos").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "win32").unwrap().into(), v8::String::new(scope, "win32").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "android").unwrap().into(), v8::String::new(scope, "android").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "cygwin").unwrap().into(), v8::String::new(scope, "cygwin").unwrap().into());
    platform_constants_obj.set(scope, v8::String::new(scope, "netbsd").unwrap().into(), v8::String::new(scope, "netbsd").unwrap().into());

    os_obj.set(scope, v8::String::new(scope, "platform").unwrap().into(), platform_constants_obj.into());

    // 设置到全局
    let global = context.global(scope);
    let os_key = v8::String::new(scope, "os").unwrap();
    global.set(scope, os_key.into(), os_obj.into());

    Ok(())
}

fn os_arch_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let arch = std::env::consts::ARCH;
    retval.set(v8::String::new(scope, arch).unwrap().into());
}

fn os_platform_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let platform = std::env::consts::OS;
    retval.set(v8::String::new(scope, platform).unwrap().into());
}

fn os_type_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let os_type = if cfg!(windows) {
        "Windows_NT"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else {
        "Linux"
    };
    retval.set(v8::String::new(scope, os_type).unwrap().into());
}

fn os_release_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 简化实现，返回通用版本
    let release = "10.0.0";
    retval.set(v8::String::new(scope, release).unwrap().into());
}

fn os_hostname_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if let Ok(hostname) = env::var("HOSTNAME") {
        retval.set(v8::String::new(scope, &hostname).unwrap().into());
    } else if let Ok(computer_name) = env::var("COMPUTERNAME") {
        retval.set(v8::String::new(scope, &computer_name).unwrap().into());
    } else {
        retval.set(v8::String::new(scope, "localhost").unwrap().into());
    }
}

fn os_loadavg_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 简化的负载平均值
    let loadavg = v8::Array::new(scope, 3);
    loadavg.set_index(scope, 0, v8::Number::new(scope, 0.1).into());
    loadavg.set_index(scope, 1, v8::Number::new(scope, 0.2).into());
    loadavg.set_index(scope, 2, v8::Number::new(scope, 0.3).into());
    retval.set(loadavg.into());
}

fn os_uptime_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 获取系统运行时间
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    retval.set(v8::Number::new(scope, uptime as f64).into());
}

fn os_cpus_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 获取CPU数量
    let cpu_count = num_cpus::get();
    let cpus_array = v8::Array::new(scope, cpu_count as u32);

    for i in 0..cpu_count {
        let cpu_obj = v8::Object::new(scope);
        cpu_obj.set(scope, v8::String::new(scope, "model").unwrap().into(), v8::String::new(scope, "Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz").unwrap().into());
        cpu_obj.set(scope, v8::String::new(scope, "speed").unwrap().into(), v8::Number::new(scope, 3600.0).into());

        let times_obj = v8::Object::new(scope);
        times_obj.set(scope, v8::String::new(scope, "user").unwrap().into(), v8::Number::new(scope, 1000000.0).into());
        times_obj.set(scope, v8::String::new(scope, "nice").unwrap().into(), v8::Number::new(scope, 0.0).into());
        times_obj.set(scope, v8::String::new(scope, "sys").unwrap().into(), v8::Number::new(scope, 500000.0).into());
        times_obj.set(scope, v8::String::new(scope, "idle").unwrap().into(), v8::Number::new(scope, 8000000.0).into());
        times_obj.set(scope, v8::String::new(scope, "irq").unwrap().into(), v8::Number::new(scope, 0.0).into());

        cpu_obj.set(scope, v8::String::new(scope, "times").unwrap().into(), times_obj.into());
        cpus_array.set_index(scope, i as u32, cpu_obj.into());
    }

    retval.set(cpus_array.into());
}

fn os_freemem_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 简化的可用内存
    let freemem = 8 * 1024 * 1024 * 1024; // 8GB
    retval.set(v8::Number::new(scope, freemem as f64).into());
}

fn os_totalmem_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 简化的总内存
    let totalmem = 16 * 1024 * 1024 * 1024; // 16GB
    retval.set(v8::Number::new(scope, totalmem as f64).into());
}

fn os_homedir_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if let Ok(home_dir) = dirs::home_dir() {
        retval.set(v8::String::new(scope, &home_dir.to_string_lossy()).unwrap().into());
    } else {
        retval.set(v8::String::new(scope, "/home/user").unwrap().into());
    }
}

fn os_tmpdir_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let tmpdir = if cfg!(windows) {
        env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string())
    } else {
        "/tmp".to_string()
    };
    retval.set(v8::String::new(scope, &tmpdir).unwrap().into());
}

fn os_network_interfaces_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 简化的网络接口
    let interfaces_obj = v8::Object::new(scope);

    // Loopback接口
    let loopback_array = v8::Array::new(scope, 1);
    let loopback_obj = v8::Object::new(scope);
    loopback_obj.set(scope, v8::String::new(scope, "address").unwrap().into(), v8::String::new(scope, "127.0.0.1").unwrap().into());
    loopback_obj.set(scope, v8::String::new(scope, "netmask").unwrap().into(), v8::String::new(scope, "255.0.0.0").unwrap().into());
    loopback_obj.set(scope, v8::String::new(scope, "family").unwrap().into(), v8::String::new(scope, "IPv4").unwrap().into());
    loopback_obj.set(scope, v8::String::new(scope, "internal").unwrap().into(), v8::Boolean::new(scope, true).into());
    loopback_array.set_index(scope, 0, loopback_obj.into());

    interfaces_obj.set(scope, v8::String::new(scope, "lo").unwrap().into(), loopback_array.into());
    interfaces_obj.set(scope, v8::String::new(scope, "lo0").unwrap().into(), loopback_array.into());

    retval.set(interfaces_obj.into());
}
