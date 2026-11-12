//! Türkçe büyük/küçük harf dönüşümleri ve metin kaydırma.

/// Türkçe kurallarıyla büyük harf: i → İ, ı → I.
pub fn upper_char(c: char) -> char {
    match c {
        'i' => 'İ',
        'ı' => 'I',
        _ => {
            let mut up = c.to_uppercase();
            match (up.next(), up.next()) {
                (Some(u), None) => u,
                _ => c,
            }
        }
    }
}

/// Türkçe kurallarıyla küçük harf: İ → i, I → ı.
pub fn lower_char(c: char) -> char {
    match c {
        'İ' => 'i',
        'I' => 'ı',
        _ => {
            let mut low = c.to_lowercase();
            match (low.next(), low.next()) {
                (Some(l), None) => l,
                _ => c,
            }
        }
    }
}

/// İlk harfi büyütür.
pub fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => {
            let mut out = String::with_capacity(word.len() + 1);
            out.push(upper_char(first));
            out.extend(chars);
            out
        }
        None => String::new(),
    }
}

/// Tuş üzerinde gösterilecek etiket.
pub fn key_label(c: char) -> char {
    if c.is_alphabetic() {
        upper_char(c)
    } else {
        c
    }
}

/// Metni en fazla `width` karakterlik satırlara böler; boşluklardan sonra keser.
/// Her satır `(başlangıç, bitiş)` karakter aralığıdır ve aralıklar metni boşluksuz kaplar.
pub fn wrap(text: &[char], width: usize) -> Vec<(usize, usize)> {
    let width = width.max(8);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut last_space: Option<usize> = None;
    let mut i = 0;
    while i < text.len() {
        if text[i] == ' ' {
            last_space = Some(i);
        }
        if i - start >= width {
            match last_space.filter(|&s| s >= start) {
                Some(sp) => {
                    lines.push((start, sp + 1));
                    start = sp + 1;
                }
                None => {
                    lines.push((start, i));
                    start = i;
                }
            }
            last_space = None;
            continue;
        }
        i += 1;
    }
    if start < text.len() || lines.is_empty() {
        lines.push((start, text.len()));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turkish_case() {
        assert_eq!(upper_char('i'), 'İ');
        assert_eq!(upper_char('ı'), 'I');
        assert_eq!(upper_char('ş'), 'Ş');
        assert_eq!(lower_char('İ'), 'i');
        assert_eq!(lower_char('I'), 'ı');
        assert_eq!(lower_char('Ğ'), 'ğ');
        assert_eq!(capitalize("istanbul"), "İstanbul");
        assert_eq!(capitalize("ılık"), "Ilık");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn wrap_covers_everything() {
        let text: Vec<char> = "bir iki üç dört beş altı yedi sekiz dokuz on on bir on iki"
            .chars()
            .collect();
        for width in [8, 10, 13, 20, 80] {
            let lines = wrap(&text, width);
            assert_eq!(lines[0].0, 0);
            assert_eq!(lines.last().unwrap().1, text.len());
            for pair in lines.windows(2) {
                assert_eq!(pair[0].1, pair[1].0);
            }
            for (s, e) in &lines {
                assert!(e - s <= width + 1, "satır çok uzun: {} > {}", e - s, width + 1);
            }
        }
    }

    #[test]
    fn wrap_long_word() {
        let text: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        let lines = wrap(&text, 10);
        assert_eq!(lines, vec![(0, 10), (10, 20), (20, 26)]);
    }

    #[test]
    fn wrap_empty() {
        assert_eq!(wrap(&[], 20), vec![(0, 0)]);
    }
}
