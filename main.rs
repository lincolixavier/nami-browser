use std::collections::HashMap;

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Element>,
    pub text_content: Option<String>,  // Add text content field
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
}

pub fn parse_html(html: &str) -> Element {
    let mut root = Element::new("root".to_string());
    let mut stack: Vec<Element> = Vec::new();
    let mut current_tag = String::new();
    let mut current_text = String::new();
    let mut current_attribute_name = String::new();
    let mut current_attribute_value = String::new();
    let mut is_tag_open = false;
    let mut is_closing_tag = false;
    let mut is_self_closing = false;

    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => {
                // Save any accumulated text content
                if !current_text.trim().is_empty() {
                    if let Some(current) = stack.last_mut() {
                        current.text_content = Some(current_text.trim().to_string());
                    }
                    current_text.clear();
                }
                
                if chars.peek() == Some(&'/') {
                    chars.next();
                    is_closing_tag = true;
                }
                is_tag_open = true;
            }
            '>' => {
                if is_closing_tag {
                    if !stack.is_empty() {
                        let closed_element = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(closed_element);
                        } else {
                            root.children.push(closed_element);
                        }
                    }
                    is_closing_tag = false;
                } else if is_self_closing {
                    let new_element = Element::new(current_tag.clone());
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(new_element);
                    } else {
                        root.children.push(new_element);
                    }
                    is_self_closing = false;
                } else {
                    let new_element = Element::new(current_tag.clone());
                    stack.push(new_element);
                }
                current_tag.clear();
                is_tag_open = false;
            }
            '/' if is_tag_open && !current_tag.is_empty() => {
                is_self_closing = true;
            }
            ' ' if is_tag_open && !current_tag.is_empty() => {
                // Handle attributes
                while let Some(&next_char) = chars.peek() {
                    if next_char == '>' {
                        break;
                    }
                    chars.next();
                    
                    if !next_char.is_whitespace() {
                        current_attribute_name.push(next_char);
                    }
                    
                    if next_char == '=' {
                        while let Some(&value_char) = chars.peek() {
                            if value_char == '"' || value_char == '\'' {
                                chars.next();
                                continue;
                            }
                            if value_char == ' ' || value_char == '>' {
                                break;
                            }
                            chars.next();
                            current_attribute_value.push(value_char);
                        }
                        
                        if !current_attribute_name.is_empty() && !current_attribute_value.is_empty() {
                            if let Some(current) = stack.last_mut() {
                                current.attributes.insert(
                                    current_attribute_name.clone(),
                                    current_attribute_value.clone(),
                                );
                            }
                        }
                        current_attribute_name.clear();
                        current_attribute_value.clear();
                    }
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

    root
}

fn main() {
    let html = r#"
        <html>
            <meta charset="utf-8"/>
            <body>
                <h1>Mugiwara no Luffy >> 🏴‍☠️ << isso não funcionar</h1>
            </body>
        </html>
    "#;
    let parsed_html = parse_html(html);
    println!("{:#?}", parsed_html);
}
