//! The app's Rust UI: reminder 桌面通用健康与生活提醒悬浮挂件
//! 纯 Rust 编写、固定深色质感、使用 NSH 官方原生 API 隐藏标题栏、支持随意拖拽、矢量环形进度图标

pub mod icons;
pub mod model;

use std::time::Duration;
use nsh_core::prelude::*;
use nsh_core::element::{desktop, input};
use nsh_core::desktop::{WindowFrame, WindowLevel};
use nsh_base::{SliderValue, slider, collapsible};
use model::ReminderItem;

#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(target_os = "windows")]
static APP_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

#[cfg(target_os = "windows")]
static CURRENT_OPACITY_PERCENT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(100);

/// 设置 Windows 原生窗口透明度 (Layered Window Alpha)
#[cfg(target_os = "windows")]
pub fn set_native_window_opacity(opacity: f32) {
    let opacity = opacity.clamp(0.20, 1.0);
    let percent = (opacity * 100.0).round() as u32;
    CURRENT_OPACITY_PERCENT.store(percent, std::sync::atomic::Ordering::SeqCst);

    let raw = APP_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if raw == 0 {
        return;
    }
    use platform::windows::Win32::Foundation::{HWND, COLORREF};
    use platform::windows::Win32::UI::WindowsAndMessaging::*;

    let hwnd = HWND(raw as *mut _);
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if (ex_style & (WS_EX_LAYERED.0 as i32)) == 0 {
            let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);
        }
        let alpha = (opacity * 255.0).round() as u8;
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

/// 跨平台透明度应用接口
pub fn apply_window_opacity(opacity: f32) {
    #[cfg(target_os = "windows")]
    set_native_window_opacity(opacity);
    let _ = opacity;
}

/// Windows 挂件专用：配置为纯净无边框悬浮卡片，移除全部系统按钮与非客户区，保留 Win11 硬件圆角与阴影
#[cfg(target_os = "windows")]
fn setup_floating_window() {
    std::thread::spawn(|| {
        use platform::windows::Win32::Foundation::{HWND, LPARAM, COLORREF};
        use platform::windows::Win32::System::Threading::GetCurrentProcessId;
        use platform::windows::Win32::UI::WindowsAndMessaging::*;
        use platform::windows::Win32::Graphics::Dwm::*;
        use platform::windows::core::BOOL;

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let mut pid = 0;
                let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == GetCurrentProcessId() {
                    let mut class = [0u16; 256];
                    let clen = GetClassNameW(hwnd, &mut class);
                    let class_name = String::from_utf16_lossy(&class[..clen as usize]);
                    if class_name == "NshWindow" {
                        *(lparam.0 as *mut HWND) = hwnd;
                        return BOOL(0);
                    }
                }
            }
            BOOL(1)
        }

        for _ in 0..50 {
            std::thread::sleep(std::time::Duration::from_millis(30));
            let mut target_hwnd = HWND::default();
            let _ = unsafe { EnumWindows(Some(enum_proc), LPARAM(&mut target_hwnd as *mut _ as isize)) };

            if !target_hwnd.is_invalid() {
                unsafe {
                    APP_HWND.store(target_hwnd.0 as isize, std::sync::atomic::Ordering::SeqCst);

                    // 1. 设置为纯净无边框悬浮挂件卡片样式 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS)
                    //    彻底剥离 WS_CAPTION、WS_THICKFRAME 以及系统三键，不留任何非客户区边框
                    let popup_style = (WS_POPUP.0 | WS_VISIBLE.0 | WS_CLIPSIBLINGS.0) as i32;
                    SetWindowLongW(target_hwnd, GWL_STYLE, popup_style);

                    // 2. 启用分层窗口 (WS_EX_LAYERED) 并应用当前预设透明度
                    let ex_style = GetWindowLongW(target_hwnd, GWL_EXSTYLE);
                    SetWindowLongW(target_hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);
                    let percent = CURRENT_OPACITY_PERCENT.load(std::sync::atomic::Ordering::SeqCst);
                    let alpha = ((percent as f32 / 100.0) * 255.0).round() as u8;
                    let _ = SetLayeredWindowAttributes(target_hwnd, COLORREF(0), alpha, LWA_ALPHA);

                    // 3. 启用 Win11 原生硬件级平滑圆角 (DWMWCP_ROUND = 2)
                    let round_pref: u32 = 2;
                    let _ = DwmSetWindowAttribute(
                        target_hwnd,
                        DWMWA_WINDOW_CORNER_PREFERENCE,
                        &round_pref as *const _ as *const _,
                        std::mem::size_of_val(&round_pref) as u32,
                    );

                    // 4. 启用系统原生精致阴影 (CS_DROPSHADOW)
                    let class_style = GetClassLongPtrW(target_hwnd, GCL_STYLE);
                    SetClassLongPtrW(target_hwnd, GCL_STYLE, (class_style | CS_DROPSHADOW.0 as usize) as isize);

                    // 5. 刷新非客户区，不改变位置、不改变尺寸、不改变Z序、不抢焦点
                    let _ = SetWindowPos(
                        target_hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }
                break;
            }
        }
    });
}

/// 统一窗口尺寸规范（逻辑像素，自动由 NSH 处理 High-DPI 缩放，杜绝模糊与尺寸割裂）
pub const WINDOW_WIDTH: f32 = 320.0;
pub const BASE_CONTENT_HEIGHT: f32 = 102.0; // padding (28) + header (30) + footer (22) + gaps (22)
pub const ITEM_ROW_HEIGHT: f32 = 46.0;
pub const DRAWER_BASE_HEIGHT: f32 = 126.0;
pub const DRAWER_PRESETS_HEIGHT: f32 = 112.0;
pub const DRAWER_CUSTOM_HEIGHT: f32 = 208.0;

/// 根据提醒项目数量、设置抽屉展开状态、常用推荐展开状态以及自定义表单展开状态精确计算窗口高度
pub fn calculate_window_height(
    item_count: usize,
    is_drawer_open: bool,
    is_presets_open: bool,
    is_custom_open: bool,
) -> f32 {
    let items_h = item_count as f32 * ITEM_ROW_HEIGHT;
    let drawer_h = if is_drawer_open {
        let mut h = DRAWER_BASE_HEIGHT;
        if is_presets_open {
            h += DRAWER_PRESETS_HEIGHT;
        }
        if is_custom_open {
            h += DRAWER_CUSTOM_HEIGHT;
        }
        h
    } else {
        0.0
    };
    (BASE_CONTENT_HEIGHT + items_h + drawer_h).clamp(195.0, 720.0)
}

/// 触发倒计时到期提醒（声音、系统原生通知、窗口置顶唤起）
pub fn trigger_reminder_alarm(item_name: &str) {
    // 1. 窗口唤起置顶：确保用户即使在打游戏或全屏工作也能看到悬浮挂件
    desktop::order_front();

    // 2. 播放系统提醒音
    #[cfg(target_os = "windows")]
    unsafe {
        use platform::windows::Win32::System::Diagnostics::Debug::MessageBeep;
        use platform::windows::Win32::UI::WindowsAndMessaging::MB_ICONEXCLAMATION;
        let _ = MessageBeep(MB_ICONEXCLAMATION);
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::io::Write;
        print!("\x07");
        let _ = std::io::stdout().flush();
    }

    // 3. 发送跨平台系统原生通知 (Windows Toast / macOS Notification / Linux D-Bus)
    let name = item_name.to_string();
    std::thread::spawn(move || {
        let _ = notify_rust::Notification::new()
            .summary("reminder 定时提醒")
            .body(&format!("{} 到时间啦！请及时打卡或推迟。", name))
            .timeout(notify_rust::Timeout::Milliseconds(6000))
            .show();
    });
}

#[native_module("reminder/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// 预设提醒模板（通用丰富场景）
struct PresetReminder {
    name: &'static str,
    icon_key: &'static str,
    minutes: u32,
}

const PRESETS: &[PresetReminder] = &[
    PresetReminder { name: "喝水", icon_key: "droplet", minutes: 30 },
    PresetReminder { name: "起身活动", icon_key: "walk", minutes: 45 },
    PresetReminder { name: "护眼远眺", icon_key: "eye", minutes: 20 },
    PresetReminder { name: "番茄专注", icon_key: "clock", minutes: 25 },
    PresetReminder { name: "带薪拉屎", icon_key: "toilet", minutes: 90 },
    PresetReminder { name: "站立拉伸", icon_key: "activity", minutes: 60 },
];

/// 应用程序根组件
#[component]
pub fn App() -> impl IntoElement {
    // 1. Windows 挂件专用：配置为纯净无边框悬浮卡片，彻底剔除原生系统标题栏与系统控制三键
    #[cfg(target_os = "windows")]
    setup_floating_window();

    // 2. 窗口原生桌面属性：置顶悬浮 (Floating)
    desktop::set_level(WindowLevel::Floating);

    // 3. 强制暗色模式：底层清屏彻底变为深黑色，杜绝任何浅色主题底色透出
    nsh_core::element::set_color_scheme(nsh_core::platform::Appearance::Dark);

    // 4. 状态管理：默认预置通用基础提醒（喝水 30min、活动 45min、带薪拉屎 90min）
    let items = signal(vec![
        ReminderItem::new("喝水", "droplet", 30 * 60),
        ReminderItem::new("起身活动", "walk", 45 * 60),
        ReminderItem::new("带薪拉屎", "toilet", 90 * 60),
    ]);

    // 全局暂停状态与设置面板展开状态，以及窗口不透明度状态 (0.2 ~ 1.0)
    let is_all_paused = signal(false);
    let show_settings = signal(false);
    let show_presets = signal(false);
    let show_custom_form = signal(false);
    let opacity = signal(1.0_f32);

    // 窗口不透明度响应式联动：当 opacity 发生变化时，实时调用原生窗口 API
    effect(move || {
        let val = opacity.get();
        apply_window_opacity(val);
    });

    // 窗口高度动态自适应：点击设置抽屉或增减提醒项时动态调节高度，并支持平滑滚动，杜绝遮挡
    effect(move || {
        let is_open = show_settings.get();
        let is_presets = show_presets.get();
        let is_custom = show_custom_form.get();
        let count = items.get().len();
        let target_h = calculate_window_height(count, is_open, is_presets, is_custom);

        desktop::set_frame(WindowFrame {
            width: Some(px(WINDOW_WIDTH)),
            height: Some(px(target_h)),
            ..Default::default()
        });
    });

    // 每秒驱动倒计时更新与到期提醒
    let timer = stored(None::<Spawned<()>>);
    timer.replace(Some(set_interval(Duration::from_secs(1), move || {
        if !is_all_paused.get() {
            let mut newly_expired = Vec::new();
            items.update(|list| {
                for item in list.iter_mut() {
                    if item.tick() {
                        newly_expired.push(item.name.clone());
                    }
                }
            });
            for name in newly_expired {
                trigger_reminder_alarm(&name);
            }
        }
    })));

    // 纯正深灰冷黑底色（固定深色，不随操作系统亮色模式变化，保持精致挂件质感）
    let card_bg = hsl(225.0, 9.0, 15.0);

    // 根视图容器：纯净深灰黑铺满整个窗口，零白边，无多余外框，各平台原生呈现
    view()
        .window_drag(true) // 按住卡片空白处即可随意拖动窗口
        .w_full()
        .h_full()
        .bg(card_bg)
        .p(px(14.0))
        .flex_col()
        .gap(px(8.0))
        // 1. 顶栏：通用「reminder 提醒」金黄微标与标题，右侧挂件操作按钮组（暂停、设置、关闭）
        .child(render_header(items, is_all_paused, show_settings))
        // 2. 新建/设置抽屉（点击顶部设置/新建时平滑展开，窗口自动变大，支持不透明度调节与快捷提醒添加）
        .child(render_drawer(show_settings, show_presets, show_custom_form, items, opacity))
        // 3. 核心提醒列表（基于 each 响应式驱动更新与增减）
        .child(render_items(items))
        // 4. 底部状态行（通用统计：「进行中 X 项 · 今日打卡 Y 次」，若处于半透明状态则额外指示不透明度）
        .child(render_footer(items, opacity))
}

/// 顶部 Header：左侧金黄微标与标题，右侧挂件操作按钮组（暂停、设置/添加、关闭退出）
fn render_header(
    items: Signal<Vec<ReminderItem>>,
    is_all_paused: Signal<bool>,
    show_settings: Signal<bool>,
) -> impl IntoElement {
    view()
        .window_drag(true)
        .flex_row()
        .items_center()
        .justify_between()
        .pb(px(4.0))
        // 左侧：微标 + 标题 + 数量指示
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    // 金黄/暖琥珀色微标小框
                    view()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded(px(6.0))
                        .bg(hsl(45.0, 25.0, 18.0))
                        .items_center()
                        .justify_center()
                        .child(
                            icons::icon_svg(icons::ICON_BELL, 13.0, hsl(45.0, 90.0, 60.0)),
                        ),
                )
                .child(
                    text("reminder 提醒")
                        .text_size(px(13.5))
                        .text_color(white().alpha(0.95)),
                )
                .child(
                    // 进行中数量指示小徽章
                    view()
                        .px(px(6.0))
                        .py(px(1.5))
                        .rounded(px(10.0))
                        .bg(white().alpha(0.08))
                        .child(
                            text(move || format!("{} 项", items.get().len()))
                                .text_size(px(10.5))
                                .text_color(white().alpha(0.60)),
                        ),
                ),
        )
        // 右侧挂件专属操作按钮组：暂停/恢复全部、展开设置/新建抽屉、关闭退出挂件
        .child(
            view()
                .window_drag(false) // 按钮区域不触发窗口拖拽
                .flex_row()
                .items_center()
                .gap(px(6.0))
                // 暂停 / 播放全部
                .child(
                    view()
                        .w(px(22.0))
                        .h(px(22.0))
                        .rounded(px(5.0))
                        .items_center()
                        .justify_center()
                        .bg(white().alpha(0.04))
                        .active(|s| s.opacity(0.5))
                        .on_press(move || is_all_paused.update(|v| *v = !*v))
                        .child(
                            when(
                                move || is_all_paused.get(),
                                move || icons::icon_svg(icons::ICON_PLAY, 11.0, hsl(45.0, 90.0, 60.0)),
                            )
                            .otherwise(move || icons::icon_svg(icons::ICON_PAUSE, 11.0, white().alpha(0.65))),
                        ),
                )
                // 设置 / 新建
                .child(
                    view()
                        .w(px(22.0))
                        .h(px(22.0))
                        .rounded(px(5.0))
                        .items_center()
                        .justify_center()
                        .bg(move || {
                            if show_settings.get() {
                                white().alpha(0.18)
                            } else {
                                white().alpha(0.04)
                            }
                        })
                        .active(|s| s.opacity(0.5))
                        .on_press(move || show_settings.update(|v| *v = !*v))
                        .child(
                            icons::icon_svg(icons::ICON_SETTINGS, 12.0, white().alpha(0.70)),
                        ),
                )
                // 退出挂件 (轻量悬浮叉叉)
                .child(
                    view()
                        .w(px(22.0))
                        .h(px(22.0))
                        .rounded(px(5.0))
                        .items_center()
                        .justify_center()
                        .bg(white().alpha(0.04))
                        .active(|s| s.opacity(0.5))
                        .on_press(move || desktop::close_window())
                        .child(
                            icons::icon_svg(icons::ICON_X, 11.0, white().alpha(0.65)),
                        ),
                ),
        )
}

/// 核心列表：包裹在 scroll 容器中，使用 each 原语基于 key 响应式高效追踪每个 item 的变化
fn render_items(items: Signal<Vec<ReminderItem>>) -> impl IntoElement {
    scroll()
        .window_drag(true)
        .w_full()
        .flex_1()
        .scrollbar(false)
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(6.0))
                .child(each(
                    items,
                    |item| item.id,
                    move |item_sig, _| render_item_row(item_sig, items),
                )),
        )
}

/// 单个提醒行：左侧发光进度图标 + 中间名称倒计时 + 完成次数 + 右侧 [✓] [+5] [🗑]
fn render_item_row(
    item_sig: ReadSignal<ReminderItem>,
    items: Signal<Vec<ReminderItem>>,
) -> impl IntoElement {
    let item_id = item_sig.get_untracked().id;
    let icon_key = item_sig.get_untracked().icon.clone();
    let hex_color = icons::get_item_color(&icon_key);

    view()
        .window_drag(true)
        .flex_row()
        .items_center()
        .justify_between()
        .py(px(4.0))
        .px(px(4.0))
        .rounded(px(6.0))
        // 到期时微光高亮警示底色
        .bg(move || {
            if item_sig.get().is_expired() {
                hsl(0.0, 75.0, 25.0).alpha(0.35)
            } else {
                white().alpha(0.0)
            }
        })
        // 左半部分：圆环进度图标 + 标题/时间
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                // 左侧发光圆环进度图标（到期时红光警示）
                .child(
                    view()
                        .w(px(36.0))
                        .h(px(36.0))
                        .items_center()
                        .justify_center()
                        .child(
                            svg(move || {
                                let it = item_sig.get();
                                let is_exp = it.is_expired();
                                let color = if is_exp { "#f87171" } else { hex_color };
                                let ratio = if is_exp { 1.0 } else { it.progress_ratio() };
                                let markup = icons::ring_progress_svg(&icon_key, ratio, color);
                                nsh_core::element::ImageSrc::svg_markup(&markup)
                            })
                            .w(px(36.0))
                            .h(px(36.0)),
                        ),
                )
                // 标题与倒计时（包含周期微标，点击周期可快速切换常见时长，到期时红字警示）
                .child(
                    view()
                        .flex_col()
                        .justify_center()
                        .gap(px(2.0))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(5.0))
                                .child(
                                    text(move || item_sig.get().name.clone())
                                        .text_size(px(13.0))
                                        .text_color(white().alpha(0.92)),
                                )
                                .child(
                                    view()
                                        .window_drag(false)
                                        .px(px(4.0))
                                        .py(px(0.5))
                                        .rounded(px(3.0))
                                        .bg(white().alpha(0.06))
                                        .border_1()
                                        .border_color(white().alpha(0.08))
                                        .active(|s| s.opacity(0.5))
                                        .on_press(move || {
                                            items.update(|list| {
                                                if let Some(it) = list.iter_mut().find(|x| x.id == item_id) {
                                                    let cur_m = it.interval_secs / 60;
                                                    let next_m = match cur_m {
                                                        0..=19 => 20,
                                                        20..=24 => 25,
                                                        25..=29 => 30,
                                                        30..=44 => 45,
                                                        45..=59 => 60,
                                                        60..=89 => 90,
                                                        90..=119 => 120,
                                                        _ => 15,
                                                    };
                                                    it.update_interval_mins(next_m);
                                                }
                                            });
                                        })
                                        .child(
                                            text(move || format!("{}m", item_sig.get().interval_secs / 60))
                                                .text_size(px(10.0))
                                                .text_color(white().alpha(0.50)),
                                        ),
                                ),
                        )
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(3.0))
                                .child(
                                    when(
                                        move || item_sig.get().is_expired(),
                                        move || icons::icon_svg(icons::ICON_ALERT, 11.0, hsl(0.0, 85.0, 65.0)),
                                    ),
                                )
                                .child(
                                    text(move || item_sig.get().format_remaining())
                                        .text_size(px(11.5))
                                        .text_color(move || {
                                            if item_sig.get().is_expired() {
                                                hsl(0.0, 85.0, 68.0)
                                            } else {
                                                white().alpha(0.60)
                                            }
                                        }),
                                ),
                        ),
                ),
        )
        // 右半部分：完成度「0 / 8」 + 胶囊按钮组 [✓] [+5] [🗑]
        .child(
            view()
                .window_drag(false)
                .flex_row()
                .items_center()
                .gap(px(6.0))
                // 完成度计数
                .child(
                    text(move || format!("{}/8", item_sig.get().completed_count))
                        .text_size(px(12.0))
                        .text_color(white().alpha(0.50)),
                )
                // 操作按钮组
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        // [✓] 打卡按钮（到期时强化为翡翠绿微亮框）
                        .child(
                            view()
                                .w(px(28.0))
                                .h(px(24.0))
                                .rounded(px(5.0))
                                .bg(move || {
                                    if item_sig.get().is_expired() {
                                        hsl(145.0, 65.0, 35.0).alpha(0.65)
                                    } else {
                                        white().alpha(0.08)
                                    }
                                })
                                .border_1()
                                .border_color(move || {
                                    if item_sig.get().is_expired() {
                                        hsl(145.0, 75.0, 50.0).alpha(0.8)
                                    } else {
                                        white().alpha(0.12)
                                    }
                                })
                                .items_center()
                                .justify_center()
                                .active(|s| s.opacity(0.5))
                                .on_press(move || {
                                    items.update(|list| {
                                        if let Some(it) = list.iter_mut().find(|x| x.id == item_id) {
                                            it.complete();
                                        }
                                    });
                                })
                                .child(
                                    icons::icon_svg(icons::ICON_CHECK, 12.0, white().alpha(0.95)),
                                ),
                        )
                        // [+5] 延期 5 分钟按钮
                        .child(
                            view()
                                .w(px(28.0))
                                .h(px(24.0))
                                .rounded(px(5.0))
                                .bg(white().alpha(0.08))
                                .border_1()
                                .border_color(white().alpha(0.12))
                                .items_center()
                                .justify_center()
                                .active(|s| s.opacity(0.5))
                                .on_press(move || {
                                    items.update(|list| {
                                        if let Some(it) = list.iter_mut().find(|x| x.id == item_id) {
                                            it.snooze(300);
                                        }
                                    });
                                })
                                .child(
                                    text("+5")
                                        .text_size(px(10.5))
                                        .text_color(white().alpha(0.85)),
                                 ),
                        )
                        // [🗑] 移除该项提醒
                        .child(
                            view()
                                .w(px(22.0))
                                .h(px(24.0))
                                .rounded(px(5.0))
                                .items_center()
                                .justify_center()
                                .bg(white().alpha(0.04))
                                .active(|s| s.opacity(0.5))
                                .on_press(move || {
                                    items.update(|list| list.retain(|x| x.id != item_id));
                                })
                                .child(
                                    icons::icon_svg(icons::ICON_TRASH, 11.0, white().alpha(0.35)),
                                ),
                        ),
                ),
        )
}

/// 底部统计行：细线分隔 + 灰白小字通用统计 + 半透明时指示不透明度状态
fn render_footer(items: Signal<Vec<ReminderItem>>, opacity: Signal<f32>) -> impl IntoElement {
    view()
        .window_drag(true)
        .flex_row()
        .items_center()
        .justify_between()
        .pt(px(6.0))
        .border_t_1()
        .border_color(white().alpha(0.06))
        .child(
            text(move || {
                let list = items.get();
                let total_completed: u32 = list.iter().map(|i| i.completed_count).sum();
                let count = list.len();
                format!("进行中 {} 项 · 今日打卡 {} 次", count, total_completed)
            })
            .text_size(px(11.0))
            .text_color(white().alpha(0.45)),
        )
        .child(
            when(
                move || opacity.get() < 0.99,
                move || {
                    text(move || format!("不透明度 {:.0}%", opacity.get() * 100.0))
                        .text_size(px(10.5))
                        .text_color(hsl(45.0, 85.0, 60.0).alpha(0.80))
                },
            ),
        )
}

/// 渲染单个常用推荐预设卡片（精致卡片、无勾选、清晰时间标识、单例防重）
fn render_preset_card(
    preset: &'static PresetReminder,
    items: Signal<Vec<ReminderItem>>,
) -> impl IntoElement {
    let name = preset.name;
    let icon = preset.icon_key;
    let mins = preset.minutes;

    view()
        .flex_1()
        .px(px(7.0))
        .py(px(5.0))
        .rounded(px(5.0))
        .bg(move || {
            let running = items.get().iter().any(|x| x.name == name);
            if running {
                hsl(145.0, 50.0, 20.0).alpha(0.35)
            } else {
                white().alpha(0.04)
            }
        })
        .border_1()
        .border_color(move || {
            let running = items.get().iter().any(|x| x.name == name);
            if running {
                hsl(145.0, 60.0, 45.0).alpha(0.40)
            } else {
                white().alpha(0.07)
            }
        })
        .active(|s| s.opacity(0.6))
        .on_press(move || {
            let running = items.get().iter().any(|x| x.name == name);
            if !running {
                items.update(|list| {
                    list.push(ReminderItem::new(
                        name,
                        icon,
                        mins * 60,
                    ));
                });
            }
        })
        .flex_row()
        .items_center()
        .justify_between()
        // 左侧：对应矢量图标 + 预设名称（无任何对钩，干净对齐）
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(5.0))
                .child(
                    icons::icon_svg(
                        icons::get_icon_markup(icon),
                        12.0,
                        icons::get_item_hsla(icon),
                    ),
                )
                .child(
                    text(name)
                        .text_size(px(10.5))
                        .text_color(move || {
                            let running = items.get().iter().any(|x| x.name == name);
                            if running {
                                hsl(145.0, 75.0, 75.0)
                            } else {
                                white().alpha(0.85)
                            }
                        }),
                ),
        )
        // 右侧：时长徽章（如 30m / 45m，运行中呈现清新激活绿色）
        .child(
            view()
                .px(px(4.0))
                .py(px(1.0))
                .rounded(px(3.0))
                .bg(move || {
                    let running = items.get().iter().any(|x| x.name == name);
                    if running {
                        hsl(145.0, 60.0, 30.0).alpha(0.35)
                    } else {
                        white().alpha(0.06)
                    }
                })
                .child(
                    text(format!("{}m", mins))
                        .text_size(px(9.5))
                        .text_color(move || {
                            let running = items.get().iter().any(|x| x.name == name);
                            if running {
                                hsl(145.0, 70.0, 70.0)
                            } else {
                                white().alpha(0.40)
                            }
                        }),
                ),
        )
}

/// 设置与添加新提醒抽屉（点击齿轮展开，支持窗口不透明度调节、推荐快速开启与优雅折叠的自定义新提醒）
fn render_drawer(
    show: Signal<bool>,
    show_presets: Signal<bool>,
    show_custom_form: Signal<bool>,
    items: Signal<Vec<ReminderItem>>,
    opacity: Signal<f32>,
) -> impl IntoElement {
    let custom_name = signal(String::new());
    let custom_mins_str = signal("30".to_string());
    let custom_icon = signal("bell".to_string());
    let error_tip = signal(Option::<String>::None);

    when(
        move || show.get(),
        move || {
            view()
                .window_drag(false)
                .p(px(8.0))
                .mb(px(4.0))
                .rounded(px(8.0))
                .bg(white().alpha(0.06))
                .flex_col()
                .gap(px(6.0))
                // 1. 窗口不透明度调节区 (纯粹专注的 NSH 官方原生 slider 组件)
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        // 第一行：标题 + 当前数值百分比徽章 (100% = 完全不透明)
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(icons::icon_svg(
                                            icons::ICON_OPACITY,
                                            12.0,
                                            hsl(45.0, 85.0, 60.0),
                                        ))
                                        .child(
                                            text("窗口不透明度")
                                                .text_size(px(11.5))
                                                .text_color(white().alpha(0.85)),
                                        ),
                                )
                                .child(
                                    view()
                                        .px(px(6.0))
                                        .py(px(1.5))
                                        .rounded(px(4.0))
                                        .bg(white().alpha(0.06))
                                        .child(
                                            text(move || format!("{:.0}%", opacity.get() * 100.0))
                                                .text_size(px(11.0))
                                                .text_color(hsl(45.0, 90.0, 62.0)),
                                        ),
                                ),
                        )
                        // 第二行：NSH 官方原生 slider() 滑块组件，平滑无级拖拽与键盘方向键微调
                        .child(
                            view()
                                .w_full()
                                .px(px(2.0))
                                .child(
                                    slider()
                                        .min(20.0)
                                        .max(100.0)
                                        .step(1.0)
                                        .value(move || SliderValue::Single((opacity.get() * 100.0).round() as f64))
                                        .on_change(move |v: &SliderValue| {
                                            let pct = (v.start() / 100.0) as f32;
                                            opacity.set(pct.clamp(0.20, 1.0));
                                        })
                                        .accessibility_label("窗口不透明度")
                                        .track_style(|s| s.w_full().h(px(20.0)).justify_center())
                                        .indicator_style(|s| s.h(px(4.0)).rounded_full().bg(white().alpha(0.15)))
                                        .range_style(|s| s.rounded_full().bg(hsl(45.0, 90.0, 60.0)))
                                        .thumb_style(|s| {
                                            s.size(px(13.0))
                                                .rounded_full()
                                                .bg(white())
                                                .border_1()
                                                .border_color(hsl(45.0, 90.0, 60.0))
                                        }),
                                ),
                        ),
                )
                // 细分割线
                .child(
                    view()
                        .w_full()
                        .my(px(2.0))
                        .border_t_1()
                        .border_color(white().alpha(0.08)),
                )
                // 2. 基于 Collapsible 的常用推荐区域：支持平滑折叠，预设 6 项精炼对称双列卡片排版
                .child(
                    collapsible()
                        .open(show_presets)
                        .reveal(true)
                        // Trigger: 常用推荐折叠栏触发条
                        .child(
                            view()
                                .w_full()
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(5.0))
                                .bg(move || {
                                    if show_presets.get() {
                                        white().alpha(0.09)
                                    } else {
                                        white().alpha(0.04)
                                    }
                                })
                                .border_1()
                                .border_color(move || {
                                    if show_presets.get() {
                                        hsl(45.0, 90.0, 60.0).alpha(0.35)
                                    } else {
                                        white().alpha(0.06)
                                    }
                                })
                                .active(|s| s.opacity(0.6))
                                .on_press(move || show_presets.update(|v| *v = !*v))
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(5.0))
                                        .child(icons::icon_svg(
                                            icons::ICON_STAR,
                                            11.5,
                                            hsl(45.0, 90.0, 62.0),
                                        ))
                                        .child(
                                            text("常用推荐")
                                                .text_size(px(11.0))
                                                .text_color(hsl(45.0, 90.0, 65.0)),
                                        )
                                        .child(
                                            text("(6项预设)")
                                                .text_size(px(10.0))
                                                .text_color(white().alpha(0.40)),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(3.0))
                                        .child(
                                            text(move || if show_presets.get() { "收起" } else { "展开" })
                                                .text_size(px(10.5))
                                                .text_color(white().alpha(0.45)),
                                        )
                                        .child(
                                            icons::dynamic_icon_svg(
                                                move || {
                                                    if show_presets.get() {
                                                        icons::ICON_CHEVRON_UP
                                                    } else {
                                                        icons::ICON_CHEVRON_DOWN
                                                    }
                                                },
                                                11.0,
                                                white().alpha(0.45),
                                            ),
                                        ),
                                ),
                        )
                        // Content: 展开后的 6 项 2列x3行 对称精炼卡片面板
                        .content(
                            view()
                                .w_full()
                                .mt(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .bg(hsl(225.0, 10.0, 12.0))
                                .border_1()
                                .border_color(white().alpha(0.08))
                                .flex_col()
                                .gap(px(4.5))
                                .children(PRESETS.chunks(2).map(|pair| {
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .gap(px(4.5))
                                        .children(pair.iter().map(|preset| {
                                            render_preset_card(preset, items)
                                        }))
                                })),
                        ),
                )
                // 细分割线
                .child(
                    view()
                        .w_full()
                        .my(px(2.0))
                        .border_t_1()
                        .border_color(white().alpha(0.08)),
                )
                // 3. 基于 Collapsible 的自定义新提醒区域：默认收起，点击平滑展开完整配置面板
                .child(
                    collapsible()
                        .open(show_custom_form)
                        .reveal(true)
                        // Trigger: 折叠栏触发条
                        .child(
                            view()
                                .w_full()
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(5.0))
                                .bg(move || {
                                    if show_custom_form.get() {
                                        white().alpha(0.09)
                                    } else {
                                        white().alpha(0.04)
                                    }
                                })
                                .border_1()
                                .border_color(move || {
                                    if show_custom_form.get() {
                                        hsl(45.0, 90.0, 60.0).alpha(0.35)
                                    } else {
                                        white().alpha(0.06)
                                    }
                                })
                                .active(|s| s.opacity(0.6))
                                .on_press(move || show_custom_form.update(|v| *v = !*v))
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(5.0))
                                        .child(icons::icon_svg(
                                            icons::ICON_EDIT,
                                            11.5,
                                            hsl(45.0, 90.0, 62.0),
                                        ))
                                        .child(
                                            text("自定义新提醒")
                                                .text_size(px(11.0))
                                                .text_color(hsl(45.0, 90.0, 65.0)),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(3.0))
                                        .child(
                                            text(move || if show_custom_form.get() { "收起" } else { "配置" })
                                                .text_size(px(10.5))
                                                .text_color(white().alpha(0.45)),
                                        )
                                        .child(
                                            icons::dynamic_icon_svg(
                                                move || {
                                                    if show_custom_form.get() {
                                                        icons::ICON_CHEVRON_UP
                                                    } else {
                                                        icons::ICON_CHEVRON_DOWN
                                                    }
                                                },
                                                11.0,
                                                white().alpha(0.45),
                                            ),
                                        ),
                                ),
                        )
                        // Content: 展开后的专属精美配置面板
                        .content(
                            view()
                                .w_full()
                                .mt(px(4.0))
                                .p(px(8.0))
                                .rounded(px(6.0))
                                .bg(hsl(225.0, 10.0, 12.0))
                                .border_1()
                                .border_color(white().alpha(0.08))
                                .flex_col()
                                .gap(px(6.0))
                                // Field 1: 提醒名称
                                .child(
                                    view()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    text("提醒事项名称")
                                                        .text_size(px(10.0))
                                                        .text_color(white().alpha(0.45)),
                                                )
                                                .child(
                                                    text(move || error_tip.get().unwrap_or_default())
                                                        .text_size(px(10.0))
                                                        .text_color(hsl(0.0, 85.0, 70.0)),
                                                ),
                                        )
                                        .child(
                                            input()
                                                .placeholder("如：站姿切换、滴眼药水、工位通风")
                                                .value(move || custom_name.get())
                                                .on_change(move |s: &str| {
                                                    custom_name.set(s.to_string());
                                                    error_tip.set(None);
                                                })
                                                .w_full()
                                                .h(px(25.0))
                                                .px(px(8.0))
                                                .rounded(px(4.0))
                                                .bg(white().alpha(0.06))
                                                .border_1()
                                                .border_color(white().alpha(0.12))
                                                .text_size(px(11.0))
                                                .text_color(white().alpha(0.95))
                                                .placeholder_color(white().alpha(0.30)),
                                        ),
                                )
                                // Field 2: 循环周期（键盘直接输入 + 1分钟微调 + 快捷填入）
                                .child(
                                    view()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    text("循环周期")
                                                        .text_size(px(10.0))
                                                        .text_color(white().alpha(0.45)),
                                                )
                                                .child(
                                                    view()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(3.0))
                                                        // 减 1 分钟
                                                        .child(
                                                            view()
                                                                .w(px(18.0))
                                                                .h(px(19.0))
                                                                .rounded(px(3.0))
                                                                .bg(white().alpha(0.08))
                                                                .items_center()
                                                                .justify_center()
                                                                .active(|s| s.opacity(0.5))
                                                                .on_press(move || {
                                                                    let cur = custom_mins_str.get().parse::<u32>().unwrap_or(30);
                                                                    let next = cur.saturating_sub(1).max(1);
                                                                    custom_mins_str.set(next.to_string());
                                                                    error_tip.set(None);
                                                                })
                                                                .child(text("-").text_size(px(11.0)).text_color(white().alpha(0.70))),
                                                        )
                                                        // 数字文本输入框：支持点击键盘任意输入
                                                        .child(
                                                            input()
                                                                .placeholder("30")
                                                                .value(move || custom_mins_str.get())
                                                                .on_change(move |s: &str| {
                                                                    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).take(4).collect();
                                                                    custom_mins_str.set(digits);
                                                                    error_tip.set(None);
                                                                })
                                                                .w(px(38.0))
                                                                .h(px(19.0))
                                                                .px(px(4.0))
                                                                .rounded(px(3.0))
                                                                .bg(white().alpha(0.08))
                                                                .border_1()
                                                                .border_color(hsl(45.0, 90.0, 60.0).alpha(0.40))
                                                                .text_size(px(11.0))
                                                                .text_color(hsl(45.0, 90.0, 65.0)),
                                                        )
                                                        // 加 1 分钟
                                                        .child(
                                                            view()
                                                                .w(px(18.0))
                                                                .h(px(19.0))
                                                                .rounded(px(3.0))
                                                                .bg(white().alpha(0.08))
                                                                .items_center()
                                                                .justify_center()
                                                                .active(|s| s.opacity(0.5))
                                                                .on_press(move || {
                                                                    let cur = custom_mins_str.get().parse::<u32>().unwrap_or(30);
                                                                    let next = (cur + 1).min(1440);
                                                                    custom_mins_str.set(next.to_string());
                                                                    error_tip.set(None);
                                                                })
                                                                .child(text("+").text_size(px(11.0)).text_color(white().alpha(0.70))),
                                                        )
                                                        .child(
                                                            text("分钟")
                                                                .text_size(px(10.0))
                                                                .text_color(white().alpha(0.50)),
                                                        ),
                                                ),
                                        )
                                        // 快捷填入颗粒度胶囊
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .gap(px(3.0))
                                                .child(
                                                    text("快捷填入:")
                                                        .text_size(px(9.5))
                                                        .text_color(white().alpha(0.35)),
                                                )
                                                .child(
                                                    view()
                                                        .flex_row()
                                                        .gap(px(3.0))
                                                        .children([15, 25, 30, 45, 60, 90].iter().map(|&mins| {
                                                            view()
                                                                .px(px(5.0))
                                                                .py(px(1.5))
                                                                .rounded(px(3.0))
                                                                .bg(move || {
                                                                    if custom_mins_str.get() == mins.to_string() {
                                                                        hsl(45.0, 90.0, 60.0).alpha(0.25)
                                                                    } else {
                                                                        white().alpha(0.05)
                                                                    }
                                                                })
                                                                .border_1()
                                                                .border_color(move || {
                                                                    if custom_mins_str.get() == mins.to_string() {
                                                                        hsl(45.0, 90.0, 60.0).alpha(0.60)
                                                                    } else {
                                                                        white().alpha(0.04)
                                                                    }
                                                                })
                                                                .active(|s| s.opacity(0.6))
                                                                .on_press(move || {
                                                                    custom_mins_str.set(mins.to_string());
                                                                    error_tip.set(None);
                                                                })
                                                                .child(
                                                                    text(format!("{}m", mins))
                                                                        .text_size(px(9.5))
                                                                        .text_color(move || {
                                                                            if custom_mins_str.get() == mins.to_string() {
                                                                                hsl(45.0, 90.0, 65.0)
                                                                            } else {
                                                                                white().alpha(0.50)
                                                                            }
                                                                        }),
                                                                )
                                                        })),
                                                ),
                                        ),
                                )
                                // Field 3: 图标分类与高光色（16 种丰富主题，两排 8 列精致排布）
                                .child(
                                    view()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            text("图标主题")
                                                .text_size(px(10.0))
                                                .text_color(white().alpha(0.45)),
                                        )
                                        .child(
                                            view()
                                                .flex_col()
                                                .gap(px(3.0))
                                                // 第一排：生活与健康 (喝水、咖啡、活动、拉伸、护眼、服药、静心、暖阳)
                                                .child(
                                                    view()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        .children([
                                                            "droplet", "coffee", "walk", "activity",
                                                            "eye", "pill", "heart", "sun",
                                                        ].iter().map(|&ic| {
                                                            let item_hsla = icons::get_item_hsla(ic);
                                                            view()
                                                                .w(px(24.0))
                                                                .h(px(24.0))
                                                                .rounded(px(4.0))
                                                                .bg(move || {
                                                                    if custom_icon.get() == ic {
                                                                        item_hsla.alpha(0.20)
                                                                    } else {
                                                                        white().alpha(0.04)
                                                                    }
                                                                })
                                                                .border_1()
                                                                .border_color(move || {
                                                                    if custom_icon.get() == ic {
                                                                        item_hsla.alpha(0.80)
                                                                    } else {
                                                                        white().alpha(0.06)
                                                                    }
                                                                })
                                                                .items_center()
                                                                .justify_center()
                                                                .active(|s| s.opacity(0.5))
                                                                .on_press(move || custom_icon.set(ic.to_string()))
                                                                .child(
                                                                    icons::icon_svg(
                                                                        icons::get_icon_markup(ic),
                                                                        12.0,
                                                                        move || {
                                                                            if custom_icon.get() == ic {
                                                                                item_hsla
                                                                            } else {
                                                                                white().alpha(0.40)
                                                                            }
                                                                        },
                                                                    ),
                                                                )
                                                        })),
                                                )
                                                // 第二排：专注与日程 (番茄、阅读、办公、目标、小憩、轻食、排便、闹钟)
                                                .child(
                                                    view()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        .children([
                                                            "clock", "book", "laptop", "target",
                                                            "moon", "apple", "toilet", "bell",
                                                        ].iter().map(|&ic| {
                                                            let item_hsla = icons::get_item_hsla(ic);
                                                            view()
                                                                .w(px(24.0))
                                                                .h(px(24.0))
                                                                .rounded(px(4.0))
                                                                .bg(move || {
                                                                    if custom_icon.get() == ic {
                                                                        item_hsla.alpha(0.20)
                                                                    } else {
                                                                        white().alpha(0.04)
                                                                    }
                                                                })
                                                                .border_1()
                                                                .border_color(move || {
                                                                    if custom_icon.get() == ic {
                                                                        item_hsla.alpha(0.80)
                                                                    } else {
                                                                        white().alpha(0.06)
                                                                    }
                                                                })
                                                                .items_center()
                                                                .justify_center()
                                                                .active(|s| s.opacity(0.5))
                                                                .on_press(move || custom_icon.set(ic.to_string()))
                                                                .child(
                                                                    icons::icon_svg(
                                                                        icons::get_icon_markup(ic),
                                                                        12.0,
                                                                        move || {
                                                                            if custom_icon.get() == ic {
                                                                                item_hsla
                                                                            } else {
                                                                                white().alpha(0.40)
                                                                            }
                                                                        },
                                                                    ),
                                                                )
                                                        })),
                                                ),
                                        ),
                                )
                                // Field 4: 底部操作栏
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .pt(px(2.0))
                                        .child(
                                            view()
                                                .px(px(8.0))
                                                .h(px(22.0))
                                                .rounded(px(4.0))
                                                .bg(white().alpha(0.06))
                                                .items_center()
                                                .justify_center()
                                                .active(|s| s.opacity(0.5))
                                                .on_press(move || {
                                                    show_custom_form.set(false);
                                                    error_tip.set(None);
                                                })
                                                .child(
                                                    text("收起")
                                                        .text_size(px(10.5))
                                                        .text_color(white().alpha(0.50)),
                                                ),
                                        )
                                        .child(
                                            view()
                                                .px(px(12.0))
                                                .h(px(22.0))
                                                .rounded(px(4.0))
                                                .bg(hsl(45.0, 90.0, 56.0))
                                                .items_center()
                                                .justify_center()
                                                .active(|s| s.opacity(0.6))
                                                .on_press(move || {
                                                    let name = custom_name.get().trim().to_string();
                                                    if name.is_empty() {
                                                        error_tip.set(Some("请输入名称".to_string()));
                                                        return;
                                                    }
                                                    let mins = match custom_mins_str.get().trim().parse::<u32>() {
                                                        Ok(m) if m > 0 => m,
                                                        _ => {
                                                            error_tip.set(Some("请输入有效分钟数(≥1)".to_string()));
                                                            return;
                                                        }
                                                    };
                                                    if mins > 1440 {
                                                        error_tip.set(Some("时长上限为 1440 分钟 (24小时)".to_string()));
                                                        return;
                                                    }
                                                    if items.get().iter().any(|x| x.name == name) {
                                                        error_tip.set(Some("该提醒已在列表中".to_string()));
                                                        return;
                                                    }
                                                    items.update(|list| {
                                                        list.push(ReminderItem::new(
                                                            &name,
                                                            custom_icon.get(),
                                                            mins * 60,
                                                        ));
                                                    });
                                                    custom_name.set(String::new());
                                                    error_tip.set(None);
                                                    show_custom_form.set(false);
                                                })
                                                .child(
                                                    text("开启提醒")
                                                        .text_size(px(11.0))
                                                        .text_color(hsl(225.0, 25.0, 10.0)),
                                                ),
                                        ),
                                ),
                        ),
                )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_height_calculation() {
        // 0 项且未展开抽屉
        let h0 = calculate_window_height(0, false, false, false);
        assert_eq!(h0, 195.0); // 102.0 clamp 最小 195.0

        // 3 项未展开抽屉
        let h3 = calculate_window_height(3, false, false, false);
        assert_eq!(h3, BASE_CONTENT_HEIGHT + 3.0 * ITEM_ROW_HEIGHT);

        // 3 项展开基础抽屉（未展开常用推荐与自定义表单）
        let h3_drawer = calculate_window_height(3, true, false, false);
        assert_eq!(h3_drawer, BASE_CONTENT_HEIGHT + 3.0 * ITEM_ROW_HEIGHT + DRAWER_BASE_HEIGHT);
        assert!(h3_drawer > h3);

        // 3 项展开抽屉且展开常用推荐（未展开自定义表单）
        let h3_presets = calculate_window_height(3, true, true, false);
        assert_eq!(h3_presets, BASE_CONTENT_HEIGHT + 3.0 * ITEM_ROW_HEIGHT + DRAWER_BASE_HEIGHT + DRAWER_PRESETS_HEIGHT);
        assert!(h3_presets > h3_drawer);

        // 3 项展开抽屉且展开自定义表单（未展开常用推荐）
        let h3_custom = calculate_window_height(3, true, false, true);
        assert_eq!(h3_custom, BASE_CONTENT_HEIGHT + 3.0 * ITEM_ROW_HEIGHT + DRAWER_BASE_HEIGHT + DRAWER_CUSTOM_HEIGHT);
        assert!(h3_custom > h3_presets);

        // 3 项展开抽屉且两者均展开
        let h3_both = calculate_window_height(3, true, true, true);
        assert_eq!(h3_both, BASE_CONTENT_HEIGHT + 3.0 * ITEM_ROW_HEIGHT + DRAWER_BASE_HEIGHT + DRAWER_PRESETS_HEIGHT + DRAWER_CUSTOM_HEIGHT);
        assert!(h3_both > h3_custom);

        // 超多项 clamp 最大值 720.0
        let h_many = calculate_window_height(20, true, true, true);
        assert_eq!(h_many, 720.0);
    }

    #[test]
    fn test_opacity_clamp_range() {
        let clamp_op = |v: f32| v.clamp(0.20, 1.0);
        assert_eq!(clamp_op(0.0), 0.20);
        assert_eq!(clamp_op(0.15), 0.20);
        assert_eq!(clamp_op(0.85), 0.85);
        assert_eq!(clamp_op(1.0), 1.0);
        assert_eq!(clamp_op(1.5), 1.0);
    }

    #[test]
    fn test_apply_window_opacity() {
        apply_window_opacity(1.0);
        apply_window_opacity(0.85);
        apply_window_opacity(0.50);
        apply_window_opacity(0.10); // 会自动 clamp 到 0.20
        apply_window_opacity(1.50); // 会自动 clamp 到 1.00
    }

    #[test]
    fn test_input_element() {
        use nsh_core::element::input;
        let _ = input().placeholder("测试输入");
    }
}
