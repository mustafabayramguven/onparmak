//! onparmak: Türkçe on parmak klavye eğitmeni.

mod app;
mod data;
mod layout;
mod lessons;
mod session;
mod state;
mod text;
mod theme;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("onparmak {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("onparmak {}", env!("CARGO_PKG_VERSION"));
        println!("Türkçe on parmak klavye eğitmeni. Argümansız çalıştır.");
        println!();
        println!("  --version   sürümü göster");
        println!("  --help      bu yardımı göster");
        if let Some(path) = state::State::path() {
            println!();
            println!("İlerleme dosyası: {}", path.display());
        }
        return Ok(());
    }

    let mut app = app::App::new();
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    app.save();
    result
}
