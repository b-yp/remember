//! 本地持久化模块：基于 JSON 存储挂件数据与状态
//! 遵循平台标准应用数据存储规范，支持掉电安全的原子写入、窗口历史坐标还原与启动离线时间流逝智能补偿

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::model::{init_next_id, ReminderItem};
use crate::theme_mode::ThemeMode;

fn default_theme_mode() -> ThemeMode {
    ThemeMode::Dark
}

fn default_opacity() -> f32 {
    1.0
}

/// 应用程序持久化完整状态
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub items: Vec<ReminderItem>,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub is_mini: bool,
    #[serde(default)]
    pub is_all_paused: bool,
    /// 窗口历史摆放物理坐标 (x, y)，开机恢复上次摆放位置
    #[serde(default)]
    pub window_pos: Option<(i32, i32)>,
    /// 上次保存的时间戳（Unix 时间戳，秒），用于计算离线期间流逝的时间
    #[serde(default)]
    pub last_saved_timestamp: Option<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            items: vec![
                ReminderItem::new("喝水", "droplet", 30 * 60),
                ReminderItem::new("起身活动", "walk", 45 * 60),
                ReminderItem::new("带薪拉屎", "toilet", 90 * 60),
            ],
            theme_mode: ThemeMode::Dark,
            opacity: 1.0,
            is_mini: false,
            is_all_paused: false,
            window_pos: None,
            last_saved_timestamp: current_timestamp(),
        }
    }
}

/// 获取当前系统的 Unix 时间戳（秒）
pub fn current_timestamp() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// 根据操作系统标准规范获取状态存储路径，支持 REMINDER_DATA_PATH 环境变量覆盖
pub fn get_storage_path() -> PathBuf {
    if let Ok(custom) = std::env::var("REMINDER_DATA_PATH") {
        return PathBuf::from(custom);
    }

    let base_dir = if cfg!(target_os = "macos") {
        std::env::var("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/com.reminder.app"))
    } else if cfg!(target_os = "windows") {
        std::env::var("APPDATA").map(|a| PathBuf::from(a).join("com.reminder.app"))
    } else {
        // Linux / Unix 标准 XDG 规范
        std::env::var("XDG_CONFIG_HOME")
            .map(|x| PathBuf::from(x).join("reminder"))
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config/reminder")))
    };

    match base_dir {
        Ok(dir) => dir.join("state.json"),
        Err(_) => PathBuf::from("reminder_state.json"),
    }
}

/// 从本地 JSON 文件加载状态
pub fn load_state() -> AppState {
    let path = get_storage_path();
    load_state_from_path(&path)
}

/// 从指定路径加载持久化状态
pub fn load_state_from_path(path: &std::path::Path) -> AppState {
    let mut state = if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str::<AppState>(&content).unwrap_or_else(|e| {
                eprintln!("[reminder] 解析持久化 JSON 失败，使用默认配置: {e}");
                AppState::default()
            }),
            Err(e) => {
                eprintln!("[reminder] 读取持久化文件失败: {e}");
                AppState::default()
            }
        }
    } else {
        AppState::default()
    };

    if state.items.is_empty() {
        state.items = AppState::default().items;
    }

    // 1. 同步自增 ID 计数器，防止重启后新建 ID 冲突
    let max_id = state.items.iter().map(|it| it.id).max().unwrap_or(0);
    init_next_id(max_id + 1);

    // 2. 离线时间补偿计算
    if let (Some(saved_time), Some(now)) = (state.last_saved_timestamp, current_timestamp()) {
        if now > saved_time && !state.is_all_paused {
            let elapsed_secs = (now - saved_time).min(7 * 24 * 3600) as u32; // 上限保护7天
            apply_elapsed_time(&mut state.items, elapsed_secs);
        }
    }

    state
}

/// 对未暂停的提醒项目应用离线流逝秒数
pub fn apply_elapsed_time(items: &mut [ReminderItem], elapsed_secs: u32) {
    if elapsed_secs == 0 {
        return;
    }
    for item in items.iter_mut() {
        if !item.is_paused {
            if item.remaining_secs > elapsed_secs {
                item.remaining_secs -= elapsed_secs;
            } else {
                let overflow = elapsed_secs - item.remaining_secs;
                item.remaining_secs = 0;
                item.overdue_secs = item.overdue_secs.saturating_add(overflow);
            }
        }
    }
}

/// 将状态以 JSON 格式安全写入磁盘（原子写入，防止意外关机导致文件损坏）
pub fn save_state(state: &mut AppState) -> Result<(), std::io::Error> {
    let path = get_storage_path();
    save_state_to_path(state, &path)
}

/// 将状态以原子写入方式安全落盘到指定路径（针对 Windows 目标存在场景具备安全覆盖与降级保护）
pub fn save_state_to_path(state: &mut AppState, path: &std::path::Path) -> Result<(), std::io::Error> {
    state.last_saved_timestamp = current_timestamp();

    let json_bytes = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, json_bytes)?;

    // Windows 上若目标文件已存在，直接 rename 可能会报错，做防御性安全替换与降级覆盖
    if fs::rename(&tmp_path, path).is_err() {
        let _ = fs::remove_file(path);
        if fs::rename(&tmp_path, path).is_err() {
            fs::copy(&tmp_path, path)?;
            let _ = fs::remove_file(&tmp_path);
        }
    }

    Ok(())
}

/// 全局快捷持久化：接收各状态快照（含窗口坐标）并落盘
pub fn persist_current(
    items: &[ReminderItem],
    theme_mode: ThemeMode,
    opacity: f32,
    is_mini: bool,
    is_all_paused: bool,
    window_pos: Option<(i32, i32)>,
) {
    let mut state = AppState {
        items: items.to_vec(),
        theme_mode,
        opacity,
        is_mini,
        is_all_paused,
        window_pos,
        last_saved_timestamp: None,
    };
    if let Err(e) = save_state(&mut state) {
        eprintln!("[reminder] 持久化保存失败: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.items.len(), 3);
        assert_eq!(state.theme_mode, ThemeMode::Dark);
        assert_eq!(state.opacity, 1.0);
        assert!(!state.is_mini);
        assert!(!state.is_all_paused);
        assert_eq!(state.window_pos, None);
        assert!(state.last_saved_timestamp.is_some());
    }

    #[test]
    fn test_serde_json_roundtrip() {
        let mut state = AppState::default();
        state.window_pos = Some((200, 300));
        let json = serde_json::to_string(&state).unwrap();
        let restored: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.items.len(), restored.items.len());
        assert_eq!(state.theme_mode, restored.theme_mode);
        assert_eq!(state.opacity, restored.opacity);
        assert_eq!(state.window_pos, Some((200, 300)));
    }

    #[test]
    fn test_apply_elapsed_time_normal_and_overdue() {
        let mut items = vec![
            ReminderItem::new("喝水", "droplet", 1800),
            ReminderItem::new("活动", "walk", 60),
            ReminderItem::new("暂停项", "toilet", 300),
        ];
        items[2].is_paused = true;

        // 流逝 100 秒
        apply_elapsed_time(&mut items, 100);

        // 喝水：1800 - 100 = 1700
        assert_eq!(items[0].remaining_secs, 1700);
        assert_eq!(items[0].overdue_secs, 0);

        // 活动：60 - 100 -> remaining=0, overdue=40
        assert_eq!(items[1].remaining_secs, 0);
        assert_eq!(items[1].overdue_secs, 40);

        // 暂停项：不变
        assert_eq!(items[2].remaining_secs, 300);
        assert_eq!(items[2].overdue_secs, 0);
    }

    #[test]
    fn test_file_io_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("reminder_test_{}", std::process::id()));
        let file_path = temp_dir.join("test_state.json");

        let mut state = AppState {
            items: vec![ReminderItem::new("自定任务", "target", 1200)],
            theme_mode: ThemeMode::Light,
            opacity: 0.85,
            is_mini: true,
            is_all_paused: true,
            window_pos: Some((450, 520)),
            last_saved_timestamp: current_timestamp(),
        };

        // 保存到临时文件
        save_state_to_path(&mut state, &file_path).expect("save state failed");
        assert!(file_path.exists());

        // 读取回来
        let loaded = load_state_from_path(&file_path);
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].name, "自定任务");
        assert_eq!(loaded.theme_mode, ThemeMode::Light);
        assert_eq!(loaded.opacity, 0.85);
        assert!(loaded.is_mini);
        assert!(loaded.is_all_paused);
        assert_eq!(loaded.window_pos, Some((450, 520)));

        // 连续二次写入测试 Windows 覆盖兼容性
        state.opacity = 0.90;
        assert!(save_state_to_path(&mut state, &file_path).is_ok());
        let loaded2 = load_state_from_path(&file_path);
        assert_eq!(loaded2.opacity, 0.90);

        // 清理临时文件
        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_corrupted_json_fallback_to_default() {
        let temp_dir = std::env::temp_dir().join(format!("reminder_corrupt_{}", std::process::id()));
        let file_path = temp_dir.join("corrupt.json");
        let _ = fs::create_dir_all(&temp_dir);

        // 写入非法 JSON
        fs::write(&file_path, "{ broken json content").unwrap();

        // 应该降级为 default 而非崩溃 panic
        let state = load_state_from_path(&file_path);
        assert_eq!(state.items.len(), 3);
        assert_eq!(state.theme_mode, ThemeMode::Dark);

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_id_generator_monotonic_after_load() {
        let temp_dir = std::env::temp_dir().join(format!("reminder_id_{}", std::process::id()));
        let file_path = temp_dir.join("id_test.json");

        let mut item = ReminderItem::new("高编号项", "bell", 600);
        item.id = 100;

        let mut state = AppState {
            items: vec![item],
            ..Default::default()
        };

        save_state_to_path(&mut state, &file_path).unwrap();
        let _ = load_state_from_path(&file_path);

        // 之后新建的 item 其 ID 必须大于 100
        let new_item = ReminderItem::new("新任务", "clock", 300);
        assert!(new_item.id > 100);

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
