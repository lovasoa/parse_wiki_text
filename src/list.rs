// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_list_end_of_line(state: &mut ::State) {
    let item_end_position = state.skip_whitespace_backwards(state.scan_position);
    state.flush(item_end_position);
    state.scan_position += 1;
    let mut level = 0;
    for open_node in &state.stack {
        match open_node.type_ {
            ::OpenNodeType::Table { .. } | ::OpenNodeType::Tag { .. } => level += 1,
            _ => break,
        }
    }
    let start_level = level;
    let mut term_level = None;
    while level < state.stack.len() {
        match (
            &state.stack[level].type_,
            state.get_byte(state.scan_position),
        ) {
            (::OpenNodeType::DefinitionList { .. }, Some(b':'))
            | (::OpenNodeType::OrderedList { .. }, Some(b'#'))
            | (::OpenNodeType::UnorderedList { .. }, Some(b'*')) => {
                level += 1;
                state.scan_position += 1;
            }
            (::OpenNodeType::DefinitionList { .. }, Some(b';')) => {
                if term_level.is_none() {
                    term_level = Some(level);
                }
                level += 1;
                state.scan_position += 1;
            }
            _ => break,
        }
    }
    if let Some(term_level) = term_level {
        if level < state.stack.len() || match state.get_byte(state.scan_position) {
            Some(b'#') | Some(b'*') | Some(b':') | Some(b';') => true,
            _ => false,
        } {
            state.scan_position -= level - term_level;
            level = term_level;
            state.warnings.push(::Warning {
                end: state.scan_position,
                message: ::WarningMessage::DefinitionTermContinuation,
                start: state.scan_position - 1,
            });
        }
    }
    while level < state.stack.len() {
        let open_node = state.stack.pop().unwrap();
        let node = match open_node.type_ {
            ::OpenNodeType::DefinitionList { mut items } => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, open_node.nodes);
                }
                ::Node::DefinitionList {
                    end: item_end_position,
                    items,
                    start: open_node.start,
                }
            }
            ::OpenNodeType::OrderedList { mut items } => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, open_node.nodes);
                }
                ::Node::OrderedList {
                    end: item_end_position,
                    items,
                    start: open_node.start,
                }
            }
            ::OpenNodeType::UnorderedList { mut items } => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, open_node.nodes);
                }
                ::Node::UnorderedList {
                    end: item_end_position,
                    items,
                    start: open_node.start,
                }
            }
            _ => unreachable!(),
        };
        state.nodes.push(node);
    }
    state.flushed_position = state.scan_position;
    if parse_list_item_start(state) {
        while parse_list_item_start(state) {}
        skip_spaces(state);
    } else if level > start_level {
        match state.stack.get_mut(level - 1) {
            Some(::OpenNode {
                type_: ::OpenNodeType::DefinitionList { items },
                ..
            }) => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, vec![]);
                }
                items.push(::DefinitionListItem {
                    end: 0,
                    nodes: vec![],
                    start: state.scan_position - 1,
                    type_: if state
                        .wiki_text
                        .as_bytes()
                        .get(state.scan_position - 1)
                        .cloned() == Some(b';')
                    {
                        ::DefinitionListItemType::Term
                    } else {
                        ::DefinitionListItemType::Details
                    },
                });
            }
            Some(::OpenNode {
                type_: ::OpenNodeType::OrderedList { items },
                ..
            }) => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, vec![]);
                };
                items.push(::ListItem {
                    end: 0,
                    nodes: vec![],
                    start: state.scan_position - 1,
                });
            }
            Some(::OpenNode {
                type_: ::OpenNodeType::UnorderedList { items },
                ..
            }) => {
                {
                    let item_index = items.len() - 1;
                    let last_item = &mut items[item_index];
                    last_item.end = item_end_position;
                    last_item.nodes = ::std::mem::replace(&mut state.nodes, vec![]);
                };
                items.push(::ListItem {
                    end: 0,
                    nodes: vec![],
                    start: state.scan_position - 1,
                });
            }
            _ => unreachable!(),
        }
        skip_spaces(state);
    } else {
        state.skip_empty_lines();
    }
}

pub fn parse_list_item_start(state: &mut ::State) -> bool {
    let open_node_type = match state.get_byte(state.scan_position) {
        Some(b'#') => ::OpenNodeType::OrderedList {
            items: vec![::ListItem {
                end: 0,
                nodes: vec![],
                start: state.scan_position + 1,
            }],
        },
        Some(b'*') => ::OpenNodeType::UnorderedList {
            items: vec![::ListItem {
                end: 0,
                nodes: vec![],
                start: state.scan_position + 1,
            }],
        },
        Some(b':') => ::OpenNodeType::DefinitionList {
            items: vec![::DefinitionListItem {
                end: 0,
                nodes: vec![],
                start: state.scan_position + 1,
                type_: ::DefinitionListItemType::Details,
            }],
        },
        Some(b';') => ::OpenNodeType::DefinitionList {
            items: vec![::DefinitionListItem {
                end: 0,
                nodes: vec![],
                start: state.scan_position + 1,
                type_: ::DefinitionListItemType::Term,
            }],
        },
        _ => return false,
    };
    let position = state.scan_position + 1;
    state.push_open_node(open_node_type, position);
    true
}

pub fn skip_spaces(state: &mut ::State) {
    while match state.get_byte(state.scan_position) {
        Some(b'\t') | Some(b' ') => true,
        _ => false,
    } {
        state.scan_position += 1;
    }
    state.flushed_position = state.scan_position;
}
