use crate::compiler::{
    compile::utils::extract_svelte_ignore::extract_svelte_ignore,
    interfaces::{BaseNode, Comment, TemplateNode},
    parse::{
        errors::Error,
        index::{Parser, StateReturn},
    },
};
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref VALID_TAG_NAME: Regex = Regex::new("^!?[a-zA-Z]{1,}:?[a-zA-Z0-9-]*").unwrap();
    static ref META_TAGS: HashMap<&'static str, &'static str> = HashMap::from([
        ("svelte:head", "Head"),
        ("svelte:options", "Options"),
        ("svelte:window", "Window"),
        ("svelte:body", "Body")
    ]);

    /// `regex` doesn't support lookahead so make sure to remove
    /// the last character
    static ref SELF: Regex = Regex::new("^svelte:self([\\s/>])").unwrap();

    /// `regex` doesn't support lookahead so make sure to remove
    /// the last character
    static ref COMPONENT: Regex = Regex::new("^svelte:component([\\s/>])").unwrap();

    /// `regex` doesn't support lookahead so make sure to remove
    /// the last character
    static ref SLOT: Regex = Regex::new("^svelte:fragment([\\s/>])").unwrap();

    /// `regex` doesn't support lookahead so make sure to remove
    /// the last character
    static ref ELEMENT: Regex = Regex::new("^svelte:element([\\s/>])").unwrap();
}

pub const VALID_META_TAGS: [&'static str; 4] = [
    "svelte:self",
    "svelte:component",
    "svelte:fragment",
    "svelte:element",
];

pub fn parent_is_head(stack: Vec<TemplateNode>) -> bool {
    for i in (0..stack.len()).rev() {
        if let Some(temp) = stack.iter().nth(i) {
            let temp_type = &temp.get_type();

            if temp_type == "Head" {
                return true;
            }
            if temp_type == "Element" || temp_type == "InlineComponent" {
                return false;
            }
        }
    }

    false
}

pub fn tag(parser: &mut Parser) -> StateReturn {
    let start = parser.index + 1;
    let parent = parser.current();

    if parser.eat("!--", false, None) {
        let data = parser.read_until(Regex::new("-->").unwrap(), None);
        parser.eat("-->", true, Some(Error::unclosed_comment()));

        let index = parser.index;
        parser
            .current()
            .get_children()
            .push(TemplateNode::Comment(Comment {
                base_node: BaseNode {
                    start: Some(start),
                    end: Some(index),
                    node_type: "Comment".to_string(),
                    children: Vec::new(),
                    prop_name: HashMap::new(),
                    expression: None,
                    elseif: false,
                    _else: false,
                },
                data: data.clone(),
                ignores: extract_svelte_ignore(&data),
            }));

        return StateReturn::None;
    }

    let is_closing_tag = parser.eat("/", false, None);
    let name = read_tag_name(parser);

    if META_TAGS.contains_key(name.as_str()) {
        let slug = META_TAGS.get(name.as_str()).unwrap().to_lowercase();
        if is_closing_tag {
            if (name == "svelte:window" || name == "svelte:body")
                && parser.current().get_children().len() > 0
            {
                let error = Error::invalid_element_content(&slug, &name);
                let index = parser
                    .current()
                    .get_children()
                    .first_mut()
                    .unwrap()
                    .get_base_node()
                    .start;
                parser.error(&error.code, &error.message, index);
            } else {
                if parser.meta_tags.contains_key(&name) {
                    let error = Error::duplicate_element(&slug, &name);
                    parser.error(&error.code, &error.message, Some(start));
                }

                if parser.stack.len() > 1 {
                    let error = Error::invalid_element_placement(&slug, &name);
                    parser.error(&error.code, &error.message, Some(start));
                }

                parser.meta_tags.insert(name, true);
            }
        }
    }

    // continues at line 105

    todo!()
}

pub fn read_tag_name(parser: &mut Parser) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{COMPONENT, ELEMENT, SELF, SLOT, VALID_TAG_NAME};

    #[test]
    fn test_valid_tag_name_regex() {
        let samples = vec![
            "svelte:options",
            "svelte:component",
            "my-element",
            "svelte:my-element",
            "!svelte:element",
            "element-multiple-dashes",
            "!svelte:multiple-dashes-element",
        ];

        for s in samples {
            assert!(VALID_TAG_NAME.is_match(s));
        }
    }

    #[test]
    fn test_self_regex() {
        let samples = vec!["svelte:self ", "svelte:self/", "svelte:self>"];

        for s in samples {
            s.ends_with("/>");
            assert!(SELF.is_match(s));
        }
    }

    #[test]
    fn test_component_regex() {
        let samples = vec![
            "svelte:component ",
            "svelte:component/",
            "svelte:component>",
        ];

        for s in samples {
            s.ends_with("/>");
            assert!(COMPONENT.is_match(s));
        }
    }

    #[test]
    fn test_slot_regex() {
        let samples = vec!["svelte:fragment ", "svelte:fragment/", "svelte:fragment>"];

        for s in samples {
            s.ends_with("/>");
            assert!(SLOT.is_match(s));
        }
    }

    #[test]
    fn test_element_regex() {
        let samples = vec!["svelte:element ", "svelte:element/", "svelte:element>"];

        for s in samples {
            s.ends_with("/>");
            assert!(ELEMENT.is_match(s));
        }
    }
}
