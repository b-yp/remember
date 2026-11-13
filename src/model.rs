use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn default_item_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// 初始化下一个分配的 ID，确保反序列化后新建项目的 ID 不与已有项目冲突
pub fn init_next_id(start_id: u64) {
    let mut current = NEXT_ID.load(Ordering::Relaxed);
    while current < start_id {
        match NEXT_ID.compare_exchange_weak(current, start_id, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => current = actual,
        }
    }
}

/// 提醒项目模型（纯数据结构，与 UI 生命周期 Owner 解耦，安全可靠）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReminderItem {
    #[serde(default = "default_item_id")]
    pub id: u64,
    pub name: String,
    pub icon: String,
    pub interval_secs: u32,
    #[serde(default)]
    pub remaining_secs: u32,
    #[serde(default)]
    pub completed_count: u32,
    #[serde(default)]
    pub is_paused: bool,
    /// 倒计时归零后的超时持续秒数（用于超时动态统计与持续提醒，避免静止在0）
    #[serde(default)]
    pub overdue_secs: u32,
}

impl ReminderItem {
    pub fn new(name: impl Into<String>, icon: impl Into<String>, interval_secs: u32) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            name: name.into(),
            icon: icon.into(),
            interval_secs,
            remaining_secs: interval_secs,
            completed_count: 0,
            is_paused: false,
            overdue_secs: 0,
        }
    }

    /// 完成打卡：计数 +1，倒计时重置回设定周期，清空超时状态
    pub fn complete(&mut self) {
        self.completed_count += 1;
        self.remaining_secs = self.interval_secs;
        self.overdue_secs = 0;
    }

    /// 推迟提醒（例如加 5 分钟 = 300 秒），清空超时状态
    pub fn snooze(&mut self, extra_secs: u32) {
        self.remaining_secs += extra_secs;
        self.overdue_secs = 0;
    }

    /// 切换暂停/恢复
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// 定时器每秒 Tick 递减倒计时
    /// 返回 true 表示此秒恰好倒计时归零（用于触发提示音、系统通知与窗口唤起）
    pub fn tick(&mut self) -> bool {
        if self.is_paused {
            return false;
        }
        if self.remaining_secs > 0 {
            self.remaining_secs -= 1;
            if self.remaining_secs == 0 {
                return true;
            }
        } else {
            self.overdue_secs = self.overdue_secs.saturating_add(1);
        }
        false
    }

    /// 当前是否处于已到期状态
    pub fn is_expired(&self) -> bool {
        self.remaining_secs == 0
    }

    /// 格式化当前剩余时间或超时时间（纯文本，绝无 Emoji，符合专业高质感设计目标）
    pub fn format_remaining(&self) -> String {
        let secs = self.remaining_secs;
        if secs == 0 {
            if self.overdue_secs == 0 {
                "到时间啦!".to_string()
            } else {
                let m = self.overdue_secs / 60;
                let s = self.overdue_secs % 60;
                format!("超时 {:02}:{:02}", m, s)
            }
        } else {
            let m = secs / 60;
            let s = secs % 60;
            format!("{:02}:{:02}", m, s)
        }
    }

    /// 进度比例（0.0 ~ 1.0）
    pub fn progress_ratio(&self) -> f32 {
        if self.remaining_secs == 0 {
            return 0.0;
        }
        let cur = self.remaining_secs as f32;
        let total = self.interval_secs.max(1) as f32;
        (cur / total).clamp(0.0, 1.0)
    }

    /// 格式化循环周期（如 "30m"、"1h"、"1h30m"）
    pub fn format_interval(&self) -> String {
        let mins = self.interval_secs / 60;
        if mins >= 60 {
            let h = mins / 60;
            let m = mins % 60;
            if m == 0 {
                format!("{}h", h)
            } else {
                format!("{}h{}m", h, m)
            }
        } else {
            format!("{}m", mins)
        }
    }

    /// 就地更新循环周期时长（分钟），并将超出新周期的倒计时拉齐
    pub fn update_interval_mins(&mut self, mins: u32) {
        let new_secs = mins.max(1) * 60;
        self.interval_secs = new_secs;
        if self.remaining_secs > new_secs {
            self.remaining_secs = new_secs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reminder_item_lifecycle() {
        let mut item = ReminderItem::new("测试喝水", "droplet", 60);
        assert_eq!(item.remaining_secs, 60);
        assert_eq!(item.completed_count, 0);
        assert_eq!(item.format_remaining(), "01:00");
        assert!(!item.is_expired());

        // 模拟 1 秒倒计时
        let expired = item.tick();
        assert!(!expired);
        assert_eq!(item.remaining_secs, 59);
        assert_eq!(item.format_remaining(), "00:59");

        // 推迟 5 分钟 (300秒)
        item.snooze(300);
        assert_eq!(item.remaining_secs, 359);
        assert_eq!(item.overdue_secs, 0);

        // 完成打卡
        item.complete();
        assert_eq!(item.completed_count, 1);
        assert_eq!(item.remaining_secs, 60);

        // 暂停测试
        item.toggle_pause();
        assert!(item.is_paused);
        let expired = item.tick();
        assert!(!expired);
        assert_eq!(item.remaining_secs, 60); // 暂停时不递减

        // 恢复并递减至归零
        item.toggle_pause();
        item.remaining_secs = 1;
        let expired = item.tick();
        assert!(expired); // 刚刚归零触发提醒
        assert_eq!(item.remaining_secs, 0);
        assert!(item.is_expired());
        assert_eq!(item.format_remaining(), "到时间啦!");

        // 归零后继续 tick：累加超时时间，避免永远停在0
        let expired = item.tick();
        assert!(!expired);
        assert_eq!(item.overdue_secs, 1);
        assert_eq!(item.format_remaining(), "超时 00:01");

        // 周期格式化测试
        assert_eq!(item.format_interval(), "1m");
        item.update_interval_mins(90);
        assert_eq!(item.interval_secs, 90 * 60);
        assert_eq!(item.format_interval(), "1h30m");
        item.update_interval_mins(120);
        assert_eq!(item.format_interval(), "2h");
    }
}
