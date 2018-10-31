// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_external_link_end<'a>(
    state: &mut ::State<'a>,
    start_position: usize,
    nodes: Vec<::Node<'a>>,
) {
    let scan_position = state.scan_position;
    state.flush(scan_position);
    state.scan_position += 1;
    state.flushed_position = state.scan_position;
    let nodes = ::std::mem::replace(&mut state.nodes, nodes);
    state.nodes.push(::Node::ExternalLink {
        end: state.scan_position,
        nodes,
        start: start_position,
    });
}

pub fn parse_external_link_end_of_line(state: &mut ::State) {
    let end = state.scan_position;
    let open_node = state.stack.pop().unwrap();
    state.warnings.push(::Warning {
        end,
        message: ::WarningMessage::InvalidLinkSyntax,
        start: open_node.start,
    });
    state.rewind(open_node.nodes, open_node.start);
}

pub fn parse_external_link_start(state: &mut ::State, configuration: &::Configuration) {
    let scheme_start_position = state.scan_position + 1;
    match configuration
        .protocols
        .find(&state.wiki_text[scheme_start_position..])
    {
        Err(_) => {
            state.scan_position = scheme_start_position;
            return;
        }
        Ok(_) => {
            state.push_open_node(::OpenNodeType::ExternalLink, scheme_start_position);
        }
    }
}
