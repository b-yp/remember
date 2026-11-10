//! The app's Rust UI: reminder 桌面通用健康与生活提醒悬浮挂件
//! 纯 Rust 编写、固定深色质感、使用 NSH 官方原生 API 隐藏标题栏、支持随意拖拽、矢量环形进度图标

pub mod icons;
pub mod model;

use std::time::Duration;
use nsh_core::prelude::*;
use nsh_core::element::desktop;
use nsh_core::desktop::{WindowFrame, WindowLevel};
use model::ReminderItem;

#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

/// Windows 挂件专用：配置为纯净无边框悬浮卡片，移除全部系统按钮与非客户区，保留 Win11 硬件圆角与阴影
#[cfg(target_os = "windows")]
fn setup_floating_window() {
    std::thread::spawn(|| {
        use platform::windows::Win32::Foundation::{HWND, LPARAM};
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
                    // 1. 设置为纯净无边框悬浮挂件卡片样式 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS)
                    //    彻底剥离 WS_CAPTION、WS_THICKFRAME 以及系统三键，不留任何非客户区边框
                    let popup_style = (WS_POPUP.0 | WS_VISIBLE.0 | WS_CLIPSIBLINGS.0) as i32;
                    SetWindowLongW(target_hwnd, GWL_STYLE, popup_style);

                    // 2. 启用 Win11 原生硬件级平滑圆角 (DWMWCP_ROUND = 2)
                    let round_pref: u32 = 2;
                    let _ = DwmSetWindowAttribute(
                        target_hwnd,
                        DWMWA_WINDOW_CORNER_PREFERENCE,
                        &round_pref as *const _ as *const _,
                        std::mem::size_of_val(&round_pref) as u32,
                    );

                    // 3. 启用系统原生精致阴影 (CS_DROPSHADOW)
                    let class_style = GetClassLongPtrW(target_hwnd, GCL_STYLE);
                    SetClassLongPtrW(target_hwnd, GCL_STYLE, (class_style | CS_DROPSHADOW.0 as usize) as isize);

                    // 4. 刷新非客户区，不改变位置、不改变尺寸、不改变Z序、不抢焦点
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
pub const DRAWER_HEIGHT: f32 = 120.0;

/// 根据提醒项目数量和设置抽屉展开状态精确计算窗口高度
pub fn calculate_window_height(item_count: usize, is_drawer_open: bool) -> f32 {
    let items_h = item_count as f32 * ITEM_ROW_HEIGHT;
    let drawer_h = if is_drawer_open { DRAWER_HEIGHT } else { 0.0 };
    (BASE_CONTENT_HEIGHT + items_h + drawer_h).clamp(195.0, 520.0)
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
    PresetReminder { name: "补充用药", icon_key: "pill", minutes: 240 },
    PresetReminder { name: "站立拉伸", icon_key: "bell", minutes: 60 },
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

    // 全局暂停状态与设置面板展开状态
    let is_all_paused = signal(false);
    let show_settings = signal(false);

    // 窗口高度动态自适应：点击设置抽屉或增减提醒项时动态调节高度，并支持平滑滚动，杜绝遮挡
    effect(move || {
        let is_open = show_settings.get();
        let count = items.get().len();
        let target_h = calculate_window_height(count, is_open);

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
        // 2. 新建/设置抽屉（点击顶部设置/新建时平滑展开，窗口自动变大，不挤压内容）
        .child(render_drawer(show_settings, items))
        // 3. 核心提醒列表（基于 each 响应式驱动更新与增减）
        .child(render_items(items))
        // 4. 底部状态行（通用统计：「进行中 X 项 · 今日打卡 Y 次」）
        .child(render_footer(items))
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
                // 标题与倒计时（到期时红字警示，纯净文本与矢量图标，零 Emoji）
                .child(
                    view()
                        .flex_col()
                        .justify_center()
                        .gap(px(2.0))
                        .child(
                            text(move || item_sig.get().name.clone())
                                .text_size(px(13.0))
                                .text_color(white().alpha(0.92)),
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

/// 底部统计行：细线分隔 + 灰白小字通用统计
fn render_footer(items: Signal<Vec<ReminderItem>>) -> impl IntoElement {
    view()
        .window_drag(true)
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
}

/// 设置与添加新提醒抽屉（点击齿轮展开）
fn render_drawer(
    show: Signal<bool>,
    items: Signal<Vec<ReminderItem>>,
) -> impl IntoElement {
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
                .child(
                    text("点击快捷添加通用提醒：")
                        .text_size(px(11.0))
                        .text_color(white().alpha(0.5)),
                )
                .child(
                    view()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(4.0))
                        .children(PRESETS.iter().map(|preset| {
                            let preset_name = preset.name;
                            let preset_icon = preset.icon_key;
                            let preset_mins = preset.minutes;

                            view()
                                .px(px(6.0))
                                .py(px(3.0))
                                .rounded(px(5.0))
                                .bg(white().alpha(0.08))
                                .active(|s| s.opacity(0.5))
                                .on_press(move || {
                                    items.update(|list| {
                                        list.push(ReminderItem::new(
                                            preset_name,
                                            preset_icon,
                                            preset_mins * 60,
                                        ));
                                    });
                                })
                                .child(
                                    text(format!("+ {} ({}m)", preset_name, preset_mins))
                                        .text_size(px(11.0))
                                        .text_color(white().alpha(0.85)),
                                )
                        })),
                )
        },
    )
}
