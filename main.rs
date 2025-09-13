use std::collections::HashMap;
use gtk::prelude::*;
use gtk::glib;

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Element>,
    pub text_content: Option<String>,
}

impl Element {
    fn new(tag: String) -> Self {
        Element {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: None,
        }
    }
    
    fn new_text(text: String) -> Self {
        Element {
            tag: "text".to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: Some(text),
        }
    }
}

pub fn parse_html(html: &str) -> Element {
    let mut root = Element::new("root".to_string());
    let mut stack: Vec<Element> = Vec::new();
    let mut current_tag = String::new();
    let mut current_text = String::new();
    let mut is_tag_open = false;
    let mut is_closing_tag = false;

    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => {
                // Only treat as tag if it's followed by a letter or /
                if let Some(&next_char) = chars.peek() {
                    if next_char.is_alphabetic() || next_char == '/' {
                        // Save any accumulated text content
                        if !current_text.trim().is_empty() {
                            let text_element = Element::new_text(current_text.trim().to_string());
                            if let Some(current) = stack.last_mut() {
                                current.children.push(text_element);
                            } else {
                                root.children.push(text_element);
                            }
                            current_text.clear();
                        }
                        
                        if next_char == '/' {
                            chars.next();
                            is_closing_tag = true;
                        }
                        is_tag_open = true;
                        current_tag.clear();
                    } else {
                        // It's not a real tag, treat as text
                        current_text.push(c);
                    }
                } else {
                    // End of string, treat as text
                    current_text.push(c);
                }
            }
            '>' => {
                if is_tag_open {
                    if is_closing_tag {
                        // Handle closing tag
                        if !stack.is_empty() {
                            let closed_element = stack.pop().unwrap();
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(closed_element);
                            } else {
                                root.children.push(closed_element);
                            }
                        }
                        is_closing_tag = false;
                    } else {
                        // Handle opening tag
                        let new_element = Element::new(current_tag.clone());
                        stack.push(new_element);
                    }
                    current_tag.clear();
                    is_tag_open = false;
                } else {
                    // Not in a tag, treat as text
                    current_text.push(c);
                }
            }
            c if is_tag_open => {
                current_tag.push(c);
            }
            c if !is_tag_open => {
                current_text.push(c);
            }
            _ => {}
        }
    }

    // Handle any remaining text
    if !current_text.trim().is_empty() {
        let text_element = Element::new_text(current_text.trim().to_string());
        if let Some(current) = stack.last_mut() {
            current.children.push(text_element);
        } else {
            root.children.push(text_element);
        }
    }

    root
}

// Função para renderizar o HTML em uma interface GTK
fn render_html_to_gtk(element: &Element, container: &gtk::Box) {
    match element.tag.as_str() {
        "h1" => {
            let text = get_text_content(element);
            if !text.is_empty() {
                let label = gtk::Label::new(Some(&text));
                label.set_markup(&format!("<b><big>{}</big></b>", text));
                container.append(&label);
            }
        }
        "h2" => {
            let text = get_text_content(element);
            if !text.is_empty() {
                let label = gtk::Label::new(Some(&text));
                label.set_markup(&format!("<b><big>{}</big></b>", text));
                container.append(&label);
            }
        }
        "h3" => {
            let text = get_text_content(element);
            if !text.is_empty() {
                let label = gtk::Label::new(Some(&text));
                label.set_markup(&format!("<b>{}</b>", text));
                container.append(&label);
            }
        }
        "p" => {
            let text = get_text_content(element);
            if !text.is_empty() {
                let label = gtk::Label::new(Some(&text));
                container.append(&label);
            }
        }
        "text" => {
            if let Some(text) = &element.text_content {
                if !text.trim().is_empty() {
                    let label = gtk::Label::new(Some(text));
                    container.append(&label);
                }
            }
        }
        _ => {
            // Para outras tags, renderizar os filhos
            for child in &element.children {
                render_html_to_gtk(child, container);
            }
        }
    }
    
    // Renderizar filhos para tags que não são de texto
    if element.tag != "text" {
        for child in &element.children {
            render_html_to_gtk(child, container);
        }
    }
}

// Função auxiliar para extrair texto de um elemento
fn get_text_content(element: &Element) -> String {
    if let Some(text) = &element.text_content {
        return text.clone();
    }
    
    let mut text = String::new();
    for child in &element.children {
        if child.tag == "text" {
            if let Some(child_text) = &child.text_content {
                text.push_str(child_text);
            }
        } else {
            text.push_str(&get_text_content(child));
        }
    }
    text
}

fn main() {
    // Inicializar GTK
    gtk::init().expect("Failed to initialize GTK");

    // Criar janela principal
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Nami Browser");
    window.set_default_size(800, 600);
    window.connect_close_request(|_| {
        gtk::main_quit();
        glib::Propagation::Stop
    });

    // Criar container principal
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    window.set_child(Some(&main_box));

    // HTML de exemplo
    let html = r#"
        <html>
            <head>
                <meta charset="utf-8"/>
                <title>Nami Browser Test</title>
            </head>
            <body>
                <h1>Mugiwara no Luffy >> 🏴‍☠️ << Isso funciona!</h1>
                <h2>Segundo título</h2>
                <p>Este é um parágrafo de teste para verificar se o parser está funcionando corretamente.</p>
                <h3>Título menor</h3>
                <p>Outro parágrafo com <strong>texto em negrito</strong> e <em>texto em itálico</em>.</p>
            </body>
        </html>
    "#;

    // Parsear HTML
    let parsed_html = parse_html(html);
    println!("HTML Parseado:");
    println!("{:#?}", parsed_html);

    // Renderizar HTML na interface GTK
    render_html_to_gtk(&parsed_html, &main_box);

    // Mostrar janela
    window.present();

    // Executar loop principal do GTK
    gtk::main();
}