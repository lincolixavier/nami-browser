//! Tokenizer mínimo + tree builder em pilha (tags, atributos, void/self-closing, texto).

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Text(String),
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
}

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Element>,
    pub text_content: Option<String>,
}

impl Element {
    pub fn new(tag: String) -> Self {
        Element {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: None,
        }
    }

    pub fn new_text(text: String) -> Self {
        Element {
            tag: "text".to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: Some(text),
        }
    }
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn skip_ascii_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c.is_ascii_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_tag_name(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            name.push(c.to_ascii_lowercase());
            chars.next();
        } else {
            break;
        }
    }
    name
}

fn parse_attr_value(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    skip_ascii_whitespace(chars);
    let Some(&q) = chars.peek() else {
        return String::new();
    };
    if q == '"' || q == '\'' {
        chars.next();
        let mut v = String::new();
        for c in chars.by_ref() {
            if c == q {
                break;
            }
            v.push(c);
        }
        return decode_entities(&v);
    }
    let mut v = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_whitespace() || c == '>' || c == '/' {
            break;
        }
        v.push(chars.next().unwrap());
    }
    decode_entities(&v)
}

fn parse_attributes(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> (HashMap<String, String>, bool) {
    let mut attrs = HashMap::new();
    let mut self_closing = false;
    loop {
        skip_ascii_whitespace(chars);
        let Some(&c) = chars.peek() else {
            break;
        };
        if c == '/' {
            chars.next();
            if chars.peek() == Some(&'>') {
                chars.next();
                self_closing = true;
            }
            break;
        }
        if c == '>' {
            chars.next();
            break;
        }
        let name = parse_tag_name(chars);
        if name.is_empty() {
            chars.next();
            continue;
        }
        skip_ascii_whitespace(chars);
        if chars.peek() == Some(&'=') {
            chars.next();
            let value = parse_attr_value(chars);
            attrs.insert(name, value);
        } else {
            attrs.insert(name, String::new());
        }
    }
    (attrs, self_closing)
}

fn skip_bang_declaration(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    if chars.peek() == Some(&'-') {
        chars.next();
        if chars.peek() == Some(&'-') {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '-' && chars.peek() == Some(&'-') {
                    chars.next();
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        break;
                    }
                }
            }
        }
        return;
    }
    for c in chars.by_ref() {
        if c == '>' {
            break;
        }
    }
}

pub fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut entity = String::new();
        let mut found_semicolon = false;
        while let Some(ec) = chars.next() {
            if ec == ';' {
                found_semicolon = true;
                break;
            }
            if entity.len() > 32 {
                break;
            }
            entity.push(ec);
        }
        if !found_semicolon {
            out.push('&');
            out.push_str(&entity);
            continue;
        }
        match entity.as_str() {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" | "#39" => out.push('\''),
            s if s.starts_with("#x") || s.starts_with("#X") => {
                let hex = &s[2..];
                if let Ok(u) = u32::from_str_radix(hex, 16) {
                    if let Some(ch) = char::from_u32(u) {
                        out.push(ch);
                    }
                } else {
                    out.push('&');
                    out.push_str(&entity);
                    out.push(';');
                }
            }
            s if s.starts_with('#') => {
                let dec = &s[1..];
                if let Ok(u) = dec.parse::<u32>() {
                    if let Some(ch) = char::from_u32(u) {
                        out.push(ch);
                    }
                } else {
                    out.push('&');
                    out.push_str(&entity);
                    out.push(';');
                }
            }
            _ => {
                out.push('&');
                out.push_str(&entity);
                out.push(';');
            }
        }
    }
    out
}

pub fn tokenize(html: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = html.chars().peekable();
    let mut text_buf = String::new();

    fn flush_text(buf: &mut String, tokens: &mut Vec<Token>) {
        if !buf.is_empty() {
            tokens.push(Token::Text(decode_entities(buf)));
            buf.clear();
        }
    }

    while let Some(c) = chars.next() {
        if c != '<' {
            text_buf.push(c);
            continue;
        }
        flush_text(&mut text_buf, &mut tokens);

        if chars.peek() == Some(&'!') {
            chars.next();
            skip_bang_declaration(&mut chars);
            continue;
        }
        if chars.peek() == Some(&'?') {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '>' {
                    break;
                }
            }
            continue;
        }

        let closing = chars.peek() == Some(&'/');
        if closing {
            chars.next();
        }
        skip_ascii_whitespace(&mut chars);
        let name = parse_tag_name(&mut chars);
        if name.is_empty() {
            text_buf.push('<');
            if closing {
                text_buf.push('/');
            }
            continue;
        }

        if closing {
            skip_ascii_whitespace(&mut chars);
            while let Some(c) = chars.next() {
                if c == '>' {
                    break;
                }
            }
            tokens.push(Token::EndTag { name });
            continue;
        }

        let (attributes, mut self_closing) = parse_attributes(&mut chars);
        if is_void_element(&name) {
            self_closing = true;
        }
        tokens.push(Token::StartTag {
            name,
            attributes,
            self_closing,
        });
    }

    flush_text(&mut text_buf, &mut tokens);
    tokens
}

/// Constrói árvore: pilha de nós abertos; texto e void entram no nó atual.
pub fn build_tree(tokens: &[Token]) -> Element {
    let mut stack: Vec<Element> = vec![Element::new("root".to_string())];

    for token in tokens {
        match token {
            Token::Text(t) => {
                if t.is_empty() {
                    continue;
                }
                let trimmed = t.trim_matches(|c: char| c.is_ascii_whitespace());
                if trimmed.is_empty() {
                    continue;
                }
                let text_el = Element::new_text(t.clone());
                if let Some(open) = stack.last_mut() {
                    open.children.push(text_el);
                }
            }
            Token::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                let mut el = Element::new(name.clone());
                el.attributes = attributes.clone();
                let void_or_sc = *self_closing || is_void_element(name);
                if void_or_sc {
                    if let Some(open) = stack.last_mut() {
                        open.children.push(el);
                    }
                } else {
                    stack.push(el);
                }
            }
            Token::EndTag { name } => {
                let target = name.to_ascii_lowercase();
                if stack.len() <= 1 {
                    continue;
                }
                if stack.last().map(|e| e.tag.to_ascii_lowercase()) == Some(target.clone()) {
                    let node = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    }
                    continue;
                }
                while stack.len() > 1 {
                    let top = stack.last().unwrap().tag.to_ascii_lowercase();
                    if top == target {
                        let node = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(node);
                        }
                        break;
                    }
                    if stack.len() == 1 {
                        break;
                    }
                    let orphan = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(orphan);
                    }
                }
            }
        }
    }

    while stack.len() > 1 {
        let node = stack.pop().unwrap();
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        }
    }

    stack.pop().unwrap_or_else(|| Element::new("root".to_string()))
}

pub fn parse_html(html: &str) -> Element {
    let tokens = tokenize(html);
    build_tree(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_self_closing_does_not_break_stack() {
        let html = r#"<html><head><meta charset="utf-8"/><title>T</title></head><body><p>Hi</p></body></html>"#;
        let root = parse_html(html);
        assert!(root.children.iter().any(|c| c.tag == "html"));
    }

    #[test]
    fn tag_name_not_merged_with_attrs() {
        let html = r#"<meta name="viewport" content="w"/>"#;
        let root = parse_html(html);
        let meta = root
            .children
            .iter()
            .find(|c| c.tag == "meta")
            .expect("meta");
        assert_eq!(meta.attributes.get("name").map(String::as_str), Some("viewport"));
    }
}
