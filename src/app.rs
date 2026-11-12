//! Uygulama durumu, ekranlar arası gezinme ve tuş işleme.

use std::io;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::data::{self, WordSet};
use crate::layout::Layout;
use crate::lessons::{self, Lesson, STEP_COUNT};
use crate::session::{Limit, Sample, Session};
use crate::state::{Record, State};
use crate::text;
use crate::theme::{self, Theme};
use crate::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
    Lessons,
    Practice,
    Typing,
    Results,
    Stats,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PracticeMode {
    Time(u64),
    Words(usize),
}

pub const MODES: [(&str, PracticeMode); 7] = [
    ("15 sn", PracticeMode::Time(15)),
    ("30 sn", PracticeMode::Time(30)),
    ("60 sn", PracticeMode::Time(60)),
    ("120 sn", PracticeMode::Time(120)),
    ("25 kelime", PracticeMode::Words(25)),
    ("50 kelime", PracticeMode::Words(50)),
    ("100 kelime", PracticeMode::Words(100)),
];

pub const MENU: [(&str, &str); 6] = [
    ("Dersler", "tuşları parmak parmak öğren"),
    ("Serbest pratik", "kelime setleriyle hız çalış"),
    ("Zayıf tuşlar", "en çok zorlandığın harflere odaklan"),
    ("İstatistikler", "ilerleme, rekorlar, ısı haritası"),
    ("Ayarlar", "düzen, tema, tercihler"),
    ("Çıkış", "görüşmek üzere"),
];

pub const SETTINGS_COUNT: usize = 5;

#[derive(Clone, Debug)]
pub enum Origin {
    Lesson { idx: usize, step: usize },
    Practice { set: usize, mode: usize },
    Weak,
}

pub struct ResultInfo {
    pub label: String,
    pub wpm: f64,
    pub raw: f64,
    pub acc: f64,
    pub secs: f64,
    pub consistency: f64,
    pub stars: Option<u8>,
    pub new_best: bool,
    pub samples: Vec<Sample>,
    pub missed: Vec<(char, u32)>,
    pub slow: Vec<(char, f64)>,
}

pub struct App {
    pub state: State,
    pub themes: Vec<Theme>,
    pub sets: Vec<WordSet>,
    pub lesson_words: Vec<String>,
    pub proverbs: Vec<String>,
    pub layout: Layout,
    pub lessons: Vec<Lesson>,
    pub screen: Screen,
    pub menu: usize,
    pub lesson_sel: usize,
    pub step_sel: usize,
    pub set_sel: usize,
    pub mode_sel: usize,
    pub settings_sel: usize,
    pub session: Option<Session>,
    pub origin: Option<Origin>,
    pub result: Option<ResultInfo>,
    pub label: String,
    pub ghost: Option<f64>,
    pub weak_now: Vec<char>,
    pub show_help: bool,
    pub message: Option<(String, Instant)>,
    pub quit: bool,
}

impl App {
    pub fn new() -> Self {
        let state = State::load();
        let themes = theme::all();
        let sets = data::load();
        let lesson_words = data::lesson_words(&sets);
        let proverbs = data::proverbs(&sets);
        let layout = Layout::new(state.settings.layout);
        let lessons = lessons::build(&layout);
        let set_sel = state.settings.set.min(sets.len().saturating_sub(1));
        let mode_sel = state.settings.mode.min(MODES.len() - 1);
        let lesson_sel = (0..lessons.len())
            .find(|&i| !state.lesson_done(layout.kind, i, STEP_COUNT))
            .unwrap_or(0);
        App {
            state,
            themes,
            sets,
            lesson_words,
            proverbs,
            layout,
            lessons,
            screen: Screen::Home,
            menu: 0,
            lesson_sel,
            step_sel: 0,
            set_sel,
            mode_sel,
            settings_sel: 0,
            session: None,
            origin: None,
            result: None,
            label: String::new(),
            ghost: None,
            weak_now: Vec::new(),
            show_help: false,
            message: None,
            quit: false,
        }
    }

    pub fn theme(&self) -> &Theme {
        let i = self
            .state
            .settings
            .theme
            .min(self.themes.len().saturating_sub(1));
        &self.themes[i]
    }

    pub fn save(&self) {
        self.state.save();
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            {
                let app: &App = self;
                terminal.draw(|f| ui::draw(f, app))?;
            }
            if event::poll(Duration::from_millis(40))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.on_key(key);
                    }
                }
            }
            self.tick();
        }
        Ok(())
    }

    // ---- sorgular -------------------------------------------------------

    pub fn lesson_unlocked(&self, i: usize) -> bool {
        self.state.settings.unlock_all
            || i == 0
            || self.state.lesson_done(self.layout.kind, i - 1, STEP_COUNT)
    }

    pub fn lesson_stars(&self, i: usize) -> u8 {
        (0..STEP_COUNT)
            .map(|s| self.state.stars(self.layout.kind, i, s))
            .min()
            .unwrap_or(0)
    }

    pub fn lessons_done(&self) -> usize {
        (0..self.lessons.len())
            .filter(|&i| self.state.lesson_done(self.layout.kind, i, STEP_COUNT))
            .count()
    }

    pub fn practice_label(&self, set: usize, mode: usize) -> String {
        format!("Pratik · {} · {}", self.sets[set].title, MODES[mode].0)
    }

    pub fn toast(&mut self, msg: &str) {
        self.message = Some((msg.to_string(), Instant::now()));
    }

    pub fn active_message(&self) -> Option<&str> {
        self.message
            .as_ref()
            .filter(|(_, at)| at.elapsed() < Duration::from_millis(3000))
            .map(|(m, _)| m.as_str())
    }

    pub fn settings_rows(&self) -> Vec<(&'static str, String, &'static str)> {
        let s = &self.state.settings;
        let onoff = |b: bool| (if b { "açık" } else { "kapalı" }).to_string();
        vec![
            (
                "Klavye düzeni",
                self.layout.kind.name().to_string(),
                "Dersler ve ekran klavyesi bu düzene göre. İstatistikler düzene göre ayrı tutulur.",
            ),
            (
                "Tema",
                self.theme().name.clone(),
                "Omarchy kuruluysa aktif sistem teması otomatik listelenir.",
            ),
            (
                "Pratikte ekran klavyesi",
                onoff(s.keyboard),
                "Derslerde hep açık. Pratikte kapatırsan gözün metinde kalır.",
            ),
            (
                "Hayalet imleç",
                onoff(s.ghost),
                "Aynı moddaki rekor hızın metinde altı çizili bir hayalet olarak koşar.",
            ),
            (
                "Tüm dersleri aç",
                onoff(s.unlock_all),
                "Kilitleri kaldırır; istediğin derse atla.",
            ),
        ]
    }

    // ---- oturum başlatma ------------------------------------------------

    fn begin(&mut self, text: String, strict: bool, limit: Limit, origin: Origin) {
        self.ghost = if self.state.settings.ghost && !strict {
            self.state.best_for(&self.label, self.layout.kind)
        } else {
            None
        };
        self.session = Some(Session::new(&text, strict, limit));
        self.origin = Some(origin);
        self.result = None;
        self.screen = Screen::Typing;
    }

    pub fn start_lesson(&mut self, idx: usize, step: usize) {
        if idx >= self.lessons.len() {
            return;
        }
        let mut rng = rand::thread_rng();
        let lesson = &self.lessons[idx];
        let text = lessons::generate(lesson, step, &self.lesson_words, &self.proverbs, &mut rng);
        let label = format!(
            "Ders {} · {} · {}",
            idx + 1,
            lesson.title,
            lessons::step_names(lesson.kind)[step.min(STEP_COUNT - 1)]
        );
        self.label = label;
        self.lesson_sel = idx;
        self.step_sel = step;
        self.begin(text, true, Limit::Full, Origin::Lesson { idx, step });
    }

    pub fn start_practice(&mut self, set: usize, mode: usize) {
        if set >= self.sets.len() {
            return;
        }
        let mut rng = rand::thread_rng();
        let m = MODES[mode].1;
        let count = match m {
            PracticeMode::Time(_) => 160,
            PracticeMode::Words(n) => n,
        };
        let text = data::practice_text(&self.sets[set], count, &mut rng);
        let limit = match m {
            PracticeMode::Time(s) => Limit::Time(s),
            PracticeMode::Words(_) => Limit::Full,
        };
        self.label = self.practice_label(set, mode);
        self.begin(text, false, limit, Origin::Practice { set, mode });
    }

    pub fn start_weak(&mut self) {
        let weak = self.state.weak_keys(self.layout.kind, 6);
        if weak.len() < 3 {
            self.toast("Henüz yeterli veri yok: önce birkaç ders ya da pratik bitir.");
            return;
        }
        let keys: Vec<char> = weak.iter().map(|w| w.ch).collect();
        let mut rng = rand::thread_rng();
        let text = data::weak_text(&self.lesson_words, &keys, 36, &mut rng);
        self.label = "Zayıf tuşlar".to_string();
        self.weak_now = keys;
        self.begin(text, false, Limit::Full, Origin::Weak);
    }

    fn restart(&mut self) {
        match self.origin.clone() {
            Some(Origin::Lesson { idx, step }) => self.start_lesson(idx, step),
            Some(Origin::Practice { set, mode }) => self.start_practice(set, mode),
            Some(Origin::Weak) => self.start_weak(),
            None => {}
        }
    }

    fn back_to_origin(&mut self) {
        self.screen = match self.origin {
            Some(Origin::Lesson { .. }) => Screen::Lessons,
            Some(Origin::Practice { .. }) => Screen::Practice,
            _ => Screen::Home,
        };
        self.session = None;
    }

    // ---- zaman ve bitiş -------------------------------------------------

    fn tick(&mut self) {
        if self.screen != Screen::Typing {
            return;
        }
        if let Some(s) = self.session.as_mut() {
            s.tick();
        }
        self.check_finished();
    }

    fn check_finished(&mut self) {
        let done = self.session.as_ref().is_some_and(|s| s.finished());
        if self.screen == Screen::Typing && done {
            self.finish();
        }
    }

    fn finish(&mut self) {
        let Some(s) = self.session.take() else { return };
        let layout = self.layout.kind;
        let wpm = s.wpm();
        let raw = s.raw();
        let acc = s.accuracy();
        let secs = s.elapsed();
        let consistency = s.consistency();

        for (ch, cs) in &s.chars {
            let id = State::key_id(layout, text::lower_char(*ch));
            let e = self.state.keys.entry(id).or_default();
            e.hits += cs.hits as u64;
            e.misses += cs.misses as u64;
            e.lat_sum += cs.lat_sum;
            e.lat_n += cs.lat_n as u64;
        }

        let prev_best = self.state.best_for(&self.label, layout);
        let enough = secs >= 3.0 && s.typed.len() >= 10;

        let mut stars = None;
        if let Some(Origin::Lesson { idx, step }) = self.origin.clone() {
            let st = lessons::stars(acc, wpm);
            stars = Some(st);
            self.state.set_stars(layout, idx, step, st);
        }

        if enough {
            self.state.history.push(Record {
                day: State::today(),
                wpm,
                raw,
                acc,
                secs,
                label: self.label.clone(),
                layout,
            });
            let n = self.state.history.len();
            if n > 2000 {
                self.state.history.drain(0..n - 2000);
            }
            self.state.mark_today();
        }
        let new_best = enough && prev_best.is_some_and(|b| wpm > b);

        let mut missed: Vec<(char, u32)> = s
            .chars
            .iter()
            .filter(|(_, c)| c.misses > 0)
            .map(|(ch, c)| (*ch, c.misses))
            .collect();
        missed.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        missed.truncate(6);

        let mut slow: Vec<(char, f64)> = s
            .chars
            .iter()
            .filter(|(ch, c)| c.lat_n >= 2 && **ch != ' ')
            .map(|(ch, c)| (*ch, c.lat_sum / c.lat_n as f64))
            .collect();
        slow.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        slow.truncate(5);

        self.result = Some(ResultInfo {
            label: self.label.clone(),
            wpm,
            raw,
            acc,
            secs,
            consistency,
            stars,
            new_best,
            samples: s.samples.clone(),
            missed,
            slow,
        });
        self.state.save();
        self.screen = Screen::Results;
    }

    // ---- tuşlar ---------------------------------------------------------

    fn on_key(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let alt = k.modifiers.contains(KeyModifiers::ALT);
        if ctrl && !alt && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('q')) {
            self.quit = true;
            return;
        }
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.screen != Screen::Typing && k.code == KeyCode::Char('?') {
            self.show_help = true;
            return;
        }
        match self.screen {
            Screen::Home => self.key_home(k),
            Screen::Lessons => self.key_lessons(k),
            Screen::Practice => self.key_practice(k),
            Screen::Typing => self.key_typing(k, ctrl, alt),
            Screen::Results => self.key_results(k),
            Screen::Stats => self.key_stats(k),
            Screen::Settings => self.key_settings(k),
        }
    }

    fn activate(&mut self, item: usize) {
        match item {
            0 => self.screen = Screen::Lessons,
            1 => self.screen = Screen::Practice,
            2 => self.start_weak(),
            3 => self.screen = Screen::Stats,
            4 => self.screen = Screen::Settings,
            _ => self.quit = true,
        }
    }

    fn key_home(&mut self, k: KeyEvent) {
        let n = MENU.len();
        match k.code {
            KeyCode::Up | KeyCode::Char('k') => self.menu = (self.menu + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.menu = (self.menu + 1) % n,
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.menu),
            KeyCode::Char(c @ '1'..='6') => {
                let i = (c as u8 - b'1') as usize;
                self.menu = i;
                self.activate(i);
            }
            KeyCode::Esc | KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
    }

    fn key_lessons(&mut self, k: KeyEvent) {
        let n = self.lessons.len();
        match k.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.lesson_sel = (self.lesson_sel + n - 1) % n;
                self.step_sel = self.first_open_step(self.lesson_sel);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.lesson_sel = (self.lesson_sel + 1) % n;
                self.step_sel = self.first_open_step(self.lesson_sel);
            }
            KeyCode::Left | KeyCode::Char('h') => self.step_sel = self.step_sel.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                self.step_sel = (self.step_sel + 1).min(STEP_COUNT - 1)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.lesson_unlocked(self.lesson_sel) {
                    self.start_lesson(self.lesson_sel, self.step_sel);
                } else {
                    self.toast("Bu ders kilitli: önce önceki dersi en az bir yıldızla bitir.");
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Home,
            _ => {}
        }
    }

    /// Henüz yıldız alınmamış ilk adım; hepsi bittiyse ilk adım.
    fn first_open_step(&self, lesson: usize) -> usize {
        (0..STEP_COUNT)
            .find(|&s| self.state.stars(self.layout.kind, lesson, s) == 0)
            .unwrap_or(0)
    }

    fn key_practice(&mut self, k: KeyEvent) {
        let n = self.sets.len();
        match k.code {
            KeyCode::Up | KeyCode::Char('k') => self.set_sel = (self.set_sel + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.set_sel = (self.set_sel + 1) % n,
            KeyCode::Left | KeyCode::Char('h') => self.mode_sel = self.mode_sel.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                self.mode_sel = (self.mode_sel + 1).min(MODES.len() - 1)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state.settings.set = self.set_sel;
                self.state.settings.mode = self.mode_sel;
                self.start_practice(self.set_sel, self.mode_sel);
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Home,
            _ => {}
        }
    }

    fn key_typing(&mut self, k: KeyEvent, ctrl: bool, alt: bool) {
        match k.code {
            KeyCode::Esc => {
                self.back_to_origin();
                return;
            }
            KeyCode::Tab => {
                self.restart();
                return;
            }
            KeyCode::Backspace => {
                if let Some(s) = self.session.as_mut() {
                    if ctrl || alt {
                        s.delete_word();
                    } else {
                        s.backspace();
                    }
                }
            }
            KeyCode::Char(c) if ctrl && !alt => {
                if c == 'w' || c == 'h' {
                    if let Some(s) = self.session.as_mut() {
                        s.delete_word();
                    }
                }
            }
            KeyCode::Char(c) => self.type_char(c),
            _ => {}
        }
        self.check_finished();
    }

    fn type_char(&mut self, c: char) {
        let need_more = match self.session.as_mut() {
            Some(s) => {
                s.type_char(c);
                s.needs_more()
            }
            None => false,
        };
        if need_more {
            if let Some(Origin::Practice { set, .. }) = self.origin.clone() {
                let more = data::practice_text(&self.sets[set], 60, &mut rand::thread_rng());
                if let Some(s) = self.session.as_mut() {
                    s.extend(&more);
                }
            }
        }
    }

    fn key_results(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.result = None;
                self.back_to_origin();
            }
            KeyCode::Tab | KeyCode::Char('r') => self.restart(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let passed = self
                    .result
                    .as_ref()
                    .map(|r| r.stars.unwrap_or(1) > 0)
                    .unwrap_or(true);
                match self.origin.clone() {
                    Some(Origin::Lesson { idx, step }) if passed => {
                        if step + 1 < STEP_COUNT {
                            self.start_lesson(idx, step + 1);
                        } else if idx + 1 < self.lessons.len() {
                            self.start_lesson(idx + 1, 0);
                        } else {
                            self.toast("Tüm dersleri bitirdin. Artık serbest pratik senin!");
                            self.back_to_origin();
                        }
                    }
                    _ => self.restart(),
                }
            }
            _ => {}
        }
    }

    fn key_stats(&mut self, k: KeyEvent) {
        if matches!(k.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
            self.screen = Screen::Home;
        }
    }

    fn key_settings(&mut self, k: KeyEvent) {
        let n = SETTINGS_COUNT;
        match k.code {
            KeyCode::Up | KeyCode::Char('k') => self.settings_sel = (self.settings_sel + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.settings_sel = (self.settings_sel + 1) % n,
            KeyCode::Left | KeyCode::Char('h') => self.change_setting(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                self.change_setting(1)
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state.save();
                self.screen = Screen::Home;
            }
            _ => {}
        }
    }

    fn change_setting(&mut self, dir: i32) {
        match self.settings_sel {
            0 => {
                let kind = self.layout.kind.toggle();
                self.state.settings.layout = kind;
                self.layout = Layout::new(kind);
                self.lessons = lessons::build(&self.layout);
                self.lesson_sel = (0..self.lessons.len())
                    .find(|&i| !self.state.lesson_done(kind, i, STEP_COUNT))
                    .unwrap_or(0);
                self.step_sel = 0;
            }
            1 => {
                let n = self.themes.len().max(1) as i32;
                let cur = self.state.settings.theme.min(self.themes.len().saturating_sub(1)) as i32;
                self.state.settings.theme = ((cur + dir).rem_euclid(n)) as usize;
            }
            2 => self.state.settings.keyboard = !self.state.settings.keyboard,
            3 => self.state.settings.ghost = !self.state.settings.ghost,
            _ => self.state.settings.unlock_all = !self.state.settings.unlock_all,
        }
        self.state.save();
    }
}
