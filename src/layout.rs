//! Layout em bloco + quebra de linha por largura (caracteres).

use crate::html::Element;

pub fn is_block_tag(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "body"
            | "div"
            | "dl"
            | "dt"
            | "dd"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "html"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
    )
}

fn extract_text_inline(element: &Element) -> String {
    if element.tag == "text" {
        return element.text_content.clone().unwrap_or_default();
    }
    let mut s = String::new();
    for child in &element.children {
        s.push_str(&extract_text_inline(child));
    }
    s
}

/// Quebra texto em linhas com até `max_chars` caracteres por linha (aproximação por palavras).
pub fn word_wrap(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return text.to_string();
    }
    let text = text.trim();
    if text.is_empty() {
        return String::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }
        if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            if word.len() <= max_chars {
                current.push_str(word);
            } else {
                for chunk in word.chars().collect::<Vec<_>>().chunks(max_chars) {
                    lines.push(chunk.iter().collect());
                }
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

fn format_block_heading(level: u8, text: &str, wrap_width: usize) -> String {
    let wrapped = word_wrap(text, wrap_width);
    let underline = match level {
        1 => "=",
        2 => "-",
        _ => "·",
    };
    let bar: String = underline.repeat(wrap_width.min(60).max(8));
    format!("\n{wrapped}\n{bar}\n")
}

fn visit_for_layout(element: &Element, out: &mut String, wrap_width: usize) {
    let tag = element.tag.as_str();
    if tag == "text" || tag == "root" {
        if tag == "text" {
            if let Some(t) = &element.text_content {
                if !t.trim().is_empty() {
                    out.push_str(t);
                }
            }
        }
        for child in &element.children {
            visit_for_layout(child, out, wrap_width);
        }
        return;
    }

    if tag == "head" || tag == "script" || tag == "style" || tag == "noscript" {
        return;
    }

    if is_block_tag(tag) {
        match tag {
            "h1" => {
                let t = extract_text_inline(element);
                if !t.is_empty() {
                    out.push_str(&format_block_heading(1, &t, wrap_width));
                }
            }
            "h2" => {
                let t = extract_text_inline(element);
                if !t.is_empty() {
                    out.push_str(&format_block_heading(2, &t, wrap_width));
                }
            }
            "h3" | "h4" | "h5" | "h6" => {
                let t = extract_text_inline(element);
                if !t.is_empty() {
                    out.push_str(&format_block_heading(3, &t, wrap_width));
                }
            }
            "p" | "blockquote" | "li" => {
                let t = extract_text_inline(element);
                if !t.is_empty() {
                    out.push_str(&word_wrap(&t, wrap_width));
                    out.push_str("\n\n");
                }
            }
            "br" | "hr" => {
                out.push('\n');
            }
            _ => {
                if tag == "html" || tag == "body" {
                    out.push('\n');
                }
                for child in &element.children {
                    visit_for_layout(child, out, wrap_width);
                }
                if tag == "html" || tag == "body" {
                    out.push('\n');
                }
            }
        }
        return;
    }

    for child in &element.children {
        visit_for_layout(child, out, wrap_width);
    }
}

/// Documento legível: blocos separados + wrap por largura em caracteres.
pub fn layout_document(root: &Element, wrap_width_chars: usize) -> String {
    let mut out = String::new();
    visit_for_layout(root, &mut out, wrap_width_chars);
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_respects_width() {
        let s = word_wrap("aaa bbb ccc ddd", 4);
        assert!(s.contains('\n'));
    }
}
