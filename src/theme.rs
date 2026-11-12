//! Renk temaları. Omarchy kuruluysa aktif sistem temasını otomatik kullanır.

use std::collections::HashMap;

use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub key_bg: Color,
    pub accent: Color,
    pub ok: Color,
    pub err: Color,
    pub warn: Color,
    pub ghost: Color,
    pub on_accent: Color,
    /// serçe, yüzük, orta, işaret, başparmak
    pub fingers: [Color; 5],
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

/// İki rengi karıştırır (0 = a, 1 = b). RGB olmayan renklerde en yakını seçer.
pub fn mix(a: Color, b: Color, x: f64) -> Color {
    let x = x.clamp(0.0, 1.0);
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let l = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * x).round() as u8;
            Color::Rgb(l(r1, r2), l(g1, g2), l(b1, b2))
        }
        _ => {
            if x < 0.5 {
                a
            } else {
                b
            }
        }
    }
}

/// 0 (iyi) → 1 (kötü) arası ısı rengi.
pub fn heat(t: &Theme, v: f64) -> Color {
    if v < 0.5 {
        mix(t.ok, t.warn, v * 2.0)
    } else {
        mix(t.warn, t.err, (v - 0.5) * 2.0)
    }
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_start_matches('#');
    if s.len() != 6 || !s.is_ascii() {
        return None;
    }
    u32::from_str_radix(s, 16).ok().map(rgb)
}

fn omarchy() -> Option<Theme> {
    let dir = dirs::home_dir()?.join(".local/state/omarchy/current");
    let raw = std::fs::read_to_string(dir.join("theme/colors.toml")).ok()?;
    let mut c: HashMap<String, Color> = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if let Some(col) = parse_hex(v) {
                c.insert(k.trim().to_string(), col);
            }
        }
    }
    let get = |k: &str| c.get(k).copied();
    let fg = get("foreground")?;
    let bg = get("background").unwrap_or(Color::Black);
    let accent = get("accent").or_else(|| get("blue")).unwrap_or(fg);
    let muted = get("muted")
        .or_else(|| get("dark_foreground"))
        .unwrap_or_else(|| mix(fg, bg, 0.6));
    let light = get("light_foreground").unwrap_or_else(|| mix(fg, bg, 0.3));
    let name = std::fs::read_to_string(dir.join("theme.name"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "aktif tema".to_string());
    Some(Theme {
        name: format!("Omarchy · {name}"),
        fg,
        dim: mix(light, muted, 0.45),
        faint: muted,
        key_bg: get("lighter_background")
            .or_else(|| get("selection"))
            .unwrap_or_else(|| mix(bg, fg, 0.08)),
        accent,
        ok: get("green").unwrap_or(accent),
        err: get("red").unwrap_or(Color::Red),
        warn: get("yellow").unwrap_or(Color::Yellow),
        ghost: get("magenta").unwrap_or(accent),
        on_accent: bg,
        fingers: [
            get("magenta").unwrap_or(accent),
            get("blue").unwrap_or(accent),
            get("cyan").unwrap_or(accent),
            get("yellow").unwrap_or(accent),
            muted,
        ],
    })
}

fn orman() -> Theme {
    Theme {
        name: "Orman".into(),
        fg: rgb(0xcfd6c4),
        dim: rgb(0x6f7f74),
        faint: rgb(0x3e4d45),
        key_bg: rgb(0x1a2622),
        accent: rgb(0x8fb88a),
        ok: rgb(0xa8cc9f),
        err: rgb(0xd4806e),
        warn: rgb(0xd8c48a),
        ghost: rgb(0xb994a8),
        on_accent: rgb(0x111a18),
        fingers: [
            rgb(0xb994a8),
            rgb(0x7fa3b0),
            rgb(0x7fb5a6),
            rgb(0xd8c48a),
            rgb(0x6f7f74),
        ],
    }
}

fn gece() -> Theme {
    Theme {
        name: "Gece".into(),
        fg: rgb(0xc0caf5),
        dim: rgb(0x565f89),
        faint: rgb(0x3b4261),
        key_bg: rgb(0x24283b),
        accent: rgb(0x7aa2f7),
        ok: rgb(0x9ece6a),
        err: rgb(0xf7768e),
        warn: rgb(0xe0af68),
        ghost: rgb(0xbb9af7),
        on_accent: rgb(0x1a1b26),
        fingers: [
            rgb(0xbb9af7),
            rgb(0x7aa2f7),
            rgb(0x7dcfff),
            rgb(0xe0af68),
            rgb(0x565f89),
        ],
    }
}

fn kehribar() -> Theme {
    Theme {
        name: "Kehribar".into(),
        fg: rgb(0xebdbb2),
        dim: rgb(0x7c6f64),
        faint: rgb(0x504945),
        key_bg: rgb(0x3c3836),
        accent: rgb(0xfabd2f),
        ok: rgb(0xb8bb26),
        err: rgb(0xfb4934),
        warn: rgb(0xfe8019),
        ghost: rgb(0xd3869b),
        on_accent: rgb(0x282828),
        fingers: [
            rgb(0xd3869b),
            rgb(0x83a598),
            rgb(0x8ec07c),
            rgb(0xfabd2f),
            rgb(0x7c6f64),
        ],
    }
}

pub fn all() -> Vec<Theme> {
    let mut v = Vec::new();
    if let Some(t) = omarchy() {
        v.push(t);
    }
    v.push(orman());
    v.push(gece());
    v.push(kehribar());
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(parse_hex(" \"#8fb88a\""), Some(Color::Rgb(0x8f, 0xb8, 0x8a)));
        assert_eq!(parse_hex("\"dark\""), None);
        assert_eq!(parse_hex("#fff"), None);
    }

    #[test]
    fn mixing() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(200, 100, 50);
        assert_eq!(mix(a, b, 0.5), Color::Rgb(100, 50, 25));
        assert_eq!(mix(a, b, 2.0), b);
    }
}
