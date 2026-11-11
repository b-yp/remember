//! 矢量图标模块：基于 Tabler Icons 规范的纯净 SVG 图标组件与发光圆环进度组件
//! SVG 内部采用 stroke="currentColor" 或指定色彩，完全摆脱 Emoji，呈现高质感专业桌面挂件

use nsh_core::prelude::*;
use nsh_core::element::{AdaptiveColor, ImageSrc};

/// 渲染一个带有尺寸和指定颜色的静态矢量图标
pub fn icon_svg<C>(markup: &'static str, size: f32, color_val: C) -> impl IntoElement
where
    C: IntoReactive<AdaptiveColor>,
{
    view()
        .items_center()
        .justify_center()
        .text_color(color_val)
        .child(
            svg(ImageSrc::svg_markup(markup))
                .w(px(size))
                .h(px(size)),
        )
}

/// 渲染一个根据闭包响应式切换 markup 的动态矢量图标
pub fn dynamic_icon_svg<F, C>(markup_fn: F, size: f32, color_val: C) -> impl IntoElement
where
    F: Fn() -> &'static str + 'static,
    C: IntoReactive<AdaptiveColor> + 'static,
{
    view()
        .items_center()
        .justify_center()
        .text_color(color_val)
        .child(
            svg(move || ImageSrc::svg_markup(markup_fn()))
                .w(px(size))
                .h(px(size)),
        )
}

/// 生成带有彩色圆环进度条和居中矢量图标的完整 SVG markup
/// 外层背景环轨采用 stroke="currentColor" stroke-opacity="0.12"，原生自适应任何明暗主题
pub fn ring_progress_svg(icon_key: &str, progress: f32, color_hex: &str) -> String {
    let circumference = 87.964_f32; // 2 * PI * 14
    let progress = progress.clamp(0.01, 1.0);
    let offset = circumference * (1.0 - progress);

    let inner_paths = match icon_key {
        "droplet" | "water" => r#"<path d="M6.8 11a6 6 0 1 0 10.396 0l-5.196 -8l-5.2 8z" /><path d="M6 14a6 6 0 0 0 6 6" />"#,
        "coffee" | "tea" => r#"<path d="M3 14c0 2.76 2.24 5 5 5h4c2.76 0 5 -2.24 5 -5v-3H3v3z" /><path d="M17 9h1a3 3 0 0 1 3 3v1a3 3 0 0 1 -3 3h-1" /><path d="M6 2v2" /><path d="M10 2v2" /><path d="M14 2v2" />"#,
        "walk" | "step" => r#"<path d="M13 4a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" /><path d="M7 21l3 -4" /><path d="M16 21l-2 -4l-3 -3l1 -6" /><path d="M6 12l2 -3l4 -1l3 3l3 1" />"#,
        "activity" | "stretch" => r#"<path d="M3 12h4l3 8l4 -16l3 8h4" />"#,
        "eye" | "vision" => r#"<path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /><path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6" />"#,
        "pill" | "medicine" => r#"<path d="M4.5 12.5l8 -8a4.95 4.95 0 0 1 7 7l-8 8a4.95 4.95 0 0 1 -7 -7" /><path d="M8.5 8.5l7 7" />"#,
        "heart" | "breathe" => r#"<path d="M19.5 12.572l-7.5 7.428l-7.5 -7.428a5 5 0 1 1 7.5 -6.566a5 5 0 1 1 7.5 6.572" />"#,
        "sun" | "light" => r#"<path d="M12 12m-4 0a4 4 0 1 0 8 0a4 4 0 1 0 -8 0" /><path d="M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7" />"#,
        "clock" | "focus" => r#"<path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" />"#,
        "book" | "read" => r#"<path d="M3 19a9 9 0 0 1 9 0a9 9 0 0 1 9 0" /><path d="M3 6a9 9 0 0 1 9 0a9 9 0 0 1 9 0" /><path d="M3 6l0 13" /><path d="M12 6l0 13" /><path d="M21 6l0 13" />"#,
        "laptop" | "work" => r#"<path d="M3 19l18 0" /><path d="M5 6m0 1a1 1 0 0 1 1 -1h12a1 1 0 0 1 1 1v8a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1z" />"#,
        "target" | "goal" => r#"<path d="M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /><path d="M12 12m-5 0a5 5 0 1 0 10 0a5 5 0 1 0 -10 0" /><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" />"#,
        "moon" | "sleep" | "rest" => r#"<path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" />"#,
        "apple" | "fruit" | "food" => r#"<path d="M12 14m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /><path d="M12 7a3 3 0 0 1 3 -3" /><path d="M12 10v4" />"#,
        "toilet" | "poop" => r#"<path d="M10 12h4v7a3 3 0 0 1 -3 3h-2a3 3 0 0 1 -3 -3v-7h4z" /><path d="M6 4h12a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1v-3a1 1 0 0 1 1 -1z" /><path d="M10 9v3" /><path d="M14 9v3" />"#,
        "bell" | "alarm" => r#"<path d="M10 5a2 2 0 0 1 4 0a7 7 0 0 1 4 6v3a4 4 0 0 0 2 3h-16a4 4 0 0 0 2 -3v-3a7 7 0 0 1 4 -6" /><path d="M9 17v1a3 3 0 0 0 6 0v-1" />"#,
        _ => r#"<path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" />"#,
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><circle cx="18" cy="18" r="14" fill="none" stroke="currentColor" stroke-opacity="0.12" stroke-width="2.5" /><circle cx="18" cy="18" r="14" fill="none" stroke="{color_hex}" stroke-width="2.5" stroke-dasharray="{circumference:.2}" stroke-dashoffset="{offset:.2}" stroke-linecap="round" transform="rotate(-90 18 18)" /><g transform="translate(10, 10) scale(0.666)" stroke="{color_hex}" fill="none" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">{inner_paths}</g></svg>"#
    )
}

// ==================== 16 种丰富业务提醒矢量图标 ====================

/// 水滴（喝水 / 补水）
pub const ICON_DROPLET: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6.8 11a6 6 0 1 0 10.396 0l-5.196 -8l-5.2 8z" /><path d="M6 14a6 6 0 0 0 6 6" /></svg>"#;

/// 咖啡杯（咖啡 / 茶饮）
pub const ICON_COFFEE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 14c0 2.76 2.24 5 5 5h4c2.76 0 5 -2.24 5 -5v-3H3v3z" /><path d="M17 9h1a3 3 0 0 1 3 3v1a3 3 0 0 1 -3 3h-1" /><path d="M6 2v2" /><path d="M10 2v2" /><path d="M14 2v2" /></svg>"#;

/// 走动小人（起身活动 / 散步）
pub const ICON_WALK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 4a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" /><path d="M7 21l3 -4" /><path d="M16 21l-2 -4l-3 -3l1 -6" /><path d="M6 12l2 -3l4 -1l3 3l3 1" /></svg>"#;

/// 脉冲波形（拉伸运动 / 健身活动）
pub const ICON_ACTIVITY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h4l3 8l4 -16l3 8h4" /></svg>"#;

/// 眼睛（护眼远眺）
pub const ICON_EYE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /><path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6" /></svg>"#;

/// 药丸（服药补充 / 维生素）
pub const ICON_PILL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 12.5l8 -8a4.95 4.95 0 0 1 7 7l-8 8a4.95 4.95 0 0 1 -7 -7" /><path d="M8.5 8.5l7 7" /></svg>"#;

/// 爱心（深呼吸 / 冥想静心 / 心率）
pub const ICON_HEART: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19.5 12.572l-7.5 7.428l-7.5 -7.428a5 5 0 1 1 7.5 -6.566a5 5 0 1 1 7.5 6.572" /></svg>"#;

/// 太阳（晒太阳 / 户外通风）
pub const ICON_SUN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 12m-4 0a4 4 0 1 0 8 0a4 4 0 1 0 -8 0" /><path d="M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7" /></svg>"#;

/// 时钟（番茄专注 / 工作定时）
pub const ICON_CLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" /></svg>"#;

/// 书本（阅读学习 / 知识积累）
pub const ICON_BOOK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 19a9 9 0 0 1 9 0a9 9 0 0 1 9 0" /><path d="M3 6a9 9 0 0 1 9 0a9 9 0 0 1 9 0" /><path d="M3 6l0 13" /><path d="M12 6l0 13" /><path d="M21 6l0 13" /></svg>"#;

/// 笔记本电脑（久坐提示 / 工作整理）
pub const ICON_LAPTOP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 19l18 0" /><path d="M5 6m0 1a1 1 0 0 1 1 -1h12a1 1 0 0 1 1 1v8a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1z" /></svg>"#;

/// 靶心目标（阶段打卡 / 目标达成）
pub const ICON_TARGET: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /><path d="M12 12m-5 0a5 5 0 1 0 10 0a5 5 0 1 0 -10 0" /><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /></svg>"#;

/// 月亮（小憩休息 / 准时就寝）
pub const ICON_MOON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" /></svg>"#;

/// 苹果（水果加餐 / 健康轻食）
pub const ICON_APPLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 14m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /><path d="M12 7a3 3 0 0 1 3 -3" /><path d="M12 10v4" /></svg>"#;

/// 卫生间 / 便便马桶（规律排便 / 带薪拉屎）
pub const ICON_TOILET: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 12h4v7a3 3 0 0 1 -3 3h-2a3 3 0 0 1 -3 -3v-7h4z" /><path d="M6 4h12a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1v-3a1 1 0 0 1 1 -1z" /><path d="M10 9v3" /><path d="M14 9v3" /></svg>"#;

/// 铃铛 / 提醒（通用提醒 / 闹钟）
pub const ICON_BELL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 5a2 2 0 0 1 4 0a7 7 0 0 1 4 6v3a4 4 0 0 0 2 3h-16a4 4 0 0 0 2 -3v-3a7 7 0 0 1 4 -6" /><path d="M9 17v1a3 3 0 0 0 6 0v-1" /></svg>"#;

// ==================== 操作界面图标 ====================

/// 垃圾桶 / 删除（Trash）
pub const ICON_TRASH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7l16 0" /><path d="M10 11l0 6" /><path d="M14 11l0 6" /><path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" /><path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" /></svg>"#;

/// 警示 / 到期（Alert）纯矢量图标
pub const ICON_ALERT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 9v4" /><path d="M12 17h.01" /><path d="M5 19h14a2 2 0 0 0 1.84 -2.75l-7.1 -12.25a2 2 0 0 0 -3.5 0l-7.1 12.25a2 2 0 0 0 1.75 2.75" /></svg>"#;

/// 打卡完成对勾（Check）
pub const ICON_CHECK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12l5 5l10 -10" /></svg>"#;

/// 暂停（Pause）
pub const ICON_PAUSE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M7 5h2v14H7zM15 5h2v14h-2z" /></svg>"#;

/// 播放/恢复（Play）
pub const ICON_PLAY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 4v16l13 -8z" /></svg>"#;

/// 加号（Plus）
pub const ICON_PLUS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5l0 14" /><path d="M5 12l14 0" /></svg>"#;

/// 关闭叉叉（X）
pub const ICON_X: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6l-12 12" /><path d="M6 6l12 12" /></svg>"#;

/// 设置（Settings）
pub const ICON_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065z" /><path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" /></svg>"#;

/// 窗口透明度（Opacity）
pub const ICON_OPACITY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor" /></svg>"#;

/// 折叠收起指示箭头（Chevron Up）
pub const ICON_CHEVRON_UP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 15l6 -6l6 6" /></svg>"#;

/// 展开指示箭头（Chevron Down）
pub const ICON_CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6l6 -6" /></svg>"#;

/// 自定义配置 / 铅笔编辑（Edit / Custom）
pub const ICON_EDIT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4" /><path d="M13.5 6.5l4 4" /></svg>"#;

/// 常用推荐 / 星标推荐（Star / Recommendation）
pub const ICON_STAR: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" /></svg>"#;


/// 根据标识获取对应 SVG
pub fn get_icon_markup(icon_key: &str) -> &'static str {
    match icon_key {
        "droplet" | "water" => ICON_DROPLET,
        "coffee" | "tea" => ICON_COFFEE,
        "walk" | "step" => ICON_WALK,
        "activity" | "stretch" => ICON_ACTIVITY,
        "eye" | "vision" => ICON_EYE,
        "pill" | "medicine" => ICON_PILL,
        "heart" | "breathe" => ICON_HEART,
        "sun" | "light" => ICON_SUN,
        "clock" | "focus" => ICON_CLOCK,
        "book" | "read" => ICON_BOOK,
        "laptop" | "work" => ICON_LAPTOP,
        "target" | "goal" => ICON_TARGET,
        "moon" | "sleep" | "rest" => ICON_MOON,
        "apple" | "fruit" | "food" => ICON_APPLE,
        "toilet" | "poop" => ICON_TOILET,
        "bell" | "alarm" => ICON_BELL,
        "star" | "recommend" => ICON_STAR,
        _ => ICON_BELL,
    }
}

/// 获取条目的主题高光色（用于环形进度 SVG 的 hex 字符串）
pub fn get_item_color(icon_key: &str) -> &'static str {
    match icon_key {
        "droplet" | "water" => "#38bdf8",         // 蓝（喝水）
        "coffee" | "tea" => "#f59e0b",            // 琥珀黄（咖啡）
        "walk" | "step" => "#34d399",             // 薄荷绿（活动）
        "activity" | "stretch" => "#a3e635",      // 青柠绿（拉伸）
        "eye" | "vision" => "#818cf8",            // 紫罗兰（护眼）
        "pill" | "medicine" => "#f472b6",         // 珊瑚粉（服药）
        "heart" | "breathe" => "#fb7185",         // 玫瑰红（静心）
        "sun" | "light" => "#fb923c",             // 暖阳橙（通风）
        "clock" | "focus" => "#a78bfa",           // 专注紫（番茄）
        "book" | "read" => "#22d3ee",             // 清爽青（阅读）
        "laptop" | "work" => "#60a5fa",           // 科技蓝（办公）
        "target" | "goal" => "#f87171",           // 焦点红（目标）
        "moon" | "sleep" | "rest" => "#c084fc",   // 夜紫（休息）
        "apple" | "fruit" | "food" => "#4ade80",  // 嫩绿（轻食）
        "toilet" | "poop" => "#fbbf24",           // 琥珀（便便）
        "bell" | "alarm" => "#eab308",            // 金黄（通用）
        "star" | "recommend" => "#f59e0b",        // 闪耀星标
        _ => "#38bdf8",
    }
}

/// 获取条目的主题自适应色彩（AdaptiveColor，明暗主题下均保持最佳对比度）
pub fn get_item_adaptive(icon_key: &str) -> AdaptiveColor {
    match icon_key {
        "droplet" | "water" => AdaptiveColor::rgb(0x0284c7, 0x38bdf8),
        "coffee" | "tea" => AdaptiveColor::rgb(0xd97706, 0xf59e0b),
        "walk" | "step" => AdaptiveColor::rgb(0x059669, 0x34d399),
        "activity" | "stretch" => AdaptiveColor::rgb(0x65a30d, 0xa3e635),
        "eye" | "vision" => AdaptiveColor::rgb(0x4f46e5, 0x818cf8),
        "pill" | "medicine" => AdaptiveColor::rgb(0xdb2777, 0xf472b6),
        "heart" | "breathe" => AdaptiveColor::rgb(0xe11d48, 0xfb7185),
        "sun" | "light" => AdaptiveColor::rgb(0xea580c, 0xfb923c),
        "clock" | "focus" => AdaptiveColor::rgb(0x7c3aed, 0xa78bfa),
        "book" | "read" => AdaptiveColor::rgb(0x0891b2, 0x22d3ee),
        "laptop" | "work" => AdaptiveColor::rgb(0x2563eb, 0x60a5fa),
        "target" | "goal" => AdaptiveColor::rgb(0xdc2626, 0xf87171),
        "moon" | "sleep" | "rest" => AdaptiveColor::rgb(0x9333ea, 0xc084fc),
        "apple" | "fruit" | "food" => AdaptiveColor::rgb(0x16a34a, 0x4ade80),
        "toilet" | "poop" => AdaptiveColor::rgb(0xd97706, 0xfbbf24),
        "bell" | "alarm" => AdaptiveColor::rgb(0xca8a04, 0xeab308),
        "star" | "recommend" => AdaptiveColor::rgb(0xd97706, 0xf59e0b),
        _ => AdaptiveColor::rgb(0x0284c7, 0x38bdf8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_progress_svg() {
        let svg_str = ring_progress_svg("droplet", 0.5, "#38bdf8");
        assert!(svg_str.contains("stroke=\"currentColor\""));
        assert!(svg_str.contains("#38bdf8"));
    }

    #[test]
    fn test_get_item_color() {
        assert_eq!(get_item_color("droplet"), "#38bdf8");
    }
}
