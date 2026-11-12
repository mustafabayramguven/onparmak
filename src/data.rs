//! Gömülü Türkçe kelime setleri ve pratik metni üretimi.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetKind {
    /// Birbirinden bağımsız kelimeler.
    Words,
    /// Her öğe bir cümle (atasözleri).
    Sentences,
    /// Sık kelimelerden büyük harf ve noktalamalı yapay cümleler.
    Realistic,
}

#[derive(Clone, Debug)]
pub struct WordSet {
    pub id: &'static str,
    pub title: String,
    pub desc: String,
    pub items: Vec<String>,
    pub kind: SetKind,
}

impl WordSet {
    pub fn word_count(&self) -> usize {
        self.items.iter().map(|i| i.split_whitespace().count()).sum()
    }
}

const RAW: [(&str, &str); 7] = [
    ("en-sik-500", include_str!("../data/sets/en-sik-500.txt")),
    ("sik-1000", include_str!("../data/sets/sik-1000.txt")),
    ("cekimli-kelimeler", include_str!("../data/sets/cekimli-kelimeler.txt")),
    ("uzun-kelimeler", include_str!("../data/sets/uzun-kelimeler.txt")),
    ("turkce-karakterler", include_str!("../data/sets/turkce-karakterler.txt")),
    ("zor-kelimeler", include_str!("../data/sets/zor-kelimeler.txt")),
    ("atasozleri", include_str!("../data/sets/atasozleri.txt")),
];

fn parse(id: &'static str, raw: &str) -> WordSet {
    let mut title = id.to_string();
    let mut desc = String::new();
    let mut headers = 0;
    let mut items = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            if headers == 0 {
                title = rest.to_string();
            } else if headers == 1 {
                desc = rest.to_string();
            }
            headers += 1;
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        items.push(line.to_string());
    }
    let kind = if id == "atasozleri" {
        SetKind::Sentences
    } else {
        SetKind::Words
    };
    WordSet {
        id,
        title,
        desc,
        items,
        kind,
    }
}

pub fn load() -> Vec<WordSet> {
    let mut sets: Vec<WordSet> = RAW.iter().map(|&(id, raw)| parse(id, raw)).collect();
    let mut items: Vec<String> = Vec::new();
    for s in sets
        .iter()
        .filter(|s| s.id == "en-sik-500" || s.id == "sik-1000")
    {
        items.extend(s.items.iter().cloned());
    }
    let realistic = WordSet {
        id: "gercekci",
        title: "Gerçekçi metin".into(),
        desc: "Sık kelimelerden kurulmuş cümleler: büyük harf, virgül, nokta, soru ve ünlem.".into(),
        items,
        kind: SetKind::Realistic,
    };
    let at = 2.min(sets.len());
    sets.insert(at, realistic);
    sets
}

/// Derslerde gerçek kelime üretmek için tüm tekil kelimeler.
pub fn lesson_words(sets: &[WordSet]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for s in sets.iter().filter(|s| s.kind == SetKind::Words) {
        for w in &s.items {
            if w.chars().all(|c| c.is_alphabetic()) && seen.insert(w.clone()) {
                out.push(w.clone());
            }
        }
    }
    out
}

pub fn proverbs(sets: &[WordSet]) -> Vec<String> {
    sets.iter()
        .find(|s| s.kind == SetKind::Sentences)
        .map(|s| s.items.clone())
        .unwrap_or_default()
}

/// Bir setten yaklaşık `count` kelimelik pratik metni üretir.
pub fn practice_text<R: Rng>(set: &WordSet, count: usize, rng: &mut R) -> String {
    if set.items.is_empty() || count == 0 {
        return String::new();
    }
    match set.kind {
        SetKind::Words => {
            let mut out: Vec<&str> = Vec::with_capacity(count);
            while out.len() < count {
                let Some(w) = set.items.choose(rng) else { break };
                if out.last() == Some(&w.as_str()) && set.items.len() > 1 {
                    continue;
                }
                out.push(w.as_str());
            }
            out.join(" ")
        }
        SetKind::Sentences => {
            let mut pool: Vec<&String> = set.items.iter().collect();
            pool.shuffle(rng);
            let mut words: Vec<&str> = Vec::with_capacity(count + 16);
            let mut i = 0;
            while words.len() < count {
                if i >= pool.len() {
                    pool.shuffle(rng);
                    i = 0;
                }
                words.extend(pool[i].split_whitespace());
                i += 1;
            }
            words.truncate(count);
            words.join(" ")
        }
        SetKind::Realistic => {
            let mut out: Vec<String> = Vec::with_capacity(count);
            let mut capital = true;
            let mut since = 0;
            let mut sentence_len = rng.gen_range(5..=11);
            for i in 0..count {
                let Some(base) = set.items.choose(rng) else { break };
                let mut w = if capital {
                    text::capitalize(base)
                } else {
                    base.clone()
                };
                capital = false;
                since += 1;
                if since >= sentence_len || i + 1 == count {
                    let end = match rng.gen_range(0..10) {
                        0 => '?',
                        1 => '!',
                        _ => '.',
                    };
                    w.push(end);
                    capital = true;
                    since = 0;
                    sentence_len = rng.gen_range(5..=11);
                } else if rng.gen_bool(0.12) {
                    w.push(',');
                }
                out.push(w);
            }
            out.join(" ")
        }
    }
}

/// Zayıf harfleri içeren kelimelerden alıştırma üretir.
pub fn weak_text<R: Rng>(words: &[String], weak: &[char], count: usize, rng: &mut R) -> String {
    let mut pool: Vec<&String> = Vec::new();
    for w in words {
        let hits = w.chars().filter(|c| weak.contains(c)).count();
        for _ in 0..hits.min(3) {
            pool.push(w);
        }
    }
    let mut out: Vec<String> = Vec::with_capacity(count);
    if pool.is_empty() {
        return words
            .choose_multiple(rng, count)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
    }
    for i in 0..count {
        if i % 6 == 5 && weak.len() >= 2 {
            // Kısa bir tuş ikilisi tekrarı: iki zayıf harf arasında gidip gelme.
            let a = weak[rng.gen_range(0..weak.len())];
            let b = weak[rng.gen_range(0..weak.len())];
            out.push([a, b, a, b].iter().collect());
            continue;
        }
        if let Some(w) = pool.choose(rng) {
            out.push((*w).to_string());
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_set_has_500_words() {
        let sets = load();
        assert!(sets.len() >= 8);
        for s in &sets {
            assert!(s.word_count() >= 500, "{} yalnızca {} kelime", s.id, s.word_count());
            assert!(!s.title.is_empty());
            assert!(!s.desc.is_empty(), "{} açıklaması yok", s.id);
        }
    }

    #[test]
    fn words_are_lowercase_turkish() {
        for s in load().iter().filter(|s| s.kind == SetKind::Words) {
            for w in &s.items {
                assert!(
                    w.chars().all(|c| "abcçdefgğhıijklmnoöprsştuüvyz".contains(c)),
                    "{}: beklenmeyen kelime {w:?}",
                    s.id
                );
            }
        }
    }

    #[test]
    fn practice_word_counts() {
        let sets = load();
        let mut rng = rand::thread_rng();
        for s in &sets {
            let t = practice_text(s, 25, &mut rng);
            assert_eq!(t.split_whitespace().count(), 25, "{}", s.id);
        }
    }

    #[test]
    fn realistic_has_punctuation() {
        let sets = load();
        let s = sets.iter().find(|s| s.kind == SetKind::Realistic).unwrap();
        let t = practice_text(s, 60, &mut rand::thread_rng());
        assert!(t.chars().next().unwrap().is_uppercase());
        assert!(t.ends_with('.') || t.ends_with('?') || t.ends_with('!'));
    }

    #[test]
    fn weak_text_targets_letters() {
        let words: Vec<String> = ["ağaç", "dağ", "ev", "göğüs", "kalem"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let t = weak_text(&words, &['ğ'], 20, &mut rand::thread_rng());
        assert!(t.split_whitespace().all(|w| w.contains('ğ')));
    }
}
