mod layout;
mod parser;

use gtk::prelude::*;
use gtk::{ScrolledWindow, TextView, Window, WindowType, WrapMode};

const SAMPLE_HTML: &str = r#"
<html>
    <head>
        <meta charset="utf-8"/>
        <title>Nami Browser Test</title>
    </head>
    <body>
        <h1>Mugiwara no Luffy &gt;&gt; 🏴‍☠️ &lt;&lt; Nami</h1>
        <h2>Segundo título</h2>
        <p>Este é um parágrafo de teste para verificar se o parser está funcionando corretamente.</p>
        <h3>Título menor</h3>
        <p>Outro parágrafo com <strong>texto em negrito</strong> e <em>texto em itálico</em>.</p>
    </body>
</html>
"#;

fn main() {
    let text_only = std::env::args().any(|a| a == "--text-only");

    let html = std::env::args()
        .position(|a| a == "--html")
        .and_then(|i| std::env::args().nth(i + 1))
        .map(|path| std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Erro ao ler {path}: {e}");
            String::new()
        }))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| SAMPLE_HTML.to_string());

    let parsed = parser::parse_html(&html);
    let for_terminal = layout::layout_document(&parsed, 72);
    let for_gui = layout::layout_document(&parsed, 10_000);

    if text_only {
        println!("Nami Browser — saída texto (--text-only)\n{}", "=".repeat(50));
        println!("{for_terminal}");
        return;
    }

    if gtk::init().is_err() {
        eprintln!("GTK não iniciou; imprimindo em texto. Use --text-only para forçar.");
        println!("{for_terminal}");
        std::process::exit(0);
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Nami Browser");
    window.set_default_size(880, 620);
    window.connect_destroy(|_| gtk::main_quit());

    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_wrap_mode(WrapMode::Word);
    text_view.set_left_margin(16);
    text_view.set_right_margin(16);
    text_view.set_top_margin(16);
    text_view.set_bottom_margin(16);

    text_view
        .buffer()
        .expect("TextView sempre tem buffer")
        .set_text(&for_gui);

    scroll.add(&text_view);
    window.add(&scroll);
    window.show_all();

    gtk::main();
}
