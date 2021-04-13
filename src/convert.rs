//! Convert markdown to Atlassian's markup format
//! Atlassian Document Format: https://developer.atlassian.com/cloud/jira/platform/apis/document/pub structure

#![allow(dead_code)]

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use serde::{Deserialize, Serialize};

fn append<T: Clone>(vec: Option<Vec<T>>, elem: T) -> Vec<T> {
    match vec {
        Some(vec) => {
            let mut new = vec.to_vec();
            new.push(elem);
            new
        }
        None => vec![elem],
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TableLayout {
    Default,
    #[serde(rename = "full-width")]
    FullWidth,
    Wide,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "attrs")]
pub enum BlockNodeType {
    // Block
    BlockQuote,
    BulletList,
    CodeBlock {
        language: Option<String>,
    },
    Heading {
        level: u32,
    },
    MediaGroup,
    MediaSingle,
    OrderedList,
    Panel,
    Paragraph,
    Rule,

    // Child block
    Table {
        is_number_column_enabled: bool,
        layout: TableLayout,
    },
    ListItem,
    Media,
    TableCell {
        background: String,
    },
    TableHeader,
    TableRow,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "attrs")]
pub enum InlineNodeType {
    Emoji {
        id: Option<String>,
        short_name: String,
        text: Option<String>,
    },
    HardBreak,
    InlineCard {
        url: String,
    },
    Mention {
        id: String,
        text: Option<String>,
        user_type: Option<String>,
    },
    Text,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SubsupType {
    Sup,
    Sub,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "attrs")]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    Code,
    Em,
    Link {
        href: String,
        title: Option<String>,
    },
    Strike,
    Strong,
    Subsup {
        #[serde(rename = "type")]
        subsuptype: SubsupType,
    },
    TextColor {
        color: String,
    },
    Underline,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Node {
    BlockNode {
        #[serde(rename = "type")]
        #[serde(flatten)]
        nodetype: BlockNodeType,
        content: Vec<Node>,
    },
    InlineNode {
        #[serde(rename = "type")]
        #[serde(flatten)]
        nodetype: InlineNodeType,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        marks: Option<Vec<Mark>>,
    },
    Root {
        version: usize,
        #[serde(rename = "type")]
        doctype: String,
        content: Vec<Node>,
    },
}

// TODO: Return a `Result` instead of panic-ing on unsupported CommonMark features
fn convert_node_markdown_to_adf<'a>(node: &'a AstNode<'a>, marks: Option<Vec<Mark>>) -> Node {
    match &node.data.borrow().value {
        NodeValue::Document => Node::Root {
            version: 1,
            doctype: "doc".to_owned(),
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::BlockQuote => Node::BlockNode {
            nodetype: BlockNodeType::BlockQuote,
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::List(list) => match list.list_type {
            comrak::nodes::ListType::Bullet => Node::BlockNode {
                nodetype: BlockNodeType::BulletList,
                content: node
                    .children()
                    .map(|c| convert_node_markdown_to_adf(c, None))
                    .collect(),
            },
            comrak::nodes::ListType::Ordered => Node::BlockNode {
                nodetype: BlockNodeType::OrderedList,
                content: node
                    .children()
                    .map(|c| convert_node_markdown_to_adf(c, None))
                    .collect(),
            },
        },
        NodeValue::Item(_) => Node::BlockNode {
            nodetype: BlockNodeType::ListItem,
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::DescriptionList => {
            panic!("Definition lists are unsupported!");
        }
        NodeValue::DescriptionItem(_) => {
            panic!("Definition lists are unsupported!");
        }
        NodeValue::DescriptionTerm => {
            panic!("Definition lists are unsupported!");
        }
        NodeValue::DescriptionDetails => {
            panic!("Definition lists are unsupported!");
        }
        NodeValue::CodeBlock(code) => Node::BlockNode {
            nodetype: BlockNodeType::CodeBlock {
                language: std::str::from_utf8(&code.info).ok().map(String::from),
            },
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::HtmlBlock(_) => {
            panic!("HTML blocks are unsupported!");
        }
        NodeValue::Paragraph => Node::BlockNode {
            nodetype: BlockNodeType::Paragraph,
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::Heading(heading) => Node::BlockNode {
            nodetype: BlockNodeType::Heading {
                level: heading.level,
            },
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::ThematicBreak => Node::BlockNode {
            nodetype: BlockNodeType::Rule,
            content: vec![],
        },
        NodeValue::FootnoteDefinition(_) => {
            panic!("Footnotes aren't supported!");
        }
        NodeValue::Table(_) => Node::BlockNode {
            nodetype: BlockNodeType::Table {
                is_number_column_enabled: false,
                layout: TableLayout::Default,
            },
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::TableRow(_) => Node::BlockNode {
            nodetype: BlockNodeType::TableRow,
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::TableCell => Node::BlockNode {
            nodetype: BlockNodeType::TableCell {
                background: "#ffffff".to_owned(),
            },
            content: node
                .children()
                .map(|c| convert_node_markdown_to_adf(c, None))
                .collect(),
        },
        NodeValue::Text(text) => Node::InlineNode {
            nodetype: InlineNodeType::Text,
            text: Some(
                std::str::from_utf8(&text[..])
                    .expect("Invalid UTF-8!")
                    .to_owned(),
            ),
            marks,
        },
        NodeValue::TaskItem(_) => {
            panic!("Task lists not supported!");
        }
        NodeValue::SoftBreak => Node::InlineNode {
            nodetype: InlineNodeType::Text,
            text: Some(" ".to_owned()),
            marks: None,
        },
        NodeValue::LineBreak => Node::InlineNode {
            nodetype: InlineNodeType::HardBreak,
            marks: None,
            text: None,
        },
        NodeValue::Code(text) => Node::InlineNode {
            nodetype: InlineNodeType::Text,
            text: Some(
                std::str::from_utf8(&text[..])
                    .expect("Invalid UTF-8!")
                    .to_owned(),
            ),
            marks: Some(vec![Mark::Code]),
        },
        NodeValue::HtmlInline(_) => {
            panic!("Raw HTML not supported!");
        }
        NodeValue::Emph => {
            let marks = append(marks, Mark::Em);
            convert_node_markdown_to_adf(
                node.first_child().expect("Nothing to emphasize!"),
                Some(marks),
            )
        }
        NodeValue::Strong => {
            let marks = append(marks, Mark::Strong);
            convert_node_markdown_to_adf(
                node.first_child().expect("Nothing to embolden!"),
                Some(marks),
            )
        }
        NodeValue::Strikethrough => {
            let marks = append(marks, Mark::Strike);
            convert_node_markdown_to_adf(
                node.first_child().expect("Nothing to strikethrough!"),
                Some(marks),
            )
        }
        NodeValue::Superscript => {
            let mark = Mark::Subsup {
                subsuptype: SubsupType::Sup,
            };
            let marks = append(marks, mark);
            convert_node_markdown_to_adf(
                node.first_child().expect("Nothing to superscript!"),
                Some(marks),
            )
        }
        NodeValue::Link(link) => {
            let mark = Mark::Link {
                href: std::str::from_utf8(&link.url[..])
                    .expect("Invalid UTF-8!")
                    .to_owned(),
                title: Some(
                    std::str::from_utf8(&link.title[..])
                        .expect("Invalid UTF-8!")
                        .to_owned(),
                ),
            };

            let marks = append(marks, mark);
            convert_node_markdown_to_adf(
                node.first_child().expect("Nothing to superscript!"),
                Some(marks),
            )
        }
        NodeValue::Image(_) => {
            panic!("Images aren't supported, sorry!");
        }
        NodeValue::FootnoteReference(_) => {
            panic!("Footnotes aren't supported!");
        }
        NodeValue::FrontMatter(_) => {
            panic!("Front matter isn't supported!");
        }
    }
}

pub fn markdown_to_adf(text: &str) -> Node {
    let arena = Arena::new();

    let root = parse_document(&arena, text, &ComrakOptions::default());

    convert_node_markdown_to_adf(root, None)
}
