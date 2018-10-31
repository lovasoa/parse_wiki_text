// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub struct OpenNode<'a> {
    pub nodes: Vec<::Node<'a>>,
    pub start: usize,
    pub type_: OpenNodeType<'a>,
}

pub enum OpenNodeType<'a> {
    DefinitionList {
        items: Vec<::DefinitionListItem<'a>>,
    },
    ExternalLink,
    Heading {
        level: u8,
    },
    Link {
        namespace: Option<::Namespace>,
        target: &'a str,
    },
    OrderedList {
        items: Vec<::ListItem<'a>>,
    },
    Parameter {
        default: Option<Vec<::Node<'a>>>,
        name: Option<Vec<::Node<'a>>>,
    },
    Preformatted,
    Table(Table<'a>),
    Tag {
        name: ::Cow<'a, str>,
    },
    Template {
        name: Option<Vec<::Node<'a>>>,
        parameters: Vec<::Parameter<'a>>,
    },
    UnorderedList {
        items: Vec<::ListItem<'a>>,
    },
}

pub struct State<'a> {
    pub flushed_position: usize,
    pub nodes: Vec<::Node<'a>>,
    pub scan_position: usize,
    pub stack: Vec<OpenNode<'a>>,
    pub warnings: Vec<::Warning>,
    pub wiki_text: &'a str,
}

pub struct Table<'a> {
    pub attributes: Vec<::Node<'a>>,
    pub before: Vec<::Node<'a>>,
    pub captions: Vec<::TableCaption<'a>>,
    pub child_element_attributes: Option<Vec<::Node<'a>>>,
    pub rows: Vec<::TableRow<'a>>,
    pub start: usize,
    pub state: TableState,
}

pub enum TableState {
    Before,
    CaptionFirstLine,
    CaptionRemainder,
    CellFirstLine,
    CellRemainder,
    HeadingFirstLine,
    HeadingRemainder,
    Row,
    TableAttributes,
}

impl<'a> State<'a> {
    pub fn flush(&mut self, end_position: usize) {
        flush(
            &mut self.nodes,
            self.flushed_position,
            end_position,
            self.wiki_text,
        );
    }

    pub fn get_byte(&self, position: usize) -> Option<u8> {
        self.wiki_text.as_bytes().get(position).cloned()
    }

    pub fn push_open_node(&mut self, type_: OpenNodeType<'a>, inner_start_position: usize) {
        let scan_position = self.scan_position;
        self.flush(scan_position);
        self.stack.push(OpenNode {
            nodes: ::std::mem::replace(&mut self.nodes, vec![]),
            start: scan_position,
            type_,
        });
        self.scan_position = inner_start_position;
        self.flushed_position = inner_start_position;
    }

    pub fn rewind(&mut self, nodes: Vec<::Node<'a>>, position: usize) {
        self.scan_position = position + 1;
        self.nodes = nodes;
        if let Some(position_before_text) = match self.nodes.last() {
            Some(::Node::Text { start, .. }) => Some(*start),
            _ => None,
        } {
            self.nodes.pop();
            self.flushed_position = position_before_text;
        } else {
            self.flushed_position = position;
        }
    }

    pub fn skip_empty_lines(&mut self) {
        match self.stack.last() {
            Some(OpenNode {
                type_: OpenNodeType::Table { .. },
                ..
            }) => {
                self.scan_position -= 1;
                ::table::parse_table_end_of_line(self, false);
            }
            _ => {
                ::line::parse_beginning_of_line(self, None);
            }
        }
    }

    pub fn skip_whitespace_backwards(&self, position: usize) -> usize {
        skip_whitespace_backwards(self.wiki_text, position)
    }

    pub fn skip_whitespace_forwards(&self, position: usize) -> usize {
        skip_whitespace_forwards(self.wiki_text, position)
    }
}

pub fn flush<'a>(
    nodes: &mut Vec<::Node<'a>>,
    flushed_position: usize,
    end_position: usize,
    wiki_text: &'a str,
) {
    if end_position > flushed_position {
        nodes.push(::Node::Text {
            end: end_position,
            start: flushed_position,
            value: &wiki_text[flushed_position..end_position],
        });
    }
}

pub fn skip_whitespace_backwards(wiki_text: &str, mut position: usize) -> usize {
    while position > 0 && match wiki_text.as_bytes()[position - 1] {
        b'\t' | b'\n' | b' ' => true,
        _ => false,
    } {
        position -= 1;
    }
    position
}

pub fn skip_whitespace_forwards(wiki_text: &str, mut position: usize) -> usize {
    while match wiki_text.as_bytes().get(position).cloned() {
        Some(b'\t') | Some(b'\n') | Some(b' ') => true,
        _ => false,
    } {
        position += 1;
    }
    position
}
