//! Tüm ekranların çizimi.

use std::collections::HashMap;

use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Alignment, Layout as Split, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, BorderType, Chart, Clear, Dataset, GraphType, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Screen, MENU, MODES};
use crate::data::SetKind;
use crate::layout::Layout;
use crate::lessons;
use crate::state::State;
use crate::text::{self, key_label};
use crate::theme::{self, Theme};

const LOGO: [&str; 3] = [
    "┌─┐┌┐┌┌─┐┌─┐┬─┐┌┬┐┌─┐┬┌─",
    "│ ││││├─┘├─┤├┬┘│││├─┤├┴┐",
    "└─┘┘└┘┴  ┴ ┴┴└─┴ ┴┴ ┴┴ ┴",
];

const DIGITS: [[&str; 5]; 10] = [
    ["███", "█ █", "█ █", "█ █", "███"],
    [" █ ", "██ ", " █ ", " █ ", "███"],
    ["███", "  █", "███", "█  ", "███"],
    ["███", "  █", "███", "  █", "███"],
    ["█ █", "█ █", "███", "  █", "  █"],
    ["███", "█  ", "███", "  █", "███"],
    ["███", "█  ", "███", "█ █", "███"],
    ["███", "  █", "  █", "  █", "  █"],
    ["███", "█ █", "███", "█ █", "███"],
    ["███", "█ █", "███", "  █", "███"],
];

const BARS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

// ---- küçük yardımcılar ----------------------------------------------------

fn sp(text: impl Into<String>, style: Style) -> Span<'static> {
    Span::styled(text.into(), style)
}

fn fg(c: Color) -> Style {
    Style::new().fg(c)
}

fn bold(c: Color) -> Style {
    Style::new().fg(c).add_modifier(Modifier::BOLD)
}

fn chip(c: Color, t: &Theme) -> Style {
    Style::new()
        .fg(t.on_accent)
        .bg(c)
        .add_modifier(Modifier::BOLD)
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

fn mmss(secs: f64) -> String {
    let s = secs.max(0.0).round() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

fn stars_spans(n: u8, t: &Theme) -> Vec<Span<'static>> {
    (0..3u8)
        .map(|i| {
            if i < n {
                sp("★", fg(t.warn))
            } else {
                sp("☆", fg(t.faint))
            }
        })
        .collect()
}

fn big_number(n: &str, color: Color) -> Vec<Line<'static>> {
    (0..5)
        .map(|row| {
            let mut s = String::new();
            for ch in n.chars() {
                if let Some(d) = ch.to_digit(10) {
                    for g in DIGITS[d as usize][row].chars() {
                        s.push_str(if g == '█' { "██" } else { "  " });
                    }
                    s.push_str("  ");
                }
            }
            Line::from(sp(s, fg(color)))
        })
        .collect()
}

// ---- ekran klavyesi ---------------------------------------------------------

pub enum KbMode<'a> {
    /// Yazarken: sıradaki tuş parmak rengiyle yanar, yanlış tuş kırmızı.
    Next {
        next: Option<char>,
        wrong: Option<char>,
        colored: bool,
    },
    /// Ders önizlemesi: yeni tuşlar dolu, açık tuşlar renkli, kalanlar sönük.
    Lesson { unlocked: &'a [char], fresh: &'a [char] },
    /// İstatistik: hata oranına göre ısı haritası (0..1).
    Heat(&'a HashMap<char, f64>),
}

pub const KB_WIDTH: u16 = 56;

fn keyboard_lines(layout: &Layout, t: &Theme, mode: &KbMode) -> Vec<Line<'static>> {
    let indents = [0usize, 6, 7, 5];
    let next_key = match mode {
        KbMode::Next { next: Some(c), .. } => layout.find(*c).map(|(k, _)| k),
        _ => None,
    };
    let wrong_key = match mode {
        KbMode::Next { wrong: Some(c), .. } => layout.find(*c).map(|(k, _)| k),
        _ => None,
    };
    let mut lines = Vec::new();
    for (r, row) in layout.rows.iter().enumerate() {
        let mut spans = vec![Span::raw(" ".repeat(indents[r.min(3)]))];
        for k in row {
            let fcol = t.fingers[k.finger.kind()];
            let same = |o: Option<crate::layout::Key>| o.is_some_and(|o| o.row == k.row && o.col == k.col);
            let mut style = Style::new().fg(t.dim).bg(t.key_bg);
            match mode {
                KbMode::Next { colored, .. } => {
                    if *colored {
                        style = style.fg(fcol);
                    }
                    if same(wrong_key) {
                        style = chip(t.err, t);
                    } else if same(next_key) {
                        style = chip(fcol, t);
                    }
                }
                KbMode::Lesson { unlocked, fresh } => {
                    style = if fresh.contains(&k.base) {
                        chip(fcol, t)
                    } else if unlocked.contains(&k.base) {
                        Style::new().fg(fcol).bg(t.key_bg)
                    } else {
                        Style::new().fg(t.faint).bg(t.key_bg)
                    };
                }
                KbMode::Heat(map) => {
                    style = match map.get(&k.base) {
                        Some(v) => Style::new().fg(t.on_accent).bg(theme::heat(t, *v)),
                        None => Style::new().fg(t.faint).bg(t.key_bg),
                    };
                }
            }
            if layout.is_home_anchor(k) {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            spans.push(sp(format!(" {} ", key_label(k.base)), style));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }
    let space_next = matches!(mode, KbMode::Next { next: Some(' '), .. });
    let space_style = if space_next {
        chip(t.fingers[4], t)
    } else {
        Style::new().fg(t.faint).bg(t.key_bg)
    };
    lines.push(Line::from(vec![
        Span::raw(" ".repeat(16)),
        sp(format!("{:^23}", "boşluk"), space_style),
    ]));
    lines
}

fn finger_legend(t: &Theme) -> Line<'static> {
    let names = ["serçe", "yüzük", "orta", "işaret", "başparmak"];
    let mut v = Vec::new();
    for (i, n) in names.iter().enumerate() {
        v.push(sp("■ ", fg(t.fingers[i])));
        v.push(sp(format!("{n}   "), fg(t.dim)));
    }
    Line::from(v).alignment(Alignment::Center)
}

fn hint_line(layout: &Layout, t: &Theme, next: Option<char>) -> Line<'static> {
    let Some(c) = next else {
        return Line::from(sp("bitti", fg(t.dim))).alignment(Alignment::Center);
    };
    match layout.find(c) {
        Some((k, shifted)) => {
            let col = t.fingers[k.finger.kind()];
            let shown = if c == ' ' {
                "boşluk".to_string()
            } else {
                c.to_string()
            };
            let mut v = vec![
                sp("sıradaki  ", fg(t.dim)),
                sp(format!(" {shown} "), chip(col, t)),
                sp("  ", fg(t.dim)),
                sp(k.finger.name(), fg(col)),
            ];
            if shifted {
                v.push(sp(format!("  +  ⇧ {}", k.finger.shift_name()), fg(t.warn)));
            }
            Line::from(v).alignment(Alignment::Center)
        }
        None => Line::from(vec![
            sp("sıradaki  ", fg(t.dim)),
            sp(format!(" {c} "), chip(t.accent, t)),
            sp("  (bu düzende AltGr gerekebilir)", fg(t.faint)),
        ])
        .alignment(Alignment::Center),
    }
}

// ---- ana çizim --------------------------------------------------------------

pub fn draw(f: &mut Frame, app: &App) {
    let t = app.theme();
    let area = f.area();
    let [top, mid, bot] = Split::vertical([Length(1), Min(1), Length(1)]).areas(area);
    draw_header(f, top, app, t);
    match app.screen {
        Screen::Home => draw_home(f, mid, app, t),
        Screen::Lessons => draw_lessons(f, mid, app, t),
        Screen::Practice => draw_practice(f, mid, app, t),
        Screen::Typing => draw_typing(f, mid, app, t),
        Screen::Results => draw_results(f, mid, app, t),
        Screen::Stats => draw_stats(f, mid, app, t),
        Screen::Settings => draw_settings(f, mid, app, t),
    }
    draw_footer(f, bot, app, t);
    if app.show_help {
        draw_help(f, area, t);
    }
}

fn screen_title(s: Screen) -> &'static str {
    match s {
        Screen::Home => "",
        Screen::Lessons => "dersler",
        Screen::Practice => "serbest pratik",
        Screen::Typing => "yazıyor",
        Screen::Results => "sonuç",
        Screen::Stats => "istatistikler",
        Screen::Settings => "ayarlar",
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let left = Line::from(vec![
        sp(" onparmak", bold(t.accent)),
        sp(format!("  {}", screen_title(app.screen)), fg(t.dim)),
    ]);
    let mut right = vec![sp(format!("⌨ {}", app.layout.kind.name()), fg(t.dim))];
    let streak = app.state.streak();
    if streak > 0 {
        right.push(sp(format!("   ✦ {streak} gün seri"), fg(t.warn)));
    }
    right.push(sp(" ", fg(t.dim)));
    f.render_widget(Paragraph::new(left), area);
    f.render_widget(
        Paragraph::new(Line::from(right)).alignment(Alignment::Right),
        area,
    );
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let line = if let Some(msg) = app.active_message() {
        Line::from(sp(msg.to_string(), bold(t.warn)))
    } else {
        let hint = match app.screen {
            Screen::Home => "↑↓ seç · enter aç · 1-6 kısayol · ? yardım · q çıkış",
            Screen::Lessons => "↑↓ ders · ←→ adım · enter başla · esc geri",
            Screen::Practice => "↑↓ set · ←→ mod · enter başla · esc geri",
            Screen::Typing => "tab yeniden · esc çık · ctrl+w kelime sil",
            Screen::Results => "enter devam · tab tekrar · esc geri",
            Screen::Stats => "esc geri",
            Screen::Settings => "↑↓ seç · ←→ değiştir · esc dön",
        };
        Line::from(sp(hint, fg(t.faint)))
    };
    f.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn draw_help(f: &mut Frame, area: Rect, t: &Theme) {
    let r = centered(area, 62, 20);
    f.render_widget(Clear, r);
    let row = |k: &str, d: &str| {
        Line::from(vec![
            sp(format!("  {k:<16}"), bold(t.accent)),
            sp(d.to_string(), fg(t.fg)),
        ])
    };
    let lines = vec![
        Line::from(""),
        row("↑ ↓  j k", "listede gezin"),
        row("← →  h l", "adım / mod / değer seç"),
        row("enter", "başla, devam et"),
        row("tab", "aynı alıştırmayı yeniden başlat"),
        row("esc", "geri dön"),
        row("backspace", "son karakteri sil"),
        row("ctrl+w", "son kelimeyi sil"),
        row("ctrl+c", "çık"),
        Line::from(""),
        Line::from(sp("  Derslerde yanlış tuş ilerlemez: doğrusunu bulana", fg(t.dim))),
        Line::from(sp("  kadar imleç bekler. %90 doğrulukla yıldız alırsın.", fg(t.dim))),
        Line::from(""),
        Line::from(sp("  wpm = dakikadaki kelime (5 karakter = 1 kelime)", fg(t.dim))),
        Line::from(sp("  ham = hatalar dahil hız · istikrar = ritmin dengesi", fg(t.dim))),
        Line::from(""),
        Line::from(sp("  herhangi bir tuşla kapat", fg(t.faint))).alignment(Alignment::Center),
    ];
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(t.accent))
        .title(Line::from(sp(" yardım ", bold(t.accent))));
    f.render_widget(Paragraph::new(lines).block(block), r);
}

// ---- ana menü ---------------------------------------------------------------

fn draw_home(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let rect = centered(area, 68, 21);
    let [logo_a, tag_a, _, menu_a, _, prov_a, _, info_a, _] = Split::vertical([
        Length(3),
        Length(1),
        Length(1),
        Length(6),
        Length(1),
        Length(4),
        Length(1),
        Length(1),
        Min(0),
    ])
    .areas(rect);

    let logo: Vec<Line> = LOGO
        .iter()
        .map(|l| Line::from(sp(*l, bold(t.accent))))
        .collect();
    f.render_widget(Paragraph::new(logo).alignment(Alignment::Center), logo_a);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            sp("on parmak, ", fg(t.dim)),
            sp("tek ritim", fg(t.fg)),
            sp("  ·  türkçe klavye eğitmeni", fg(t.dim)),
        ]))
        .alignment(Alignment::Center),
        tag_a,
    );

    let mut lines = Vec::new();
    for (i, (name, desc)) in MENU.iter().enumerate() {
        if i == app.menu {
            lines.push(Line::from(vec![
                sp(" ❯ ", bold(t.accent)),
                sp(format!("{name:<17}"), bold(t.accent)),
                sp(*desc, fg(t.dim)),
            ]));
        } else {
            lines.push(Line::from(vec![
                sp("   ", fg(t.dim)),
                sp(format!("{name:<17}"), fg(t.fg)),
                sp(*desc, fg(t.faint)),
            ]));
        }
    }
    f.render_widget(Paragraph::new(lines), centered(menu_a, 58, 6));

    if !app.proverbs.is_empty() {
        let p = &app.proverbs[(State::today() as usize) % app.proverbs.len()];
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(fg(t.faint))
            .title(Line::from(sp(" günün sözü ", fg(t.dim))).alignment(Alignment::Center));
        f.render_widget(
            Paragraph::new(Line::from(sp(
                p.clone(),
                fg(t.fg).add_modifier(Modifier::ITALIC),
            )))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(block),
            centered(prov_a, 64, 4),
        );
    }

    let hist = app.state.history_for(app.layout.kind);
    let info = if hist.is_empty() {
        Line::from(vec![
            sp("ilk adım: ", fg(t.dim)),
            sp("Dersler", bold(t.accent)),
            sp(" → İşaret parmakları", fg(t.dim)),
        ])
    } else {
        let today = State::today();
        let today_n = hist.iter().filter(|r| r.day == today).count();
        let best = hist.iter().map(|r| r.wpm).fold(0.0, f64::max);
        Line::from(vec![
            sp(format!("bugün {today_n} alıştırma"), fg(t.dim)),
            sp("  ·  ", fg(t.faint)),
            sp(format!("rekor {best:.0} wpm"), fg(t.dim)),
            sp("  ·  ", fg(t.faint)),
            sp(
                format!("{}/{} ders", app.lessons_done(), app.lessons.len()),
                fg(t.dim),
            ),
        ])
    };
    f.render_widget(Paragraph::new(info).alignment(Alignment::Center), info_a);
}

// ---- dersler ----------------------------------------------------------------

fn draw_lessons(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let rect = centered(area, 100, area.height.min(26));
    let [left, _, right] = Split::horizontal([Length(42), Length(2), Min(30)]).areas(rect);

    let mut lines = vec![
        Line::from(vec![
            sp("Dersler", bold(t.fg)),
            sp(format!("  ·  {}", app.layout.kind.name()), fg(t.dim)),
        ]),
        Line::from(""),
    ];
    for (i, l) in app.lessons.iter().enumerate() {
        let unlocked = app.lesson_unlocked(i);
        let sel = i == app.lesson_sel;
        let name_style = if !unlocked {
            fg(t.faint)
        } else if sel {
            bold(t.accent)
        } else {
            fg(t.fg)
        };
        let mut v = vec![
            sp(if sel { "❯ " } else { "  " }, bold(t.accent)),
            sp(format!("{:02}  ", i + 1), fg(t.faint)),
            sp(format!("{:<26}", l.title), name_style),
        ];
        if unlocked {
            v.extend(stars_spans(app.lesson_stars(i), t));
        } else {
            v.push(sp("kilitli", fg(t.faint)));
        }
        lines.push(Line::from(v));
    }
    let offset = (app.lesson_sel + 3).saturating_sub(left.height as usize) as u16;
    f.render_widget(Paragraph::new(lines).scroll((offset, 0)), left);

    let Some(l) = app.lessons.get(app.lesson_sel) else { return };
    let [info_a, kb_a] = Split::vertical([Min(8), Length(6)]).areas(right);
    let mut lines = vec![
        Line::from(vec![
            sp(format!("Ders {}  ", app.lesson_sel + 1), fg(t.dim)),
            sp(l.title, bold(t.fg)),
        ]),
        Line::from(""),
    ];
    if !l.fresh.is_empty() && l.fresh.len() <= 4 {
        let mut caps = vec![sp("yeni tuşlar   ", fg(t.dim))];
        for &c in &l.fresh {
            let col = app
                .layout
                .find(c)
                .map(|(k, _)| t.fingers[k.finger.kind()])
                .unwrap_or(t.accent);
            caps.push(sp(format!(" {} ", key_label(c)), chip(col, t)));
            caps.push(sp(" ", fg(t.dim)));
        }
        lines.push(Line::from(caps));
        lines.push(Line::from(""));
        for &c in &l.fresh {
            if let Some((k, _)) = app.layout.find(c) {
                let col = t.fingers[k.finger.kind()];
                lines.push(Line::from(vec![
                    sp(format!("  {}  ", key_label(c)), bold(col)),
                    sp(k.finger.name(), fg(t.dim)),
                ]));
            }
        }
        lines.push(Line::from(""));
    }
    lines.push(Line::from(sp(lessons::about(l), fg(t.dim))));
    lines.push(Line::from(""));
    let names = lessons::step_names(l.kind);
    for (s, name) in names.iter().enumerate() {
        let sel = s == app.step_sel;
        let st = app.state.stars(app.layout.kind, app.lesson_sel, s);
        let mut v = vec![
            sp(if sel { " ❯ " } else { "   " }, bold(t.accent)),
            sp(
                format!("{}. {:<12}", s + 1, name),
                if sel { bold(t.accent) } else { fg(t.fg) },
            ),
        ];
        v.extend(stars_spans(st, t));
        lines.push(Line::from(v));
    }
    if !app.lesson_unlocked(app.lesson_sel) {
        lines.push(Line::from(""));
        lines.push(Line::from(sp(
            "Kilitli: önce önceki dersi en az bir yıldızla bitir.",
            fg(t.err),
        )));
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), info_a);

    let kb = keyboard_lines(
        &app.layout,
        t,
        &KbMode::Lesson {
            unlocked: &l.unlocked,
            fresh: &l.fresh,
        },
    );
    f.render_widget(Paragraph::new(kb), kb_a);
}

// ---- serbest pratik ---------------------------------------------------------

fn draw_practice(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let rect = centered(area, 96, 20);
    let [left, _, right] = Split::horizontal([Length(34), Length(3), Min(30)]).areas(rect);

    let mut lines = vec![Line::from(sp("Kelime setleri", bold(t.fg))), Line::from("")];
    for (i, s) in app.sets.iter().enumerate() {
        let sel = i == app.set_sel;
        lines.push(Line::from(vec![
            sp(if sel { "❯ " } else { "  " }, bold(t.accent)),
            sp(
                format!("{:<23}", s.title),
                if sel { bold(t.accent) } else { fg(t.fg) },
            ),
            sp(format!("{:>6}", s.word_count()), fg(t.faint)),
        ]));
    }
    f.render_widget(Paragraph::new(lines), left);

    let Some(set) = app.sets.get(app.set_sel) else { return };
    let mut lines = vec![
        Line::from(sp(set.title.clone(), bold(t.fg))),
        Line::from(sp(set.desc.clone(), fg(t.dim))),
        Line::from(""),
    ];
    let mut chips = Vec::new();
    for (i, (name, _)) in MODES.iter().enumerate() {
        if i == app.mode_sel {
            chips.push(sp(format!(" {name} "), chip(t.accent, t)));
        } else {
            chips.push(sp(format!(" {name} "), fg(t.dim)));
        }
        chips.push(sp(" ", fg(t.dim)));
    }
    lines.push(Line::from(chips));
    lines.push(Line::from(""));
    let sample: String = match set.kind {
        SetKind::Sentences => set.items.iter().take(2).cloned().collect::<Vec<_>>().join(" "),
        _ => set.items.iter().take(16).cloned().collect::<Vec<_>>().join(" "),
    };
    lines.push(Line::from(sp("örnek", fg(t.faint))));
    lines.push(Line::from(sp(
        sample,
        fg(t.dim).add_modifier(Modifier::ITALIC),
    )));
    lines.push(Line::from(""));
    let label = app.practice_label(app.set_sel, app.mode_sel);
    match app.state.best_for(&label, app.layout.kind) {
        Some(b) => lines.push(Line::from(vec![
            sp("bu moddaki rekorun  ", fg(t.dim)),
            sp(format!("{b:.0} wpm"), bold(t.warn)),
        ])),
        None => lines.push(Line::from(sp(
            "bu modda henüz rekor yok, ilki senin olsun",
            fg(t.faint),
        ))),
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), right);
}

// ---- yazma ekranı -----------------------------------------------------------

fn draw_typing(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let Some(s) = app.session.as_ref() else { return };
    let width = (area.width.saturating_sub(8) as usize).clamp(20, 76);
    let show_kb = (app.state.settings.keyboard || s.strict) && area.height >= 20;
    let kb_h: u16 = if show_kb { 8 } else { 0 };
    let w = (width as u16 + 2).max(if show_kb { KB_WIDTH } else { 0 });
    let rect = centered(area, w, 9 + kb_h);
    let [label_a, _, stats_a, _, text_a, _, kb_a] = Split::vertical([
        Length(1),
        Length(1),
        Length(1),
        Length(1),
        Length(3),
        Length(2),
        Length(kb_h),
    ])
    .areas(rect);

    let mut label = vec![sp(app.label.clone(), fg(t.dim))];
    if !app.weak_now.is_empty() && app.label == "Zayıf tuşlar" {
        label.push(sp("  ", fg(t.dim)));
        for c in &app.weak_now {
            label.push(sp(format!(" {} ", key_label(*c)), chip(t.err, t)));
            label.push(sp(" ", fg(t.dim)));
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(label)).alignment(Alignment::Center),
        label_a,
    );

    let stats = if !s.started() {
        let msg = if s.strict {
            "yazmaya başla · yanlış tuşta imleç bekler"
        } else {
            "yazmaya başla · süre ilk tuşla başlar"
        };
        Line::from(sp(msg, fg(t.faint)))
    } else {
        let time = match s.remaining() {
            Some(r) => mmss(r),
            None => mmss(s.elapsed()),
        };
        let mut v = vec![
            sp(format!("{:.0}", s.wpm()), bold(t.accent)),
            sp(" wpm", fg(t.dim)),
            sp("     ", fg(t.dim)),
            sp(format!("%{:.0}", s.accuracy()), bold(t.fg)),
            sp(" doğruluk", fg(t.dim)),
            sp("     ", fg(t.dim)),
            sp(time, bold(t.fg)),
        ];
        if s.remaining().is_none() {
            let pct = 100.0 * s.typed.len() as f64 / s.target.len().max(1) as f64;
            v.push(sp(format!("     %{pct:.0} tamam"), fg(t.faint)));
        }
        Line::from(v)
    };
    f.render_widget(Paragraph::new(stats).alignment(Alignment::Center), stats_a);

    // Metin: imlecin satırı ortada kalacak şekilde 3 satır.
    let lines_idx = text::wrap(&s.target, width);
    let caret = s.typed.len();
    let caret_line = lines_idx
        .iter()
        .position(|&(a, b)| caret >= a && caret < b)
        .unwrap_or(lines_idx.len().saturating_sub(1));
    let first = caret_line.saturating_sub(1);
    let ghost = app
        .ghost
        .filter(|_| s.started() && !s.finished())
        .map(|g| (g * 5.0 / 60.0 * s.elapsed()) as usize);
    let flash = s
        .last_wrong
        .is_some_and(|(_, at)| at.elapsed().as_millis() < 350);

    let mut text_lines = Vec::new();
    for &(a, b) in lines_idx.iter().skip(first).take(3) {
        let mut spans = Vec::with_capacity(b - a);
        for i in a..b {
            let ch = s.target[i];
            let mut shown = ch;
            let style = if i < caret {
                if s.typed[i] == ch {
                    fg(t.fg)
                } else {
                    if ch == ' ' {
                        shown = '·';
                    }
                    fg(t.err).add_modifier(Modifier::UNDERLINED)
                }
            } else if i == caret {
                if flash {
                    if ch == ' ' {
                        shown = '·';
                    }
                    Style::new().fg(t.on_accent).bg(t.err)
                } else {
                    Style::new()
                        .fg(t.accent)
                        .bg(t.key_bg)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                }
            } else if Some(i) == ghost {
                fg(t.ghost).add_modifier(Modifier::UNDERLINED)
            } else {
                fg(t.dim)
            };
            spans.push(sp(shown.to_string(), style));
        }
        text_lines.push(Line::from(spans));
    }
    f.render_widget(
        Paragraph::new(text_lines),
        centered(text_a, width as u16 + 1, 3),
    );

    if show_kb {
        let next = s.target.get(caret).copied();
        let wrong = s
            .last_wrong
            .filter(|(_, at)| at.elapsed().as_millis() < 400)
            .map(|(c, _)| c);
        let mut lines = keyboard_lines(
            &app.layout,
            t,
            &KbMode::Next {
                next,
                wrong,
                colored: s.strict,
            },
        );
        lines.push(finger_legend(t));
        lines.push(hint_line(&app.layout, t, next));
        f.render_widget(Paragraph::new(lines), centered(kb_a, KB_WIDTH, kb_h));
    }
}

// ---- sonuç ------------------------------------------------------------------

fn draw_results(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let Some(r) = app.result.as_ref() else { return };
    let rect = centered(area, 86, 25);
    let [title_a, msg_a, _, big_a, _, chart_a, _, keys_a] = Split::vertical([
        Length(1),
        Length(1),
        Length(1),
        Length(5),
        Length(1),
        Length(10),
        Length(1),
        Length(2),
    ])
    .areas(rect);

    f.render_widget(
        Paragraph::new(Line::from(sp(r.label.clone(), fg(t.dim)))).alignment(Alignment::Center),
        title_a,
    );

    let msg = match r.stars {
        Some(0) => Line::from(sp(
            "Doğruluk %90'ın altında kaldı. Acele etme: önce doğruluk, hız arkasından gelir.",
            fg(t.err),
        )),
        Some(n) => {
            let mut v = stars_spans(n, t);
            let text = match n {
                1 => "   Geçtin! Biraz daha akıcılık, bir yıldız daha.",
                2 => "   Çok iyi! Üçüncü yıldız bir nefes uzakta.",
                _ => "   Kusursuz. Parmakların ezberledi.",
            };
            v.push(sp(text, fg(t.fg)));
            if r.new_best {
                v.push(sp("   ✦ yeni rekor", bold(t.warn)));
            }
            Line::from(v)
        }
        None if r.new_best => Line::from(sp("✦ yeni rekor!", bold(t.warn))),
        None => Line::from(""),
    };
    f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), msg_a);

    let [num_a, info_a] = Split::horizontal([Percentage(50), Percentage(50)]).areas(big_a);
    f.render_widget(
        Paragraph::new(big_number(&format!("{:.0}", r.wpm), t.accent)).alignment(Alignment::Right),
        num_a,
    );
    let row = |k: &str, v: String, st: Style| {
        Line::from(vec![sp(format!("   {k:<11}"), fg(t.dim)), sp(v, st)])
    };
    let info = vec![
        Line::from(sp("   wpm", bold(t.accent))),
        row("doğruluk", format!("%{:.1}", r.acc), bold(t.fg)),
        row("ham", format!("{:.0}", r.raw), fg(t.fg)),
        row("istikrar", format!("%{:.0}", r.consistency), fg(t.fg)),
        row("süre", mmss(r.secs), fg(t.fg)),
    ];
    f.render_widget(Paragraph::new(info), info_a);

    if r.samples.len() >= 2 {
        let wpm_pts: Vec<(f64, f64)> = r.samples.iter().map(|s| (s.sec, s.wpm)).collect();
        let raw_pts: Vec<(f64, f64)> = r.samples.iter().map(|s| (s.sec, s.raw)).collect();
        let err_pts: Vec<(f64, f64)> = r
            .samples
            .iter()
            .filter(|s| s.errors > 0)
            .map(|s| (s.sec, s.wpm))
            .collect();
        let min_x = r.samples[0].sec.min(1.0);
        let max_x = r.samples.last().map(|s| s.sec).unwrap_or(1.0).max(min_x + 1.0);
        let top = wpm_pts
            .iter()
            .chain(raw_pts.iter())
            .map(|p| p.1)
            .fold(10.0, f64::max);
        let max_y = (top / 10.0).ceil() * 10.0;
        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(fg(t.faint))
                .data(&raw_pts),
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(fg(t.accent))
                .data(&wpm_pts),
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(fg(t.err))
                .data(&err_pts),
        ];
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .style(fg(t.faint))
                    .bounds([min_x, max_x])
                    .labels(vec![
                        sp(format!("{min_x:.0} sn"), fg(t.dim)),
                        sp(format!("{max_x:.0} sn"), fg(t.dim)),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .style(fg(t.faint))
                    .bounds([0.0, max_y])
                    .labels(vec![
                        sp("0", fg(t.dim)),
                        sp(format!("{:.0}", max_y / 2.0), fg(t.dim)),
                        sp(format!("{max_y:.0}"), fg(t.dim)),
                    ]),
            );
        f.render_widget(chart, chart_a);
    } else {
        f.render_widget(
            Paragraph::new(Line::from(sp("grafik için alıştırma çok kısa", fg(t.faint))))
                .alignment(Alignment::Center),
            chart_a,
        );
    }

    let mut l1 = vec![
        sp("── ", fg(t.faint)),
        sp("wpm", fg(t.accent)),
        sp("  ── ", fg(t.faint)),
        sp("ham", fg(t.dim)),
        sp("  • ", fg(t.err)),
        sp("hata", fg(t.dim)),
        sp("      en çok hata  ", fg(t.dim)),
    ];
    if r.missed.is_empty() {
        l1.push(sp("yok ✦", fg(t.ok)));
    }
    for (c, n) in &r.missed {
        let shown = if *c == ' ' { "␣".to_string() } else { c.to_string() };
        l1.push(sp(format!(" {shown} "), chip(t.err, t)));
        l1.push(sp(format!("×{n}  "), fg(t.dim)));
    }
    let mut l2 = vec![sp("en yavaş  ", fg(t.dim))];
    for (c, ms) in &r.slow {
        l2.push(sp(format!(" {c} "), chip(t.warn, t)));
        l2.push(sp(format!("{ms:.0}ms  "), fg(t.dim)));
    }
    f.render_widget(
        Paragraph::new(vec![
            Line::from(l1).alignment(Alignment::Center),
            Line::from(l2).alignment(Alignment::Center),
        ]),
        keys_a,
    );
}

// ---- istatistikler ----------------------------------------------------------

fn draw_stats(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let kind = app.layout.kind;
    let hist = app.state.history_for(kind);
    let rect = centered(area, 84, 25);
    let [title_a, _, sum_a, _, trend_t, trend_a, _, heat_t, heat_a, _, weak_a] =
        Split::vertical([
            Length(1),
            Length(1),
            Length(2),
            Length(1),
            Length(1),
            Length(3),
            Length(1),
            Length(1),
            Length(6),
            Length(1),
            Length(2),
        ])
        .areas(rect);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            sp("İstatistikler", bold(t.fg)),
            sp(format!("  ·  {}", kind.name()), fg(t.dim)),
        ])),
        title_a,
    );

    if hist.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(sp(
                "Henüz kayıt yok. Bir ders ya da pratik bitir, burası dolsun.",
                fg(t.dim),
            ))),
            sum_a,
        );
    } else {
        let total_secs: f64 = hist.iter().map(|r| r.secs).sum();
        let best = hist.iter().map(|r| r.wpm).fold(0.0, f64::max);
        let recent: Vec<_> = hist.iter().rev().take(10).collect();
        let avg_wpm = recent.iter().map(|r| r.wpm).sum::<f64>() / recent.len() as f64;
        let avg_acc = recent.iter().map(|r| r.acc).sum::<f64>() / recent.len() as f64;
        let cell = |k: &str, v: String| {
            vec![sp(format!("{k} "), fg(t.dim)), sp(format!("{v:<10}"), bold(t.fg))]
        };
        let mut l1 = cell("alıştırma", hist.len().to_string());
        l1.extend(cell("süre", format!("{:.0} dk", total_secs / 60.0)));
        l1.extend(cell("rekor", format!("{best:.0} wpm")));
        let mut l2 = cell("son 10 ort.", format!("{avg_wpm:.0} wpm"));
        l2.extend(cell("doğruluk", format!("%{avg_acc:.0}")));
        l2.extend(cell(
            "ders",
            format!("{}/{}", app.lessons_done(), app.lessons.len()),
        ));
        f.render_widget(Paragraph::new(vec![Line::from(l1), Line::from(l2)]), sum_a);
    }

    // Gidişat: son alıştırmaların wpm'i, 3 satırlık çubuklar.
    f.render_widget(
        Paragraph::new(Line::from(sp("gidişat", fg(t.faint)))),
        trend_t,
    );
    let n = (trend_a.width as usize).saturating_sub(8).max(1);
    let vals: Vec<f64> = hist.iter().rev().take(n).rev().map(|r| r.wpm).collect();
    if !vals.is_empty() {
        let max = vals.iter().copied().fold(1.0, f64::max);
        let mut lines = Vec::new();
        for row in (0..3).rev() {
            let label = match row {
                2 => format!("{max:>5.0} ┤ "),
                0 => format!("{:>5} ┤ ", 0),
                _ => "      │ ".to_string(),
            };
            let mut v = vec![sp(label, fg(t.faint))];
            let mut bar = String::new();
            for x in &vals {
                let level = (x / max * 24.0).round() as i64 - row * 8;
                bar.push_str(BARS[level.clamp(0, 8) as usize]);
            }
            v.push(sp(bar, fg(t.accent)));
            lines.push(Line::from(v));
        }
        f.render_widget(Paragraph::new(lines), trend_a);
    }

    // Isı haritası
    f.render_widget(
        Paragraph::new(Line::from(vec![
            sp("tuş ısı haritası  ", fg(t.faint)),
            sp(" az hata ", Style::new().fg(t.on_accent).bg(t.ok)),
            sp(" ", fg(t.dim)),
            sp(" orta ", Style::new().fg(t.on_accent).bg(t.warn)),
            sp(" ", fg(t.dim)),
            sp(" çok hata ", Style::new().fg(t.on_accent).bg(t.err)),
        ])),
        heat_t,
    );
    let mut heat: HashMap<char, f64> = HashMap::new();
    for (ch, st) in app.state.key_stats(kind) {
        if st.total() >= 5 {
            heat.insert(ch, (st.error_rate() / 0.15).min(1.0));
        }
    }
    let kb = keyboard_lines(&app.layout, t, &KbMode::Heat(&heat));
    f.render_widget(Paragraph::new(kb), centered(heat_a, KB_WIDTH, 6));

    let weak = app.state.weak_keys(kind, 8);
    let mut v = vec![sp("en zayıf  ", fg(t.dim))];
    if weak.is_empty() {
        v.push(sp("henüz yeterli veri yok", fg(t.faint)));
    }
    for w in &weak {
        v.push(sp(format!(" {} ", key_label(w.ch)), chip(theme::heat(t, (w.error_rate / 0.15).min(1.0)), t)));
        v.push(sp(
            format!(" %{:.0} {:.0}ms   ", w.error_rate * 100.0, w.latency),
            fg(t.dim),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(v)).wrap(Wrap { trim: true }), weak_a);
}

// ---- ayarlar ----------------------------------------------------------------

fn draw_settings(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let rect = centered(area, 76, 16);
    let rows = app.settings_rows();
    let mut lines = vec![Line::from(sp("Ayarlar", bold(t.fg))), Line::from("")];
    for (i, (name, value, _)) in rows.iter().enumerate() {
        let sel = i == app.settings_sel;
        lines.push(Line::from(vec![
            sp(if sel { " ❯ " } else { "   " }, bold(t.accent)),
            sp(
                format!("{name:<28}"),
                if sel { bold(t.accent) } else { fg(t.fg) },
            ),
            sp(if sel { "‹ " } else { "  " }, fg(t.dim)),
            sp(value.clone(), if sel { bold(t.fg) } else { fg(t.dim) }),
            sp(if sel { " ›" } else { "  " }, fg(t.dim)),
        ]));
    }
    lines.push(Line::from(""));
    if let Some((_, _, desc)) = rows.get(app.settings_sel) {
        lines.push(Line::from(sp(format!("   {desc}"), fg(t.dim))));
    }
    lines.push(Line::from(""));
    if let Some(p) = State::path() {
        lines.push(Line::from(sp(
            format!("   ilerleme: {}", p.display()),
            fg(t.faint),
        )));
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), rect);
}
