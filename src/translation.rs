use crate::{Modifier, RawTextComponent};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::translation))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct TranslatedContent<'a> {
    #[cfg_attr(feature = "serde", serde(rename = "translate"))]
    pub key: Cow<'a, str>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub fallback: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", rename = "with", default)
    )]
    pub args: Option<Box<[RawTextComponent<'a>]>>,
}
impl<'a> TranslatedContent<'a> {
    /// Creates a new `TranslatedContent` without fallback.
    /// ### Warning
    /// Using this method directly is discouraged.
    /// Please use a compiled [Translation] instead.
    pub const fn new(key: &'a str, args: Option<Box<[RawTextComponent<'a>]>>) -> Self {
        Self {
            key: Cow::Borrowed(key),
            args,
            fallback: None,
        }
    }

    #[inline]
    pub fn component(self) -> RawTextComponent<'a> {
        RawTextComponent::translated(self)
    }
    #[inline]
    pub fn component_fallback(mut self, fallback: impl Into<Cow<'a, str>>) -> RawTextComponent<'a> {
        self.fallback = Some(fallback.into());
        RawTextComponent::translated(self)
    }
}

impl<'a> From<TranslatedContent<'a>> for RawTextComponent<'a> {
    fn from(value: TranslatedContent<'a>) -> Self {
        value.component()
    }
}

pub struct Translation<'a, const ARGS: usize>(pub &'a str);

impl<'a> Translation<'a, 0> {
    /// Creates a new `TranslatedContent` with no arguments.
    #[must_use]
    pub const fn msg(&self) -> TranslatedContent<'_> {
        TranslatedContent::new(self.0, None)
    }
}

impl<'a, const ARGS: usize> Translation<'a, ARGS> {
    /// Creates a new `TranslatedContent` with the given arguments.
    #[must_use]
    pub fn message(&self, args: [impl Into<RawTextComponent<'a>>; ARGS]) -> TranslatedContent<'_> {
        TranslatedContent::new(self.0, Some(Box::new(args.map(Into::into))))
    }
}

impl<'a> From<&'a Translation<'a, 0>> for RawTextComponent<'a> {
    fn from(value: &'a Translation<'a, 0>) -> Self {
        value.msg().component()
    }
}

/// Minimal part of a full translation
///
/// It has 2 possible values
/// Text -> Raw text parts of the translation
/// Arg -> 1 based index of the translation arguments
pub enum TranslationToken<'a> {
    Text(Cow<'a, str>),
    Arg(usize),
}
impl<'a> TranslationToken<'a> {
    pub fn component(
        &'a self,
        values: &Option<Box<[RawTextComponent<'a>]>>,
    ) -> RawTextComponent<'a> {
        match self {
            Self::Text(text) => RawTextComponent::const_plain(text),
            Self::Arg(idx) => values
                .as_ref()
                .map(|component| component[*idx - 1].clone())
                .unwrap_or_default(),
        }
    }
}

pub trait TranslationTokenArray<'a> {
    fn component(&'a self, values: &Option<Box<[RawTextComponent<'a>]>>) -> RawTextComponent<'a>;
}
impl<'a> TranslationTokenArray<'a> for [TranslationToken<'a>] {
    fn component(&'a self, values: &Option<Box<[RawTextComponent<'a>]>>) -> RawTextComponent<'a> {
        let mut tokens = self.iter();
        let mut component = tokens
            .next()
            .map_or_else(RawTextComponent::new, |c| c.component(values));
        for token in tokens {
            component = component.add_child(token.component(values));
        }
        component
    }
}
