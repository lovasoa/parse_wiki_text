// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

macro_rules! impl_positioned {
    ($type:tt) => {
        impl<'a> ::Positioned for ::$type<'a> {
            fn end(&self) -> usize {
                self.end
            }

            fn start(&self) -> usize {
                self.start
            }
        }
    };
}

impl_positioned!(DefinitionListItem);
impl_positioned!(ListItem);
impl_positioned!(Parameter);
impl_positioned!(TableCaption);
impl_positioned!(TableCell);
impl_positioned!(TableRow);

impl<'a> ::Positioned for ::Node<'a> {
    fn end(&self) -> usize {
        match *self {
            ::Node::Bold { end, .. } => end,
            ::Node::BoldItalic { end, .. } => end,
            ::Node::Category { end, .. } => end,
            ::Node::CharacterEntity { end, .. } => end,
            ::Node::Comment { end, .. } => end,
            ::Node::DefinitionList { end, .. } => end,
            ::Node::EndTag { end, .. } => end,
            ::Node::ExternalLink { end, .. } => end,
            ::Node::Heading { end, .. } => end,
            ::Node::HorizontalDivider { end, .. } => end,
            ::Node::Image { end, .. } => end,
            ::Node::Italic { end, .. } => end,
            ::Node::Link { end, .. } => end,
            ::Node::MagicWord { end, .. } => end,
            ::Node::OrderedList { end, .. } => end,
            ::Node::ParagraphBreak { end, .. } => end,
            ::Node::Parameter { end, .. } => end,
            ::Node::Preformatted { end, .. } => end,
            ::Node::Redirect { end, .. } => end,
            ::Node::StartTag { end, .. } => end,
            ::Node::Table { end, .. } => end,
            ::Node::Tag { end, .. } => end,
            ::Node::Template { end, .. } => end,
            ::Node::Text { end, .. } => end,
            ::Node::UnorderedList { end, .. } => end,
        }
    }

    fn start(&self) -> usize {
        match *self {
            ::Node::Bold { start, .. } => start,
            ::Node::BoldItalic { start, .. } => start,
            ::Node::Category { start, .. } => start,
            ::Node::CharacterEntity { start, .. } => start,
            ::Node::Comment { start, .. } => start,
            ::Node::DefinitionList { start, .. } => start,
            ::Node::EndTag { start, .. } => start,
            ::Node::ExternalLink { start, .. } => start,
            ::Node::Heading { start, .. } => start,
            ::Node::HorizontalDivider { start, .. } => start,
            ::Node::Image { start, .. } => start,
            ::Node::Italic { start, .. } => start,
            ::Node::Link { start, .. } => start,
            ::Node::MagicWord { start, .. } => start,
            ::Node::OrderedList { start, .. } => start,
            ::Node::ParagraphBreak { start, .. } => start,
            ::Node::Parameter { start, .. } => start,
            ::Node::Preformatted { start, .. } => start,
            ::Node::Redirect { start, .. } => start,
            ::Node::StartTag { start, .. } => start,
            ::Node::Table { start, .. } => start,
            ::Node::Tag { start, .. } => start,
            ::Node::Template { start, .. } => start,
            ::Node::Text { start, .. } => start,
            ::Node::UnorderedList { start, .. } => start,
        }
    }
}
