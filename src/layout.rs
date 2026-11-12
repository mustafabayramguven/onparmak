//! Türkçe F ve Türkçe Q klavye düzenleri, parmak eşlemesiyle birlikte.

use serde::{Deserialize, Serialize};

use crate::text::upper_char;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutKind {
    #[default]
    F,
    Q,
}

impl LayoutKind {
    pub fn name(self) -> &'static str {
        match self {
            LayoutKind::F => "Türkçe F",
            LayoutKind::Q => "Türkçe Q",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            LayoutKind::F => "F",
            LayoutKind::Q => "Q",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            LayoutKind::F => LayoutKind::Q,
            LayoutKind::Q => LayoutKind::F,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    Thumb,
}

impl Finger {
    pub fn name(self) -> &'static str {
        match self {
            Finger::LeftPinky => "sol serçe parmak",
            Finger::LeftRing => "sol yüzük parmağı",
            Finger::LeftMiddle => "sol orta parmak",
            Finger::LeftIndex => "sol işaret parmağı",
            Finger::RightIndex => "sağ işaret parmağı",
            Finger::RightMiddle => "sağ orta parmak",
            Finger::RightRing => "sağ yüzük parmağı",
            Finger::RightPinky => "sağ serçe parmak",
            Finger::Thumb => "başparmak",
        }
    }

    pub fn is_left(self) -> bool {
        matches!(
            self,
            Finger::LeftPinky | Finger::LeftRing | Finger::LeftMiddle | Finger::LeftIndex
        )
    }

    /// Renk paletindeki yeri: 0 serçe, 1 yüzük, 2 orta, 3 işaret, 4 başparmak.
    pub fn kind(self) -> usize {
        match self {
            Finger::LeftPinky | Finger::RightPinky => 0,
            Finger::LeftRing | Finger::RightRing => 1,
            Finger::LeftMiddle | Finger::RightMiddle => 2,
            Finger::LeftIndex | Finger::RightIndex => 3,
            Finger::Thumb => 4,
        }
    }

    /// Bu parmağın tuşu büyük harf için hangi Shift'i ister.
    pub fn shift_name(self) -> &'static str {
        if self.is_left() {
            "sağ Shift"
        } else {
            "sol Shift"
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub base: char,
    pub shift: char,
    pub finger: Finger,
    pub row: usize,
    pub col: usize,
}

pub const SPACE: Key = Key {
    base: ' ',
    shift: ' ',
    finger: Finger::Thumb,
    row: 4,
    col: 0,
};

pub struct Layout {
    pub kind: LayoutKind,
    pub rows: Vec<Vec<Key>>,
}

fn row_fingers(row: usize) -> Vec<Finger> {
    use self::Finger::*;
    match row {
        0 => vec![
            LeftPinky,
            LeftPinky,
            LeftRing,
            LeftMiddle,
            LeftIndex,
            LeftIndex,
            RightIndex,
            RightIndex,
            RightMiddle,
            RightRing,
            RightPinky,
            RightPinky,
            RightPinky,
        ],
        1 | 2 => vec![
            LeftPinky,
            LeftRing,
            LeftMiddle,
            LeftIndex,
            LeftIndex,
            RightIndex,
            RightIndex,
            RightMiddle,
            RightRing,
            RightPinky,
            RightPinky,
            RightPinky,
        ],
        _ => vec![
            LeftPinky,
            LeftPinky,
            LeftRing,
            LeftMiddle,
            LeftIndex,
            LeftIndex,
            RightIndex,
            RightIndex,
            RightMiddle,
            RightRing,
            RightPinky,
        ],
    }
}

fn letters(s: &str) -> Vec<(char, char)> {
    s.chars().map(|c| (c, upper_char(c))).collect()
}

fn spec(kind: LayoutKind) -> Vec<Vec<(char, char)>> {
    match kind {
        LayoutKind::F => {
            let row0 = vec![
                ('+', '*'),
                ('1', '!'),
                ('2', '"'),
                ('3', '^'),
                ('4', '$'),
                ('5', '%'),
                ('6', '&'),
                ('7', '\''),
                ('8', '('),
                ('9', ')'),
                ('0', '='),
                ('/', '?'),
                ('-', '_'),
            ];
            let row1 = letters("fgğıodrnhpqw");
            let row2 = letters("uieaütkmlyşx");
            let mut row3 = vec![('<', '>')];
            row3.extend(letters("jövcçzsb"));
            row3.push(('.', ':'));
            row3.push((',', ';'));
            vec![row0, row1, row2, row3]
        }
        LayoutKind::Q => {
            let row0 = vec![
                ('"', 'é'),
                ('1', '!'),
                ('2', '\''),
                ('3', '^'),
                ('4', '+'),
                ('5', '%'),
                ('6', '&'),
                ('7', '/'),
                ('8', '('),
                ('9', ')'),
                ('0', '='),
                ('*', '?'),
                ('-', '_'),
            ];
            let row1 = letters("qwertyuıopğü");
            let mut row2 = letters("asdfghjklşi");
            row2.push((',', ';'));
            let mut row3 = vec![('<', '>')];
            row3.extend(letters("zxcvbnmöç"));
            row3.push(('.', ':'));
            vec![row0, row1, row2, row3]
        }
    }
}

impl Layout {
    pub fn new(kind: LayoutKind) -> Self {
        let rows = spec(kind)
            .into_iter()
            .enumerate()
            .map(|(r, row)| {
                let fingers = row_fingers(r);
                row.into_iter()
                    .enumerate()
                    .map(|(c, (base, shift))| Key {
                        base,
                        shift,
                        finger: fingers[c.min(fingers.len() - 1)],
                        row: r,
                        col: c,
                    })
                    .collect()
            })
            .collect();
        Layout { kind, rows }
    }

    /// Karakteri üreten tuşu ve Shift gerekip gerekmediğini bulur.
    pub fn find(&self, ch: char) -> Option<(Key, bool)> {
        if ch == ' ' {
            return Some((SPACE, false));
        }
        for row in &self.rows {
            for k in row {
                if k.base == ch {
                    return Some((*k, false));
                }
            }
        }
        for row in &self.rows {
            for k in row {
                if k.shift == ch {
                    return Some((*k, true));
                }
            }
        }
        None
    }

    pub fn key_at(&self, row: usize, col: usize) -> Option<Key> {
        self.rows.get(row)?.get(col).copied()
    }

    /// İşaret parmaklarının çıkıntılı ana tuşları (F'de A/K, Q'da F/J).
    pub fn is_home_anchor(&self, k: &Key) -> bool {
        k.row == 2 && (k.col == 3 || k.col == 6)
    }

    /// Ana sıradaki sekiz tuş.
    pub fn home_keys(&self) -> Vec<char> {
        [0, 1, 2, 3, 6, 7, 8, 9]
            .iter()
            .filter_map(|&c| self.key_at(2, c).map(|k| k.base))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f_layout() {
        let l = Layout::new(LayoutKind::F);
        assert_eq!(l.home_keys(), vec!['u', 'i', 'e', 'a', 'k', 'm', 'l', 'y']);
        let (k, shifted) = l.find('ğ').unwrap();
        assert_eq!((k.row, k.col, shifted), (1, 2, false));
        assert_eq!(k.finger, Finger::LeftMiddle);
        let (k, shifted) = l.find('İ').unwrap();
        assert_eq!(k.base, 'i');
        assert!(shifted);
        let (k, _) = l.find('a').unwrap();
        assert!(l.is_home_anchor(&k));
        assert_eq!(l.find('?').unwrap().0.base, '/');
    }

    #[test]
    fn q_layout() {
        let l = Layout::new(LayoutKind::Q);
        assert_eq!(l.home_keys(), vec!['a', 's', 'd', 'f', 'j', 'k', 'l', 'ş']);
        let (k, _) = l.find('i').unwrap();
        assert_eq!(k.finger, Finger::RightPinky);
        let (k, _) = l.find('ç').unwrap();
        assert_eq!(k.finger, Finger::RightRing);
        assert!(l.find('j').map(|(k, _)| l.is_home_anchor(&k)).unwrap());
    }

    #[test]
    fn every_turkish_letter_is_typable() {
        for kind in [LayoutKind::F, LayoutKind::Q] {
            let l = Layout::new(kind);
            for c in "abcçdefgğhıijklmnoöprsştuüvyzABCÇDEFGĞHIİJKLMNOÖPRSŞTUÜVYZ0123456789.,?!:;'-"
                .chars()
            {
                assert!(l.find(c).is_some(), "{:?} düzeninde {c:?} yok", kind);
            }
        }
    }
}
