use crate::{RawTextComponent, resolving::TextResolutor};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::custom))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct CustomData<'a> {
    pub id: Cow<'a, str>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Payload::is_empty", default)
    )]
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::custom))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Payload {
    #[default]
    Empty,
    // More payload data
}
impl Payload {
    pub fn is_empty(&self) -> bool {
        self == &Payload::Empty
    }
}

pub trait CustomContentExt<'a> {
    fn as_data(&self) -> CustomData<'a>;
}

pub trait CustomContent<'a, Ctx>: CustomContentExt<'a> {
    fn resolve(
        &self,
        resolutor: &dyn TextResolutor<'a>,
        context: Ctx,
        payload: Payload,
    ) -> RawTextComponent<'a>;
}

impl<'a> From<CustomData<'a>> for RawTextComponent<'a> {
    fn from(value: CustomData<'a>) -> Self {
        RawTextComponent {
            content: crate::content::Content::Custom(value),
            ..Default::default()
        }
    }
}
impl<'a, T: CustomContentExt<'a>> From<T> for RawTextComponent<'a> {
    fn from(value: T) -> Self {
        RawTextComponent {
            content: crate::content::Content::Custom(value.as_data()),
            ..Default::default()
        }
    }
}
