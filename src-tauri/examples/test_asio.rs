/// ASUSManagement device_status 暴力扫描 + 高级 API 精准测试
///
/// 思路：桌面板的 Device ID 魔法字典跟笔记本不同，
/// 暴力扫描关键区间找出所有返回非零值的 ID。
/// 同时用正确参数测试 GetManualFanCurvePro / GetManualFanCurve。
///
/// 用法：以管理员身份运行
///   cargo run --example test_asio
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL,
    RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Variant::{
    VariantChangeType, VARIANT, VAR_CHANGE_FLAGS, VT_EMPTY, VT_I4, VT_NULL,
};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, WbemLocator, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_LOCAL_ONLY,
    WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_FLAG_RETURN_WBEM_COMPLETE,
};

static LOG: std::sync::LazyLock<Mutex<File>> = std::sync::LazyLock::new(|| {
    let path = std::path::PathBuf::from(r"E:\Repo\NoCrate\asio_diag.txt");
    let f = File::create(&path).expect(&format!("cannot create {}", path.display()));
    Mutex::new(f)
});

macro_rules! log {
    ($($tt:tt)*) => {{
        let msg = format!($($tt)*);
        print!("{}", msg);
        if let Ok(mut f) = LOG.lock() {
            let _ = write!(f, "{}", msg);
            let _ = f.flush();
        }
    }};
}

macro_rules! logln {
    ($($tt:tt)*) => {{
        let msg = format!($($tt)*);
        println!("{}", msg);
        if let Ok(mut f) = LOG.lock() {
            let _ = writeln!(f, "{}", msg);
            let _ = f.flush();
        }
    }};
}

fn main() {
    logln!("=== ASUSManagement device_status 暴力扫描 + 高级 API 精准测试 ===\n");
    unsafe {
        if let Err(e) = run_test() {
            logln!("测试失败: {e}");
        }
    }
}

// ─── 辅助类型 ───
enum WmiParamVal<'a> {
    U8(u8),
    U32(u32),
    Str(&'a str),
}

unsafe fn get_property_u32(obj: &IWbemClassObject, name: &str) -> Result<u32, String> {
    let mut val = VARIANT::default();
    obj.Get(&BSTR::from(name), 0, &mut val, None, None)
        .map_err(|e| format!("Get({name}): {e}"))?;
    let vt = val.Anonymous.Anonymous.vt;
    if vt == VT_EMPTY || vt == VT_NULL {
        return Ok(0);
    }
    if let Ok(i4) = i32::try_from(&val) {
        return Ok(i4 as u32);
    }
    let mut coerced = VARIANT::default();
    VariantChangeType(&mut coerced, &val, VAR_CHANGE_FLAGS(0), VT_I4)
        .map_err(|e| format!("VariantChangeType({name}): {e}"))?;
    let i4: i32 = (&coerced)
        .try_into()
        .map_err(|_| format!("{name}: 转后非 i32"))?;
    Ok(i4 as u32)
}

unsafe fn get_property_string(obj: &IWbemClassObject, name: &str) -> Result<String, String> {
    let mut val = VARIANT::default();
    obj.Get(&BSTR::from(name), 0, &mut val, None, None)
        .map_err(|e| format!("Get({name}): {e}"))?;
    let bstr: BSTR = (&val).try_into().map_err(|_| format!("{name}: 非字符串"))?;
    Ok(bstr.to_string())
}

unsafe fn run_test() -> Result<(), String> {
    // ─── 初始化 COM ───
    CoInitializeEx(None, COINIT_MULTITHREADED)
        .ok()
        .map_err(|e| format!("CoInitializeEx: {e}"))?;

    let _ = CoInitializeSecurity(
        None,
        -1,
        None,
        None,
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
        None,
    );

    let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
        .map_err(|e| format!("WbemLocator: {e}"))?;

    let services = locator
        .ConnectServer(
            &BSTR::from("root\\WMI"),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            0,
            &BSTR::new(),
            None,
        )
        .map_err(|e| format!("ConnectServer: {e}"))?;
    logln!("✓ 已连接 root\\WMI");

    CoSetProxyBlanket(
        &services,
        10,
        0,
        None,
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
    )
    .map_err(|e| format!("CoSetProxyBlanket: {e}"))?;

    // ─── 获取 ASUSManagement 实例路径 + 类定义 ───
    let enumerator = services
        .CreateInstanceEnum(
            &BSTR::from("ASUSManagement"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )
        .map_err(|e| format!("CreateInstanceEnum: {e}"))?;

    let mut row = [None; 1];
    let mut returned: u32 = 0;
    enumerator
        .Next(5000, &mut row, &mut returned)
        .ok()
        .map_err(|e| format!("Next: {e}"))?;
    if returned == 0 || row[0].is_none() {
        return Err("未找到 ASUSManagement 实例".into());
    }
    let inst = row[0].as_ref().unwrap();
    let mut path_val = VARIANT::default();
    inst.Get(&BSTR::from("__RELPATH"), 0, &mut path_val, None, None)
        .map_err(|e| format!("Get __RELPATH: {e}"))?;
    let instance_path: String = BSTR::try_from(&path_val)
        .map_err(|_| "__RELPATH 非字符串".to_string())?
        .to_string();
    logln!("✓ 实例路径: {instance_path}");

    let mut class_obj_opt = None;
    services
        .GetObject(
            &BSTR::from("ASUSManagement"),
            WBEM_FLAG_RETURN_WBEM_COMPLETE,
            None,
            Some(&mut class_obj_opt),
            None,
        )
        .map_err(|e| format!("GetObject: {e}"))?;
    let class_obj = class_obj_opt.ok_or("类定义为空")?;

    // ─── 通用方法调用闭包 ───
    let call = |method: &str, params: &[(&str, WmiParamVal)]| -> Result<IWbemClassObject, String> {
        let mut in_def = None;
        class_obj
            .GetMethod(&BSTR::from(method), 0, &mut in_def, &mut None)
            .map_err(|e| format!("GetMethod({method}): {e}"))?;

        let in_params = if let Some(def) = in_def {
            let instance = def
                .SpawnInstance(0)
                .map_err(|e| format!("SpawnInstance: {e}"))?;
            for (name, value) in params {
                let variant = match value {
                    WmiParamVal::U8(v) => VARIANT::from(*v as i32),
                    WmiParamVal::U32(v) => VARIANT::from(*v as i32),
                    WmiParamVal::Str(s) => VARIANT::from(BSTR::from(*s)),
                };
                instance
                    .Put(&BSTR::from(*name), 0, &variant, 0)
                    .map_err(|e| format!("Put({name}): {e}"))?;
            }
            Some(instance)
        } else {
            None
        };

        let mut out = None;
        services
            .ExecMethod(
                &BSTR::from(instance_path.as_str()),
                &BSTR::from(method),
                Default::default(),
                None,
                in_params.as_ref(),
                Some(&mut out),
                None,
            )
            .map_err(|e| format!("ExecMethod({method}): {e}"))?;

        out.ok_or_else(|| format!("{method}: 无输出"))
    };

    // ═══════════════════════════════════════════
    //  第一步：GetFanPolicy 基线验证
    // ═══════════════════════════════════════════
    logln!("\n══════════════════════════════════════════");
    logln!("  基线验证: GetFanPolicy");
    logln!("══════════════════════════════════════════");

    for ft in 0..8u8 {
        match call("GetFanPolicy", &[("FanType", WmiParamVal::U8(ft))]) {
            Ok(out) => {
                let ec = get_property_u32(&out, "ErrorCode").unwrap_or(u32::MAX);
                if ec == 0 {
                    let mode = get_property_string(&out, "Mode").unwrap_or_default();
                    let profile = get_property_string(&out, "Profile").unwrap_or_default();
                    let source = get_property_string(&out, "Source").unwrap_or_default();
                    let low_limit = get_property_u32(&out, "LowLimit").unwrap_or(0);
                    logln!(
                        "  FanType={ft}: Mode={mode}, Profile={profile}, Source={source}, LowLimit={low_limit}"
                    );
                }
            }
            Err(_) => {}
        }
    }

    // ═══════════════════════════════════════════
    //  第二步：GetManualFanCurvePro — 用正确的 Mode 参数
    // ═══════════════════════════════════════════
    logln!("\n══════════════════════════════════════════");
    logln!("  精准测试: GetManualFanCurvePro");
    logln!("══════════════════════════════════════════");

    for ft in 0..8u8 {
        for mode in &["PWM", "DC", "AUTO"] {
            match call(
                "GetManualFanCurvePro",
                &[
                    ("FanType", WmiParamVal::U8(ft)),
                    ("Mode", WmiParamVal::Str(mode)),
                ],
            ) {
                Ok(out) => {
                    let ec = get_property_u32(&out, "ErrorCode").unwrap_or(u32::MAX);
                    if ec == 0 {
                        logln!("  ★ FanType={ft}, Mode={mode}:");
                        for pt in 1..=8 {
                            let temp =
                                get_property_u32(&out, &format!("Point{pt}Temp")).unwrap_or(0);
                            let duty =
                                get_property_u32(&out, &format!("Point{pt}Duty")).unwrap_or(0);
                            log!("    P{pt}: {temp}°C→{duty}%");
                        }
                        logln!("");
                    } else {
                        logln!("  FanType={ft}, Mode={mode} → ErrorCode={ec}");
                    }
                }
                Err(e) => logln!("  FanType={ft}, Mode={mode} → {e}"),
            }
        }
    }

    // ═══════════════════════════════════════════
    //  第三步：GetManualFanCurve — 用正确的 Mode 参数
    // ═══════════════════════════════════════════
    logln!("\n══════════════════════════════════════════");
    logln!("  精准测试: GetManualFanCurve");
    logln!("══════════════════════════════════════════");

    for ft in 0..4u8 {
        for mode in &["PWM", "DC", "AUTO"] {
            match call(
                "GetManualFanCurve",
                &[
                    ("FanType", WmiParamVal::U8(ft)),
                    ("Mode", WmiParamVal::Str(mode)),
                ],
            ) {
                Ok(out) => {
                    let ec = get_property_u32(&out, "ErrorCode").unwrap_or(u32::MAX);
                    if ec == 0 {
                        let lt = get_property_u32(&out, "LowTemp").unwrap_or(0);
                        let ld = get_property_u32(&out, "LowDuty").unwrap_or(0);
                        let mt = get_property_u32(&out, "MidTemp").unwrap_or(0);
                        let md = get_property_u32(&out, "MidDuty").unwrap_or(0);
                        let ht = get_property_u32(&out, "HighTemp").unwrap_or(0);
                        let hd = get_property_u32(&out, "HighDuty").unwrap_or(0);
                        logln!(
                            "  ★ FanType={ft}, Mode={mode}: Low={lt}°C→{ld}% Mid={mt}°C→{md}% High={ht}°C→{hd}%"
                        );
                    } else {
                        logln!("  FanType={ft}, Mode={mode} → ErrorCode={ec}");
                    }
                }
                Err(e) => logln!("  FanType={ft}, Mode={mode} → {e}"),
            }
        }
    }

    // ═══════════════════════════════════════════
    //  第四步：device_status 暴力扫描
    // ═══════════════════════════════════════════
    logln!("\n══════════════════════════════════════════");
    logln!("  device_status 暴力扫描");
    logln!("══════════════════════════════════════════");

    // 华硕 DSTS 的 Device ID 编码规则:
    //   高16位 = 设备类(DevType), 低16位 = 设备序号(DevNum)
    // 已知笔记本常见前缀: 0x0011xxxx (风扇), 0x0012xxxx (电源/策略)
    // 桌面板可能用完全不同的前缀
    let ranges: &[(&str, std::ops::RangeInclusive<u32>)] = &[
        // 基础区间
        ("0x0000_00xx 基础状态", 0x00000000..=0x000000FF),
        // 各个 DevType 的前 256 个设备
        ("0x0001_00xx", 0x00010000..=0x000100FF),
        ("0x0002_00xx", 0x00020000..=0x000200FF),
        ("0x0003_00xx", 0x00030000..=0x000300FF),
        ("0x0004_00xx", 0x00040000..=0x000400FF),
        ("0x0005_00xx", 0x00050000..=0x000500FF),
        ("0x0006_00xx", 0x00060000..=0x000600FF),
        ("0x0007_00xx", 0x00070000..=0x000700FF),
        ("0x0008_00xx", 0x00080000..=0x000800FF),
        ("0x0009_00xx", 0x00090000..=0x000900FF),
        ("0x000A_00xx", 0x000A0000..=0x000A00FF),
        ("0x000B_00xx", 0x000B0000..=0x000B00FF),
        ("0x000C_00xx", 0x000C0000..=0x000C00FF),
        ("0x000D_00xx", 0x000D0000..=0x000D00FF),
        ("0x000E_00xx", 0x000E0000..=0x000E00FF),
        ("0x000F_00xx", 0x000F0000..=0x000F00FF),
        ("0x0010_00xx", 0x00100000..=0x001000FF),
        // 笔记本已知的核心区间 — 桌面板可能复用
        ("0x0011_00xx 风扇/温度", 0x00110000..=0x001100FF),
        ("0x0012_00xx 电源/策略", 0x00120000..=0x001200FF),
        ("0x0013_00xx", 0x00130000..=0x001300FF),
        ("0x0014_00xx", 0x00140000..=0x001400FF),
        // 更高区间
        ("0x0020_00xx", 0x00200000..=0x002000FF),
        ("0x0021_00xx", 0x00210000..=0x002100FF),
        ("0x0030_00xx", 0x00300000..=0x003000FF),
        ("0x0040_00xx", 0x00400000..=0x004000FF),
        ("0x0050_00xx", 0x00500000..=0x005000FF),
        ("0x0060_00xx", 0x00600000..=0x006000FF),
        // AURA/LED 区间
        ("0x0010_10xx AURA", 0x00101000..=0x001010FF),
    ];

    let mut total_found = 0u32;

    for (label, range) in ranges {
        log!("  扫描 {label} ...");
        let mut hits = 0u32;
        for did in range.clone() {
            match call("device_status", &[("device_id", WmiParamVal::U32(did))]) {
                Ok(out) => {
                    let val = get_property_u32(&out, "ctrl_param").unwrap_or(0);
                    if val != 0 && val != 0xFFFFFFFF {
                        if hits == 0 {
                            logln!("");
                        }
                        logln!("    ★ 0x{did:08X} → {val} (0x{val:08X})");
                        hits += 1;
                    }
                }
                Err(_) => {}
            }
        }
        if hits == 0 {
            logln!(" 无有效数据");
        } else {
            logln!("    小计: {hits} 个命中");
        }
        total_found += hits;
    }

    logln!("\n  ═══ 暴力扫描结束！共 {total_found} 个有效 Device ID ═══");

    // ═══════════════════════════════════════════
    //  第五步：GetLastError 诊断
    // ═══════════════════════════════════════════
    logln!("\n══════════════════════════════════════════");
    logln!("  GetLastError 诊断");
    logln!("══════════════════════════════════════════");
    match call("GetLastError", &[]) {
        Ok(out) => {
            let ec = get_property_u32(&out, "ErrorCode").unwrap_or(u32::MAX);
            logln!("  ErrorCode = {ec} (0x{ec:08X})");
        }
        Err(e) => logln!("  {e}"),
    }

    logln!("\n══════════════════════════════════════════");
    logln!("  完成");
    logln!("══════════════════════════════════════════");
    Ok(())
}

/// 转储实例的所有非系统属性
unsafe fn dump_instance_props(obj: &IWbemClassObject) {
    if obj.BeginEnumeration(0).is_err() {
        logln!("    [BeginEnumeration 失败]");
        return;
    }
    let mut count = 0;
    loop {
        let mut name = BSTR::new();
        let mut val = VARIANT::default();
        let mut cim_type: i32 = 0;
        let hr = obj.Next(0, &mut name, &mut val, &mut cim_type, std::ptr::null_mut());
        if hr.is_err() || name.is_empty() {
            break;
        }
        let name_str = name.to_string();
        if name_str.starts_with("__") {
            continue;
        }
        count += 1;
        let vt = val.Anonymous.Anonymous.vt;
        let val_str = if vt == VT_EMPTY || vt == VT_NULL {
            "(null)".to_string()
        } else if let Ok(i4) = i32::try_from(&val) {
            format!("{i4} (0x{:08X})", i4 as u32)
        } else if let Ok(bstr) = BSTR::try_from(&val) {
            let s = bstr.to_string();
            if s.len() > 120 {
                format!("'{}'...", &s[..120])
            } else {
                format!("'{s}'")
            }
        } else {
            format!("(VT={:?})", vt)
        };
        logln!(
            "    {}: {} = {}",
            name_str,
            cim_type_name(cim_type),
            val_str
        );
    }
    let _ = obj.EndEnumeration();
    if count == 0 {
        logln!("    (无可读属性)");
    }
}

/// CIM type ID → 可读名称
fn cim_type_name(cim: i32) -> &'static str {
    match cim {
        2 => "sint16",
        3 => "sint32",
        4 => "real32",
        5 => "real64",
        8 => "string",
        11 => "bool",
        13 => "object",
        16 => "sint8",
        17 => "uint8",
        18 => "uint16",
        19 => "uint32",
        20 => "sint64",
        21 => "uint64",
        101 => "datetime",
        102 => "reference",
        103 => "char16",
        8192.. => "array",
        _ => "unknown",
    }
}
