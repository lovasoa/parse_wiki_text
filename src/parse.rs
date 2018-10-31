// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

#[must_use]
pub fn parse<'a>(configuration: &::Configuration, wiki_text: &'a str) -> ::Output<'a> {
    let mut state = ::State {
        flushed_position: 0,
        nodes: vec![],
        scan_position: 0,
        stack: vec![],
        warnings: vec![],
        wiki_text,
    };
    {
        let mut has_line_break = false;
        let mut position = 0;
        loop {
            match state.get_byte(position) {
                Some(b'\n') => {
                    if has_line_break {
                        state.warnings.push(::Warning {
                            end: position + 1,
                            message: ::WarningMessage::RepeatedEmptyLine,
                            start: position,
                        });
                    }
                    has_line_break = true;
                    position += 1;
                    state.flushed_position = position;
                    state.scan_position = position;
                }
                Some(b' ') => position += 1,
                Some(b'#') => {
                    ::redirect::parse_redirect(&mut state, configuration, position);
                    break;
                }
                _ => break,
            }
        }
    }
    ::line::parse_beginning_of_line(&mut state, None);
    loop {
        match state.get_byte(state.scan_position) {
            None => {
                ::line::parse_end_of_line(&mut state);
                if state.scan_position < state.wiki_text.len() {
                    continue;
                }
                if let Some(::OpenNode { nodes, start, .. }) = state.stack.pop() {
                    state.warnings.push(::Warning {
                        end: state.scan_position,
                        message: ::WarningMessage::MissingEndTagRewinding,
                        start,
                    });
                    state.rewind(nodes, start);
                } else {
                    break;
                }
            }
            Some(0) | Some(1) | Some(2) | Some(3) | Some(4) | Some(5) | Some(6) | Some(7)
            | Some(8) | Some(11) | Some(12) | Some(13) | Some(14) | Some(15) | Some(16)
            | Some(17) | Some(18) | Some(19) | Some(20) | Some(21) | Some(22) | Some(23)
            | Some(24) | Some(25) | Some(26) | Some(27) | Some(28) | Some(29) | Some(30)
            | Some(31) | Some(127) => {
                state.warnings.push(::Warning {
                    end: state.scan_position + 1,
                    message: ::WarningMessage::InvalidCharacter,
                    start: state.scan_position,
                });
                state.scan_position += 1;
            }
            Some(b'\n') => {
                ::line::parse_end_of_line(&mut state);
            }
            Some(b'!') if state.get_byte(state.scan_position + 1) == Some(b'!') => {
                ::table::parse_heading_cell(&mut state);
            }
            Some(b'&') => ::character_entity::parse_character_entity(&mut state, configuration),
            Some(b'\'') => if state.get_byte(state.scan_position + 1) == Some(b'\'') {
                ::bold_italic::parse_bold_italic(&mut state);
            } else {
                state.scan_position += 1;
            },
            Some(b'<') => match state.get_byte(state.scan_position + 1) {
                Some(b'!')
                    if state.get_byte(state.scan_position + 2) == Some(b'-')
                        && state.get_byte(state.scan_position + 3) == Some(b'-') =>
                {
                    ::comment::parse_comment(&mut state)
                }
                Some(b'/') => ::tag::parse_end_tag(&mut state, configuration),
                _ => ::tag::parse_start_tag(&mut state, configuration),
            },
            Some(b'=') => {
                ::template::parse_parameter_name_end(&mut state);
            }
            Some(b'[') => if state.get_byte(state.scan_position + 1) == Some(b'[') {
                ::link::parse_link_start(&mut state, configuration);
            } else {
                ::external_link::parse_external_link_start(&mut state, configuration);
            },
            Some(b']') => match state.stack.pop() {
                None => state.scan_position += 1,
                Some(::OpenNode {
                    nodes,
                    start,
                    type_: ::OpenNodeType::ExternalLink,
                }) => {
                    ::external_link::parse_external_link_end(&mut state, start, nodes);
                }
                Some(::OpenNode {
                    nodes,
                    start,
                    type_: ::OpenNodeType::Link { namespace, target },
                }) => if state.get_byte(state.scan_position + 1) == Some(b']') {
                    ::link::parse_link_end(
                        &mut state,
                        &configuration,
                        start,
                        nodes,
                        namespace,
                        target,
                    );
                } else {
                    state.scan_position += 1;
                    state.stack.push(::OpenNode {
                        nodes,
                        start,
                        type_: ::OpenNodeType::Link { namespace, target },
                    });
                },
                Some(open_node) => {
                    state.scan_position += 1;
                    state.stack.push(open_node);
                }
            },
            Some(b'_') => if state.get_byte(state.scan_position + 1) == Some(b'_') {
                ::magic_word::parse_magic_word(&mut state, configuration);
            } else {
                state.scan_position += 1;
            },
            Some(b'{') => if state.get_byte(state.scan_position + 1) == Some(b'{') {
                ::template::parse_template_start(&mut state);
            } else {
                state.scan_position += 1;
            },
            Some(b'|') => match state.stack.last_mut() {
                Some(::OpenNode {
                    type_: ::OpenNodeType::Parameter { default: None, .. },
                    ..
                }) => {
                    ::template::parse_parameter_separator(&mut state);
                }
                Some(::OpenNode {
                    type_: ::OpenNodeType::Table(..),
                    ..
                }) => {
                    ::table::parse_inline_token(&mut state);
                }
                Some(::OpenNode {
                    type_: ::OpenNodeType::Template { .. },
                    ..
                }) => {
                    ::template::parse_template_separator(&mut state);
                }
                _ => state.scan_position += 1,
            },
            Some(b'}') => if state.get_byte(state.scan_position + 1) == Some(b'}') {
                ::template::parse_template_end(&mut state);
            } else {
                state.scan_position += 1;
            },
            _ => {
                state.scan_position += 1;
            }
        }
    }
    let end_position = state.skip_whitespace_backwards(wiki_text.len());
    state.flush(end_position);
    ::Output {
        nodes: state.nodes,
        warnings: state.warnings,
    }
}
