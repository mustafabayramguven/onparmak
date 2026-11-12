//! Kalıcı durum: ayarlar, geçmiş, tuş istatistikleri, ders yıldızları.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::layout::LayoutKind;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyStat {
    pub hits: u64,
    pub misses: u64,
    pub lat_sum: f64,
    pub lat_n: u64,
}

impl KeyStat {
    pub fn total(&self) -> u64 {
        self.hits + self.misses
    }

    pub fn error_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.misses as f64 / self.total() as f64
        }
    }

    pub fn latency(&self) -> f64 {
        if self.lat_n == 0 {
            0.0
        } else {
            self.lat_sum / self.lat_n as f64
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub day: u64,
    pub wpm: f64,
    pub raw: f64,
    pub acc: f64,
    pub secs: f64,
    pub label: String,
    #[serde(default)]
    pub layout: LayoutKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub layout: LayoutKind,
    pub theme: usize,
    pub keyboard: bool,
    pub ghost: bool,
    pub unlock_all: bool,
    pub set: usize,
    pub mode: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            layout: LayoutKind::F,
            theme: 0,
            keyboard: true,
            ghost: true,
            unlock_all: false,
            set: 0,
            mode: 1,
        }
    }
}

/// Zayıf bir tuş: karakter, hata oranı, ortalama gecikme (ms).
#[derive(Clone, Copy, Debug)]
pub struct Weak {
    pub ch: char,
    pub error_rate: f64,
    pub latency: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub settings: Settings,
    pub history: Vec<Record>,
    pub keys: HashMap<String, KeyStat>,
    pub lessons: HashMap<String, u8>,
    pub days: Vec<u64>,
}

impl State {
    pub fn path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("ONPARMAK_STATE") {
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
        dirs::data_dir().map(|d| d.join("onparmak").join("state.json"))
    }

    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    /// Unix çağından bu yana gün (UTC).
    pub fn today() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() / 86_400)
            .unwrap_or(0)
    }

    pub fn mark_today(&mut self) {
        let t = Self::today();
        if !self.days.contains(&t) {
            self.days.push(t);
        }
    }

    /// Ardışık çalışılan gün sayısı (bugün ya da dün biten seri).
    pub fn streak(&self) -> u32 {
        let today = Self::today();
        let mut day = if self.days.contains(&today) {
            today
        } else if today > 0 && self.days.contains(&(today - 1)) {
            today - 1
        } else {
            return 0;
        };
        let mut n = 0;
        while self.days.contains(&day) {
            n += 1;
            if day == 0 {
                break;
            }
            day -= 1;
        }
        n
    }

    pub fn key_id(layout: LayoutKind, ch: char) -> String {
        format!("{}:{}", layout.short(), ch)
    }

    pub fn lesson_id(layout: LayoutKind, lesson: usize, step: usize) -> String {
        format!("{}:{}:{}", layout.short(), lesson, step)
    }

    pub fn stars(&self, layout: LayoutKind, lesson: usize, step: usize) -> u8 {
        self.lessons
            .get(&Self::lesson_id(layout, lesson, step))
            .copied()
            .unwrap_or(0)
    }

    pub fn set_stars(&mut self, layout: LayoutKind, lesson: usize, step: usize, stars: u8) {
        let e = self
            .lessons
            .entry(Self::lesson_id(layout, lesson, step))
            .or_insert(0);
        if stars > *e {
            *e = stars;
        }
    }

    pub fn lesson_done(&self, layout: LayoutKind, lesson: usize, steps: usize) -> bool {
        (0..steps).all(|s| self.stars(layout, lesson, s) > 0)
    }

    pub fn history_for(&self, layout: LayoutKind) -> Vec<&Record> {
        self.history.iter().filter(|r| r.layout == layout).collect()
    }

    pub fn best_for(&self, label: &str, layout: LayoutKind) -> Option<f64> {
        self.history
            .iter()
            .filter(|r| r.layout == layout && r.label == label)
            .map(|r| r.wpm)
            .fold(None, |acc: Option<f64>, w| Some(acc.map_or(w, |a| a.max(w))))
    }

    /// Bir düzene ait tuş istatistikleri.
    pub fn key_stats(&self, layout: LayoutKind) -> Vec<(char, &KeyStat)> {
        let prefix = format!("{}:", layout.short());
        self.keys
            .iter()
            .filter_map(|(k, st)| {
                let ch = k.strip_prefix(prefix.as_str())?.chars().next()?;
                Some((ch, st))
            })
            .collect()
    }

    /// En zayıf harfler: hata oranı ağır, yavaşlık hafif basar.
    pub fn weak_keys(&self, layout: LayoutKind, n: usize) -> Vec<Weak> {
        let mut v: Vec<(Weak, f64)> = self
            .key_stats(layout)
            .into_iter()
            .filter(|(ch, st)| ch.is_alphabetic() && st.total() >= 8)
            .map(|(ch, st)| {
                let w = Weak {
                    ch,
                    error_rate: st.error_rate(),
                    latency: st.latency(),
                };
                let score = w.error_rate * 4.0 + w.latency / 1000.0;
                (w, score)
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        v.into_iter().take(n).map(|(w, _)| w).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stars_only_go_up() {
        let mut s = State::default();
        s.set_stars(LayoutKind::F, 0, 0, 2);
        s.set_stars(LayoutKind::F, 0, 0, 1);
        assert_eq!(s.stars(LayoutKind::F, 0, 0), 2);
        assert_eq!(s.stars(LayoutKind::Q, 0, 0), 0);
        assert!(!s.lesson_done(LayoutKind::F, 0, 3));
        s.set_stars(LayoutKind::F, 0, 1, 1);
        s.set_stars(LayoutKind::F, 0, 2, 3);
        assert!(s.lesson_done(LayoutKind::F, 0, 3));
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let mut s = State::default();
        let t = State::today();
        s.days = vec![t, t - 1, t - 2, t - 4];
        assert_eq!(s.streak(), 3);
        s.days = vec![t - 1, t - 2];
        assert_eq!(s.streak(), 2);
        s.days = vec![t - 3];
        assert_eq!(s.streak(), 0);
    }

    #[test]
    fn weak_keys_rank_errors() {
        let mut s = State::default();
        s.keys.insert(
            State::key_id(LayoutKind::F, 'ğ'),
            KeyStat { hits: 10, misses: 10, lat_sum: 4000.0, lat_n: 10 },
        );
        s.keys.insert(
            State::key_id(LayoutKind::F, 'a'),
            KeyStat { hits: 100, misses: 1, lat_sum: 15000.0, lat_n: 100 },
        );
        s.keys.insert(
            State::key_id(LayoutKind::F, ':'),
            KeyStat { hits: 1, misses: 50, lat_sum: 0.0, lat_n: 0 },
        );
        let weak = s.weak_keys(LayoutKind::F, 5);
        assert_eq!(weak[0].ch, 'ğ');
        assert_eq!(weak.len(), 2);
    }

    #[test]
    fn roundtrip_json() {
        let mut s = State::default();
        s.keys.insert(State::key_id(LayoutKind::Q, 'ş'), KeyStat::default());
        s.history.push(Record {
            day: 1,
            wpm: 40.0,
            raw: 45.0,
            acc: 97.0,
            secs: 30.0,
            label: "x".into(),
            layout: LayoutKind::Q,
        });
        let json = serde_json::to_string(&s).unwrap();
        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.history.len(), 1);
        assert_eq!(back.history[0].layout, LayoutKind::Q);
        assert!(back.keys.contains_key("Q:ş"));
    }
}
