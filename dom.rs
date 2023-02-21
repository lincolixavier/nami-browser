use std::collections::HashMap;

#[derive(Debug)]
pub struct Element{
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Element>
}

pub fn parse_html(html: &str) -> Element {
    let mut root = Element {
        tag: String::new(),
        attributes: HashMap::new(),
        children: Vec::new(),
    };
    let mut current_element = &mut root;
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut attribute_name = String::new();
    let mut attribute_value =  String::new();
    let mut escape_next = false;

    for character in html.chars(){
        match character {
            '>' if in_tag => {
                in_tag = false;
                current_element.tag = tag_name;
                tag_name = String::new();
                if !attribute_name.is_empty() {
                    current_element
                        .attributes
                        .insert(attribute_name, attribute_value);
                    attribute_name = String::new();
                    attribute_value = String::new();
                }
            }
            ' ' | '/' if in_tag => {
                current_element.tag = tag_name;
                tag_name = String::new();
                if !attribute_name.is_empty() {
                    current_element
                        .attributes
                        .insert(attribute_name, attribute_value);
                    attribute_name = String::new();
                    attribute_value = String::new(); 
                }
            }
            '\'' | '"' if in_tag => { escape_next = true } 
            '=' if in_tag => {
                attribute_value = String::new();
            }
            _ if in_tag => {
                if attribute_name.is_empty() {
                    tag_name.push(character);
                } else if !escape_next {
                    attribute_value.push(character);
                } else {
                    escape_next = false;
                    attribute_value.push(character);
                }
            }
            '<' => in_tag = true,
            _ => current_element.children.push(Element {
                tag: character.to_string(),
                attributes: HashMap::new(),
                children: Vec::new(),
            }),
        }
        
    }
    root
}
