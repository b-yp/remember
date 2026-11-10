//! 矢量图标模块：基于 Tabler Icons 规范的纯净 SVG 图标组件与发光圆环进度组件
//! SVG 内部采用 stroke="currentColor" 或指定色彩，完全摆脱 Emoji，呈现高质感专业桌面挂件

use nsh_core::prelude::*;
use nsh_core::element::{AdaptiveColor, ImageSrc};

/// 渲染一个带有尺寸和指定颜色的矢量图标
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

/// 生成带有彩色圆环进度条和居中矢量图标的完整 SVG markup（参考图核心质感）
pub fn ring_progress_svg(icon_key: &str, progress: f32, color_hex: &str) -> String {
    let circumference = 87.964_f32; // 2 * PI * 14
    let progress = progress.clamp(0.01, 1.0);
    let offset = circumference * (1.0 - progress);

    let inner_paths = match icon_key {
        "droplet" | "water" => r#"<path d="M6.8 11a6 6 0 1 0 10.396 0l-5.196 -8l-5.2 8z" /><path d="M6 14a6 6 0 0 0 6 6" />"#,
        "walk" | "activity" => r#"<path d="M13 4a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" /><path d="M7 21l3 -4" /><path d="M16 21l-2 -4l-3 -3l1 -6" /><path d="M6 12l2 -3l4 -1l3 3l3 1" />"#,
        "toilet" | "poop" => r#"<path d="M10 12h4v7a3 3 0 0 1 -3 3h-2a3 3 0 0 1 -3 -3v-7h4z" /><path d="M6 4h12a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1v-3a1 1 0 0 1 1 -1z" />"#,
        "eye" => r#"<path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /><path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6" />"#,
        "pill" => r#"<path d="M4.5 12.5l8 -8a4.95 4.95 0 0 1 7 7l-8 8a4.95 4.95 0 0 1 -7 -7" /><path d="M8.5 8.5l7 7" />"#,
        "bell" | "alarm" => r#"<path d="M10 5a2 2 0 0 1 4 0a7 7 0 0 1 4 6v3a4 4 0 0 0 2 3h-16a4 4 0 0 0 2 -3v-3a7 7 0 0 1 4 -6" /><path d="M9 17v1a3 3 0 0 0 6 0v-1" />"#,
        "clock" | "focus" => r#"<path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" />"#,
        _ => r#"<path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" />"#,
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><circle cx="18" cy="18" r="14" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="2.5" /><circle cx="18" cy="18" r="14" fill="none" stroke="{color_hex}" stroke-width="2.5" stroke-dasharray="{circumference:.2}" stroke-dashoffset="{offset:.2}" stroke-linecap="round" transform="rotate(-90 18 18)" /><g transform="translate(10, 10) scale(0.666)" stroke="{color_hex}" fill="none" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">{inner_paths}</g></svg>"#
    )
}

// ==================== 业务提醒图标 ====================

/// 水滴（喝水）
pub const ICON_DROPLET: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6.8 11a6 6 0 1 0 10.396 0l-5.196 -8l-5.2 8z" /><path d="M6 14a6 6 0 0 0 6 6" /></svg>"#;

/// 走动小人（起身活动）
pub const ICON_WALK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 4a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" /><path d="M7 21l3 -4" /><path d="M16 21l-2 -4l-3 -3l1 -6" /><path d="M6 12l2 -3l4 -1l3 3l3 1" /></svg>"#;

/// 卫生间 / 便便马桶（规律排便）
pub const ICON_TOILET: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 12h4v7a3 3 0 0 1 -3 3h-2a3 3 0 0 1 -3 -3v-7h4z" /><path d="M6 4h12a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1v-3a1 1 0 0 1 1 -1z" /><path d="M10 9v3" /><path d="M14 9v3" /></svg>"#;

/// 眼睛（护眼远眺）
pub const ICON_EYE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /><path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6" /></svg>"#;

/// 药丸（服药补充）
pub const ICON_PILL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 12.5l8 -8a4.95 4.95 0 0 1 7 7l-8 8a4.95 4.95 0 0 1 -7 -7" /><path d="M8.5 8.5l7 7" /></svg>"#;

/// 铃铛 / 提醒（Bell）
pub const ICON_BELL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 5a2 2 0 0 1 4 0a7 7 0 0 1 4 6v3a4 4 0 0 0 2 3h-16a4 4 0 0 0 2 -3v-3a7 7 0 0 1 4 -6" /><path d="M9 17v1a3 3 0 0 0 6 0v-1" /></svg>"#;

/// 时钟 / 专注（Clock）
pub const ICON_CLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /><path d="M12 7v5l3 3" /></svg>"#;

/// 垃圾桶 / 删除（Trash）
pub const ICON_TRASH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7l16 0" /><path d="M10 11l0 6" /><path d="M14 11l0 6" /><path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" /><path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" /></svg>"#;

/// 警示 / 到期（Alert）纯矢量图标，杜绝 Emoji
pub const ICON_ALERT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 9v4" /><path d="M12 17h.01" /><path d="M5 19h14a2 2 0 0 0 1.84 -2.75l-7.1 -12.25a2 2 0 0 0 -3.5 0l-7.1 12.25a2 2 0 0 0 1.75 2.75" /></svg>"#;

// ==================== 操作界面图标 ====================

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

/// 下拉指示箭头（Chevron Down）
pub const ICON_CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6l6 -6" /></svg>"#;

/// 根据标识获取对应 SVG
pub fn get_icon_markup(icon_key: &str) -> &'static str {
    match icon_key {
        "droplet" | "water" => ICON_DROPLET,
        "walk" | "activity" => ICON_WALK,
        "toilet" | "poop" => ICON_TOILET,
        "eye" => ICON_EYE,
        "pill" => ICON_PILL,
        "bell" | "alarm" => ICON_BELL,
        "clock" | "focus" => ICON_CLOCK,
        _ => ICON_BELL,
    }
}

/// 获取条目的主题高光色
pub fn get_item_color(icon_key: &str) -> &'static str {
    match icon_key {
        "droplet" | "water" => "#38bdf8",  // 天空发光蓝（喝水）
        "walk" | "activity" => "#34d399",  // 清新薄荷绿（活动）
        "toilet" | "poop" => "#fbbf24",    // 暖金琥珀色（带薪拉屎）
        "eye" => "#818cf8",                // 优雅紫罗兰（护眼）
        "pill" => "#f472b6",               // 珊瑚粉红（服药）
        "bell" | "alarm" => "#eab308",      // 醒目金黄（通用提醒）
        "clock" | "focus" => "#a78bfa",    // 专注柔紫（番茄时钟）
        _ => "#38bdf8",
    }
}
