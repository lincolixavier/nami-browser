extern crate gtk;

use gtk::prelude::*;
use gtk::{ Label };

mod dom;


fn render_html(element: &dom::Element, buffer: &gtk::TextBuffer, window: &gtk::Window) {
    for child in &element.children {
        if child.tag == "#text" {
            buffer.insert_at_cursor(&child.tag);
        }
    }
}

fn main() {
    if gtk::init().is_err() {
        println!("Falha ao inicializar o GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("NAMI");
    window.set_default_size(800, 600);

    let text_view = gtk::TextView::new();
    let label = Label::new(Some("<h1>BOOOORAA</h1>"));

    let buffer = text_view.buffer().expect("Erro ao obter o buffer de texto");

    text_view.set_editable(false);
    text_view.set_cursor_visible(false);


    let html = "<html><body> NAMI </body></html>";
    let root = dom::parse_html(html);

    render_html(&root, &buffer, &window);
    //window.add(&text_view);
    //window.add(&label);

    window.show_all();
    gtk::main();
}


