//! Bir yazma oturumu: hedef metin, girilenler, zamanlama ve tuş istatistikleri.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// Süre dolunca biter (saniye).
    Time(u64),
    /// Metnin tamamı yazılınca biter.
    Full,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub sec: f64,
    pub wpm: f64,
    pub raw: f64,
    pub errors: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CharStat {
    pub hits: u32,
    pub misses: u32,
    pub lat_sum: f64,
    pub lat_n: u32,
}

pub struct Session {
    pub target: Vec<char>,
    pub typed: Vec<char>,
    /// Sıkı modda yanlış tuş ilerlemez (dersler).
    pub strict: bool,
    pub limit: Limit,
    pub start: Option<Instant>,
    pub end: Option<Instant>,
    pub keystrokes: u32,
    pub errors: u32,
    pub chars: HashMap<char, CharStat>,
    pub samples: Vec<Sample>,
    pub last_wrong: Option<(char, Instant)>,
    last_key: Option<Instant>,
    sec_errors: u32,
    sec_typed: usize,
}

impl Session {
    pub fn new(text: &str, strict: bool, limit: Limit) -> Self {
        Session {
            target: text.trim().chars().collect(),
            typed: Vec::new(),
            strict,
            limit,
            start: None,
            end: None,
            keystrokes: 0,
            errors: 0,
            chars: HashMap::new(),
            samples: Vec::new(),
            last_wrong: None,
            last_key: None,
            sec_errors: 0,
            sec_typed: 0,
        }
    }

    pub fn started(&self) -> bool {
        self.start.is_some()
    }

    pub fn finished(&self) -> bool {
        self.end.is_some()
    }

    pub fn type_char(&mut self, c: char) {
        self.type_char_at(c, Instant::now());
    }

    pub fn type_char_at(&mut self, c: char, now: Instant) {
        if self.end.is_some() || self.typed.len() >= self.target.len() {
            return;
        }
        if self.start.is_none() {
            self.start = Some(now);
        }
        let idx = self.typed.len();
        let expected = self.target[idx];
        self.keystrokes += 1;
        let ok = c == expected;
        let latency = self
            .last_key
            .map(|t| now.duration_since(t).as_secs_f64() * 1000.0);
        self.last_key = Some(now);

        let stat = self.chars.entry(expected).or_default();
        if ok {
            stat.hits += 1;
            if let Some(ms) = latency {
                if ms < 2500.0 {
                    stat.lat_sum += ms;
                    stat.lat_n += 1;
                }
            }
        } else {
            stat.misses += 1;
            self.errors += 1;
            self.sec_errors += 1;
            self.last_wrong = Some((c, now));
        }

        if ok || !self.strict {
            self.typed.push(c);
        }
        if self.limit == Limit::Full && self.typed.len() >= self.target.len() {
            self.end = Some(now);
            self.push_sample(now);
        }
    }

    pub fn backspace(&mut self) {
        if self.end.is_none() {
            self.typed.pop();
        }
    }

    /// Önceki kelimeyi siler (Ctrl+W / Ctrl+Backspace).
    pub fn delete_word(&mut self) {
        if self.end.is_some() {
            return;
        }
        while self.typed.last() == Some(&' ') {
            self.typed.pop();
        }
        while let Some(&c) = self.typed.last() {
            if c == ' ' {
                break;
            }
            self.typed.pop();
        }
    }

    pub fn tick(&mut self) {
        self.tick_at(Instant::now());
    }

    pub fn tick_at(&mut self, now: Instant) {
        let Some(start) = self.start else { return };
        if self.end.is_some() {
            return;
        }
        let elapsed = now.duration_since(start).as_secs_f64();
        if let Limit::Time(limit) = self.limit {
            if elapsed >= limit as f64 {
                while (self.samples.len() as u64) < limit {
                    let at = start + Duration::from_secs(self.samples.len() as u64 + 1);
                    self.push_sample(at);
                }
                self.end = Some(start + Duration::from_secs(limit));
                return;
            }
        }
        while (self.samples.len() as f64) + 1.0 <= elapsed {
            let at = start + Duration::from_secs(self.samples.len() as u64 + 1);
            self.push_sample(at);
        }
    }

    fn push_sample(&mut self, at: Instant) {
        let Some(start) = self.start else { return };
        let secs = at.duration_since(start).as_secs_f64().max(0.001);
        let prev = self.samples.last().map(|s| s.sec).unwrap_or(0.0);
        let span = (secs - prev).max(0.25);
        let wpm = self.wpm_with(secs);
        let delta = self.typed.len().saturating_sub(self.sec_typed) as f64;
        let raw = delta / 5.0 / (span / 60.0);
        self.samples.push(Sample {
            sec: secs,
            wpm,
            raw,
            errors: self.sec_errors,
        });
        self.sec_errors = 0;
        self.sec_typed = self.typed.len();
    }

    pub fn elapsed(&self) -> f64 {
        match (self.start, self.end) {
            (Some(s), Some(e)) => e.duration_since(s).as_secs_f64(),
            (Some(s), None) => s.elapsed().as_secs_f64(),
            _ => 0.0,
        }
    }

    /// Süreli modda kalan saniye.
    pub fn remaining(&self) -> Option<f64> {
        match self.limit {
            Limit::Time(t) => Some((t as f64 - self.elapsed()).max(0.0)),
            Limit::Full => None,
        }
    }

    pub fn correct_chars(&self) -> usize {
        self.typed
            .iter()
            .zip(self.target.iter())
            .filter(|(a, b)| a == b)
            .count()
    }

    pub fn wpm_with(&self, secs: f64) -> f64 {
        if secs <= 0.0 {
            return 0.0;
        }
        self.correct_chars() as f64 / 5.0 / (secs.max(1.0) / 60.0)
    }

    pub fn wpm(&self) -> f64 {
        self.wpm_with(self.elapsed())
    }

    pub fn raw(&self) -> f64 {
        let secs = self.elapsed();
        if secs <= 0.0 {
            return 0.0;
        }
        self.typed.len() as f64 / 5.0 / (secs.max(1.0) / 60.0)
    }

    /// Basılan tuşlar içinde doğru olanların yüzdesi (düzeltilen hatalar da sayılır).
    pub fn accuracy(&self) -> f64 {
        if self.keystrokes == 0 {
            return 100.0;
        }
        100.0 * (self.keystrokes - self.errors.min(self.keystrokes)) as f64
            / self.keystrokes as f64
    }

    /// Saniyelik ham hızın ne kadar dengeli olduğu (100 = kusursuz ritim).
    pub fn consistency(&self) -> f64 {
        let raws: Vec<f64> = self.samples.iter().map(|s| s.raw).collect();
        if raws.len() < 2 {
            return 100.0;
        }
        let mean = raws.iter().sum::<f64>() / raws.len() as f64;
        if mean <= 0.0 {
            return 0.0;
        }
        let var = raws.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / raws.len() as f64;
        (100.0 * (1.0 - var.sqrt() / mean)).clamp(0.0, 100.0)
    }

    /// Süreli modda metnin sonuna yaklaşıldı mı?
    pub fn needs_more(&self) -> bool {
        matches!(self.limit, Limit::Time(_))
            && self.target.len().saturating_sub(self.typed.len()) < 120
    }

    pub fn extend(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if self.target.last().is_some_and(|c| *c != ' ') {
            self.target.push(' ');
        }
        self.target.extend(text.chars());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_mode_counts_errors() {
        let mut s = Session::new("ana", false, Limit::Full);
        let t0 = Instant::now();
        s.type_char_at('a', t0);
        s.type_char_at('m', t0 + Duration::from_millis(200));
        s.type_char_at('a', t0 + Duration::from_millis(400));
        assert!(s.finished());
        assert_eq!(s.correct_chars(), 2);
        assert_eq!(s.errors, 1);
        assert!((s.accuracy() - 66.666).abs() < 0.1);
    }

    #[test]
    fn strict_mode_blocks_wrong_keys() {
        let mut s = Session::new("ak", true, Limit::Full);
        let t0 = Instant::now();
        s.type_char_at('a', t0);
        s.type_char_at('x', t0);
        assert_eq!(s.typed, vec!['a']);
        assert!(s.last_wrong.is_some());
        s.type_char_at('k', t0);
        assert!(s.finished());
        assert_eq!(s.errors, 1);
        assert_eq!(s.keystrokes, 3);
    }

    #[test]
    fn wpm_math() {
        let mut s = Session::new("kelime kelime", false, Limit::Full);
        for c in "kelime kel".chars() {
            s.type_char_at(c, Instant::now());
        }
        // 10 doğru karakter = 2 kelime; 30 saniyede 4 wpm.
        assert!((s.wpm_with(30.0) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn delete_word() {
        let mut s = Session::new("bir iki üç", false, Limit::Full);
        for c in "bir ik".chars() {
            s.type_char(c);
        }
        s.delete_word();
        assert_eq!(s.typed.iter().collect::<String>(), "bir ");
        s.delete_word();
        assert!(s.typed.is_empty());
    }

    #[test]
    fn time_limit_ends_session() {
        let mut s = Session::new("a a a a a a a a", false, Limit::Time(2));
        let t0 = Instant::now();
        s.type_char_at('a', t0);
        s.tick_at(t0 + Duration::from_millis(1500));
        assert!(!s.finished());
        assert_eq!(s.samples.len(), 1);
        s.tick_at(t0 + Duration::from_millis(2100));
        assert!(s.finished());
        assert_eq!(s.samples.len(), 2);
        assert!((s.elapsed() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn extend_adds_separator() {
        let mut s = Session::new("bir", false, Limit::Time(30));
        s.extend("iki");
        assert_eq!(s.target.iter().collect::<String>(), "bir iki");
    }
}
