// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_parameter_name_end(state: &mut ::State) {
    let stack_length = state.stack.len();
    if stack_length > 0 {
        if let ::OpenNode {
            type_:
                ::OpenNodeType::Template {
                    name: Some(_),
                    parameters,
                },
            ..
        } = &mut state.stack[stack_length - 1]
        {
            let parameters_length = parameters.len();
            let name = &mut parameters[parameters_length - 1].name;
            if name.is_none() {
                ::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    ::state::skip_whitespace_backwards(state.wiki_text, state.scan_position),
                    state.wiki_text,
                );
                state.flushed_position =
                    ::state::skip_whitespace_forwards(state.wiki_text, state.scan_position + 1);
                state.scan_position = state.flushed_position;
                *name = Some(::std::mem::replace(&mut state.nodes, vec![]));
                return;
            }
        }
    }
    state.scan_position += 1;
}

pub fn parse_parameter_separator(state: &mut ::State) {
    match state.stack.last_mut() {
        Some(::OpenNode {
            type_: ::OpenNodeType::Parameter { default, name },
            ..
        }) => {
            if name.is_none() {
                let position =
                    ::state::skip_whitespace_backwards(state.wiki_text, state.scan_position);
                ::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    position,
                    state.wiki_text,
                );
                *name = Some(::std::mem::replace(&mut state.nodes, vec![]));
            } else {
                ::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    state.scan_position,
                    state.wiki_text,
                );
                *default = Some(::std::mem::replace(&mut state.nodes, vec![]));
                state.warnings.push(::Warning {
                    end: state.scan_position + 1,
                    message: ::WarningMessage::UselessTextInParameter,
                    start: state.scan_position,
                });
            }
            state.scan_position += 1;
            state.flushed_position = state.scan_position;
        }
        _ => unreachable!(),
    }
}

pub fn parse_template_end(state: &mut ::State) {
    match state.stack.pop() {
        Some(::OpenNode {
            nodes,
            start,
            type_: ::OpenNodeType::Parameter { default, name },
        }) => if state.get_byte(state.scan_position + 2) == Some(b'}') {
            if let Some(name) = name {
                let start_position = state.scan_position;
                state.flush(start_position);
                let nodes = ::std::mem::replace(&mut state.nodes, nodes);
                state.nodes.push(::Node::Parameter {
                    default: Some(default.unwrap_or(nodes)),
                    end: state.scan_position,
                    name,
                    start,
                });
            } else {
                let start_position = state.skip_whitespace_backwards(state.scan_position);
                state.flush(start_position);
                let nodes = ::std::mem::replace(&mut state.nodes, nodes);
                state.nodes.push(::Node::Parameter {
                    default: None,
                    end: state.scan_position,
                    name: nodes,
                    start,
                });
            }
            state.scan_position += 3;
            state.flushed_position = state.scan_position;
        } else {
            state.warnings.push(::Warning {
                end: state.scan_position + 2,
                message: ::WarningMessage::UnexpectedEndTagRewinding,
                start: state.scan_position,
            });
            state.rewind(nodes, start);
        },
        Some(::OpenNode {
            nodes,
            start,
            type_:
                ::OpenNodeType::Template {
                    name,
                    mut parameters,
                },
        }) => {
            let position = state.skip_whitespace_backwards(state.scan_position);
            state.flush(position);
            state.scan_position += 2;
            state.flushed_position = state.scan_position;
            let name = match name {
                None => ::std::mem::replace(&mut state.nodes, nodes),
                Some(name) => {
                    let parameters_length = parameters.len();
                    let parameter = &mut parameters[parameters_length - 1];
                    parameter.end = position;
                    parameter.value = ::std::mem::replace(&mut state.nodes, nodes);
                    name
                }
            };
            state.nodes.push(::Node::Template {
                end: state.scan_position,
                name,
                parameters,
                start,
            });
        }
        Some(::OpenNode { nodes, start, .. }) => {
            state.warnings.push(::Warning {
                end: state.scan_position + 2,
                message: ::WarningMessage::UnexpectedEndTagRewinding,
                start: state.scan_position,
            });
            state.rewind(nodes, start);
        }
        _ => {
            state.warnings.push(::Warning {
                end: state.scan_position + 2,
                message: ::WarningMessage::UnexpectedEndTag,
                start: state.scan_position,
            });
            state.scan_position += 2;
        }
    }
}

pub fn parse_template_separator(state: &mut ::State) {
    match state.stack.last_mut() {
        Some(::OpenNode {
            type_: ::OpenNodeType::Template { name, parameters },
            ..
        }) => {
            let position = ::state::skip_whitespace_backwards(state.wiki_text, state.scan_position);
            ::state::flush(
                &mut state.nodes,
                state.flushed_position,
                position,
                state.wiki_text,
            );
            state.flushed_position =
                ::state::skip_whitespace_forwards(state.wiki_text, state.scan_position + 1);
            state.scan_position = state.flushed_position;
            if name.is_none() {
                *name = Some(::std::mem::replace(&mut state.nodes, vec![]));
            } else {
                let parameters_length = parameters.len();
                let parameter = &mut parameters[parameters_length - 1];
                parameter.end = position;
                parameter.value = ::std::mem::replace(&mut state.nodes, vec![]);
            }
            parameters.push(::Parameter {
                end: 0,
                name: None,
                start: state.scan_position,
                value: vec![],
            });
        }
        _ => unreachable!(),
    }
}

pub fn parse_template_start(state: &mut ::State) {
    let scan_position = state.scan_position;
    if state.get_byte(state.scan_position + 2) == Some(b'{') {
        let position = state.skip_whitespace_forwards(scan_position + 3);
        state.push_open_node(
            ::OpenNodeType::Parameter {
                default: None,
                name: None,
            },
            position,
        );
    } else {
        let position = state.skip_whitespace_forwards(scan_position + 2);
        state.push_open_node(
            ::OpenNodeType::Template {
                name: None,
                parameters: vec![],
            },
            position,
        );
    }
}
