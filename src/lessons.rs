//! TypingClub tarzı dersler: tuşlar parmak parmak, sıra sıra açılır.
//!
//! Dersler tuş konumlarıyla (satır, sütun) tanımlanır; böylece aynı plan hem
//! F hem Q düzeninde kendi harfleriyle çalışır.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::layout::Layout;
use crate::text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LessonKind {
    Keys,
    Shift,
    Digits,
    Punct,
    Proverbs,
}

#[derive(Clone, Debug)]
pub struct Lesson {
    pub title: &'static str,
    /// Bu derste tanışılan tuşlar.
    pub fresh: Vec<char>,
    /// Bu derse kadar açılmış tüm tuşlar.
    pub unlocked: Vec<char>,
    pub kind: LessonKind,
}

pub const STEP_COUNT: usize = 3;

pub fn step_names(kind: LessonKind) -> [&'static str; STEP_COUNT] {
    match kind {
        LessonKind::Keys => ["Tanışma", "Karışık", "Kelimeler"],
        _ => ["Bölüm 1", "Bölüm 2", "Bölüm 3"],
    }
}

pub fn about(lesson: &Lesson) -> &'static str {
    match lesson.kind {
        LessonKind::Keys if lesson.title.starts_with("Ana sıra") => {
            "Yeni tuş yok. Ana sırayı ekrana bakmadan, ritimle pekiştir."
        }
        LessonKind::Keys => {
            "Parmakların ana sırada dursun; yalnızca gereken parmağı uzat ve hemen geri dön."
        }
        LessonKind::Shift => {
            "Büyük harf için karşı eldeki Shift: sol eldeki harfe sağ Shift, sağdakine sol Shift."
        }
        LessonKind::Digits => {
            "Rakamlar en üst sırada. Sütunun parmağıyla uzan, sonra ana sıraya dön."
        }
        LessonKind::Punct => "Nokta, virgül, soru ve ünlem: gerçek cümlelerle noktalama.",
        LessonKind::Proverbs => "Final: atasözleriyle her şey bir arada.",
    }
}

fn plan() -> Vec<(&'static str, Vec<(usize, usize)>)> {
    vec![
        ("İşaret parmakları", vec![(2, 3), (2, 6)]),
        ("Orta parmaklar", vec![(2, 2), (2, 7)]),
        ("Yüzük parmakları", vec![(2, 1), (2, 8)]),
        ("Serçe parmakları", vec![(2, 0), (2, 9)]),
        ("Ana sıra tekrarı", vec![]),
        ("İşaret uzanışı", vec![(2, 4), (2, 5)]),
        ("Üst sıra: işaret", vec![(1, 3), (1, 4), (1, 5), (1, 6)]),
        ("Üst sıra: orta ve yüzük", vec![(1, 2), (1, 7), (1, 1), (1, 8)]),
        ("Üst sıra: serçe", vec![(1, 0), (1, 9), (1, 10), (1, 11)]),
        ("Alt sıra: işaret", vec![(3, 4), (3, 5), (3, 6), (3, 7)]),
        ("Alt sıra: orta ve yüzük", vec![(3, 3), (3, 8), (3, 2), (3, 9)]),
        ("Kalan tuşlar", vec![(3, 1), (3, 10), (2, 10), (2, 11)]),
    ]
}

pub fn build(layout: &Layout) -> Vec<Lesson> {
    let mut unlocked: Vec<char> = Vec::new();
    let mut out = Vec::new();
    for (title, keys) in plan() {
        let mut fresh = Vec::new();
        for (r, c) in keys {
            if let Some(k) = layout.key_at(r, c) {
                fresh.push(k.base);
                if !unlocked.contains(&k.base) {
                    unlocked.push(k.base);
                }
            }
        }
        out.push(Lesson {
            title,
            fresh,
            unlocked: unlocked.clone(),
            kind: LessonKind::Keys,
        });
    }

    let digits: Vec<char> = "1234567890".chars().collect();
    let mut with_digits = unlocked.clone();
    with_digits.extend(digits.iter().copied());
    let extras = [
        ("Büyük harfler", LessonKind::Shift, Vec::new()),
        ("Rakamlar", LessonKind::Digits, digits),
        ("Noktalama", LessonKind::Punct, vec!['.', ',']),
        ("Atasözleri", LessonKind::Proverbs, Vec::new()),
    ];
    for (title, kind, fresh) in extras {
        let unlocked = if kind == LessonKind::Shift {
            unlocked.clone()
        } else {
            with_digits.clone()
        };
        out.push(Lesson {
            title,
            fresh,
            unlocked,
            kind,
        });
    }
    out
}

/// Doğruluk ve hızdan yıldız: 0 = tekrar, 3 = kusursuz.
pub fn stars(acc: f64, wpm: f64) -> u8 {
    if acc < 90.0 {
        0
    } else if acc >= 97.0 && wpm >= 25.0 {
        3
    } else if acc >= 94.0 && wpm >= 15.0 {
        2
    } else {
        1
    }
}

pub fn generate<R: Rng>(
    lesson: &Lesson,
    step: usize,
    words: &[String],
    proverbs: &[String],
    rng: &mut R,
) -> String {
    let letters: Vec<char> = lesson
        .unlocked
        .iter()
        .copied()
        .filter(|c| c.is_alphabetic())
        .collect();
    let fresh: Vec<char> = if lesson.fresh.is_empty() {
        letters.clone()
    } else {
        lesson.fresh.clone()
    };
    match lesson.kind {
        LessonKind::Keys => match step {
            0 => intro(&fresh, &letters, rng),
            1 => pseudo(&letters, &fresh, 16, rng),
            _ => match real_words(&letters, &fresh, words, 14, rng) {
                Some(t) => t,
                None => pseudo(&letters, &fresh, 14, rng),
            },
        },
        LessonKind::Shift => shift_text(words, step, rng),
        LessonKind::Digits => digits_text(words, step, rng),
        LessonKind::Punct => sentences(proverbs, 3, rng),
        LessonKind::Proverbs => sentences(proverbs, 4, rng),
    }
}

fn pick<R: Rng>(fresh: &[char], letters: &[char], p_fresh: f64, rng: &mut R) -> char {
    let pool = if letters.is_empty() || fresh.is_empty() {
        if fresh.is_empty() {
            letters
        } else {
            fresh
        }
    } else if rng.gen_bool(p_fresh) {
        fresh
    } else {
        letters
    };
    pool.choose(rng).copied().unwrap_or('a')
}

/// Yeni tuşlarla tanışma: üçlü tekrarlar, gidip gelmeler, kısa kombinasyonlar.
fn intro<R: Rng>(fresh: &[char], letters: &[char], rng: &mut R) -> String {
    let mut words: Vec<String> = Vec::new();
    for &c in fresh {
        words.push([c, c, c].iter().collect());
    }
    for &c in fresh {
        for &d in fresh {
            if c != d && words.len() < 14 {
                words.push([c, d, c].iter().collect());
            }
        }
    }
    while words.len() < 20 {
        let len = rng.gen_range(2..=4);
        let mut w = String::new();
        for _ in 0..len {
            w.push(pick(fresh, letters, 0.7, rng));
        }
        words.push(w);
    }
    words.join(" ")
}

/// Açık harflerden anlamsız ama telaffuz edilebilir uzunlukta kelimeler.
fn pseudo<R: Rng>(letters: &[char], fresh: &[char], n: usize, rng: &mut R) -> String {
    let fresh_letters: Vec<char> = fresh.iter().copied().filter(|c| c.is_alphabetic()).collect();
    let fresh_use: Vec<char> = if fresh_letters.is_empty() {
        fresh.to_vec()
    } else {
        fresh_letters
    };
    let mut out: Vec<String> = Vec::with_capacity(n);
    for _ in 0..n {
        let len = rng.gen_range(3..=6);
        let mut w = String::new();
        for _ in 0..len {
            w.push(pick(&fresh_use, letters, 0.4, rng));
        }
        out.push(w);
    }
    out.join(" ")
}

/// Yalnızca açık harflerden oluşan gerçek kelimeler (yeni tuşu içerenler öncelikli).
fn real_words<R: Rng>(
    letters: &[char],
    fresh: &[char],
    words: &[String],
    n: usize,
    rng: &mut R,
) -> Option<String> {
    let ok: Vec<&String> = words
        .iter()
        .filter(|w| w.chars().count() >= 2 && w.chars().all(|c| letters.contains(&c)))
        .collect();
    if ok.len() < 6 {
        return None;
    }
    let preferred: Vec<&String> = ok
        .iter()
        .copied()
        .filter(|w| w.chars().any(|c| fresh.contains(&c)))
        .collect();
    let pool = if preferred.len() >= 6 { preferred } else { ok };
    let mut out: Vec<String> = Vec::with_capacity(n);
    for _ in 0..n {
        if let Some(w) = pool.choose(rng) {
            out.push(w.to_string());
        }
    }
    Some(out.join(" "))
}

fn shift_text<R: Rng>(words: &[String], step: usize, rng: &mut R) -> String {
    let pool: Vec<&String> = words
        .iter()
        .filter(|w| (3..=8).contains(&w.chars().count()))
        .collect();
    let mut out: Vec<String> = Vec::new();
    for i in 0..14 {
        let Some(w) = pool.choose(rng) else { break };
        let capital = match step {
            0 => true,
            1 => i % 2 == 0,
            _ => rng.gen_bool(0.5),
        };
        out.push(if capital {
            text::capitalize(w)
        } else {
            w.to_string()
        });
    }
    out.join(" ")
}

fn digits_text<R: Rng>(words: &[String], step: usize, rng: &mut R) -> String {
    let mut out: Vec<String> = Vec::new();
    for i in 0..14 {
        if i % 3 == 2 {
            if let Some(w) = words.choose(rng) {
                out.push(w.clone());
                continue;
            }
        }
        let n: u32 = match step {
            0 => rng.gen_range(0..100),
            1 => rng.gen_range(100..10_000),
            _ => rng.gen_range(0..100_000),
        };
        out.push(n.to_string());
    }
    out.join(" ")
}

fn sentences<R: Rng>(proverbs: &[String], n: usize, rng: &mut R) -> String {
    proverbs
        .choose_multiple(rng, n)
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use crate::layout::LayoutKind;

    #[test]
    fn sixteen_lessons_on_both_layouts() {
        for kind in [LayoutKind::F, LayoutKind::Q] {
            let l = build(&Layout::new(kind));
            assert_eq!(l.len(), 16);
            assert_eq!(l[0].fresh.len(), 2);
            assert_eq!(l[4].fresh.len(), 0);
        }
        let f = build(&Layout::new(LayoutKind::F));
        assert_eq!(f[0].fresh, vec!['a', 'k']);
        let q = build(&Layout::new(LayoutKind::Q));
        assert_eq!(q[0].fresh, vec!['f', 'j']);
    }

    #[test]
    fn lessons_only_use_unlocked_keys() {
        let sets = data::load();
        let words = data::lesson_words(&sets);
        let proverbs = data::proverbs(&sets);
        let mut rng = rand::thread_rng();
        for kind in [LayoutKind::F, LayoutKind::Q] {
            let lessons = build(&Layout::new(kind));
            for lesson in lessons.iter().filter(|l| l.kind == LessonKind::Keys) {
                for step in 0..STEP_COUNT {
                    for _ in 0..5 {
                        let t = generate(lesson, step, &words, &proverbs, &mut rng);
                        assert!(!t.trim().is_empty());
                        for c in t.chars() {
                            assert!(
                                c == ' ' || lesson.unlocked.contains(&c),
                                "{:?} {} adım {}: {c:?} açık değil ({t})",
                                kind,
                                lesson.title,
                                step
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn later_lessons_use_real_words() {
        let sets = data::load();
        let words = data::lesson_words(&sets);
        let lessons = build(&Layout::new(LayoutKind::F));
        let mut rng = rand::thread_rng();
        let t = generate(&lessons[8], 2, &words, &[], &mut rng);
        for w in t.split_whitespace() {
            assert!(words.iter().any(|x| x == w), "{w} gerçek kelime değil");
        }
    }

    #[test]
    fn star_rules() {
        assert_eq!(stars(85.0, 60.0), 0);
        assert_eq!(stars(92.0, 10.0), 1);
        assert_eq!(stars(95.0, 20.0), 2);
        assert_eq!(stars(99.0, 30.0), 3);
    }
}
