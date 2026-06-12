use uuid::Uuid;

use crate::RawTextComponent;
#[cfg(feature = "custom")]
use crate::custom::CustomData;
use std::borrow::Cow;

/// Represents interactive elements of a text component, such as click and hover events,
/// and text insertion on shift-click.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::interactivity))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Interactivity<'a> {
    /// The text to insert into the chat when the player shift-clicks this component.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub insertion: Option<Cow<'a, str>>,

    /// The action performed when the player clicks on this component.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            rename = "click_event",
            default
        )
    )]
    pub click: Option<ClickEvent<'a>>,

    /// The action performed when the player hovers over this component.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            rename = "hover_event",
            default
        )
    )]
    pub hover: Option<HoverEvent<'a>>,
}

impl<'a> Default for Interactivity<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Interactivity<'a> {
    pub const fn new() -> Self {
        Self {
            insertion: None,
            click: None,
            hover: None,
        }
    }

    pub fn is_none(&self) -> bool {
        self.insertion.is_none() && self.click.is_none() && self.hover.is_none()
    }

    pub fn mix(&self, other: &mut Self) {
        if self.insertion.is_some() {
            other.insertion = self.insertion.clone()
        }
        if self.click.is_some() {
            other.click = self.click.clone()
        }
        if self.hover.is_some() {
            other.hover = self.hover.clone()
        }
    }
}

/// Defines an action that occurs when a text component is clicked.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::interactivity))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "action", rename_all = "snake_case"))]
pub enum ClickEvent<'a> {
    /// Opens the given URL.
    OpenUrl {
        /// The URL to open.
        url: Cow<'a, str>,
    },
    /// Runs a command as the player.
    RunCommand {
        /// The command to execute.
        command: Cow<'a, str>,
    },
    /// Replaces the player's chat input with a command (without sending it).
    SuggestCommand {
        /// The command to suggest.
        command: Cow<'a, str>,
    },
    /// Changes the current page in a book.
    ChangePage {
        /// The page number to switch to (1-based, signed for protocol compatibility).
        page: i32,
    },
    /// Copies the given value to the clipboard.
    CopyToClipboard {
        /// The string to copy.
        value: Cow<'a, str>,
    },
    /// Shows a custom dialog.
    ShowDialog {
        /// Either a dialog resource identifier or a full dialog definition.
        dialog: Cow<'a, str>,
    },
    /// A custom click event, available only with the `custom` feature.
    #[cfg(feature = "custom")]
    Custom(CustomData<'a>),
}

impl<'a> ClickEvent<'a> {
    /// Creates a [ClickEvent] that opens a url when triggered.
    pub fn open_url(url: impl Into<Cow<'a, str>>) -> Self {
        ClickEvent::OpenUrl { url: url.into() }
    }

    /// Creates a [ClickEvent] that runs a command when triggered.
    pub fn run_command(command: impl Into<Cow<'a, str>>) -> Self {
        ClickEvent::RunCommand {
            command: command.into(),
        }
    }

    /// Creates a [ClickEvent] that replaces the chat input with a command when triggered.
    pub fn suggest_command(command: impl Into<Cow<'a, str>>) -> Self {
        ClickEvent::SuggestCommand {
            command: command.into(),
        }
    }

    /// Creates a [ClickEvent] that changes the page of a book when triggered.
    pub fn change_page(page: u32) -> Self {
        ClickEvent::ChangePage { page: page as i32 }
    }

    /// Creates a [ClickEvent] that copies it's content to the clipboard when triggered.
    pub fn copy_to_clipboard(value: impl Into<Cow<'a, str>>) -> Self {
        ClickEvent::CopyToClipboard {
            value: value.into(),
        }
    }

    /// Creates a [ClickEvent] that shows a custom dialog when triggered.
    /// * `dialog` - Either a dialog id or a dialog definition
    pub fn show_dialog(dialog: impl Into<Cow<'a, str>>) -> Self {
        ClickEvent::ShowDialog {
            dialog: dialog.into(),
        }
    }
}

/// Defines an action that occurs when hovering over a text component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components::interactivity))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "action", rename_all = "snake_case"))]
pub enum HoverEvent<'a> {
    /// Displays a text component as a tooltip.
    ShowText {
        /// The text component to display.
        value: Box<RawTextComponent<'a>>,
    },
    /// Displays an item tooltip.
    ShowItem {
        /// The item identifier (e.g., `minecraft:stone`).
        id: Cow<'a, str>,
        /// Optional stack size to display.
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        count: Option<i32>,
        /// Optional stringified item components (JSON).
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        components: Option<Cow<'a, str>>,
    },
    /// Displays an entity tooltip.
    ShowEntity {
        /// An optional custom name component to show instead of the entity's actual name.
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        name: Option<Box<RawTextComponent<'a>>>,
        /// The entity type identifier (e.g., `minecraft:creeper`).
        id: Cow<'a, str>,
        /// The UUID of the entity.
        #[cfg_attr(feature = "ownable", ownable(clone))]
        uuid: Uuid,
    },
}

impl<'a> HoverEvent<'a> {
    /// Creates a [HoverEvent] that will show a text component.
    pub fn show_text(text: impl Into<RawTextComponent<'a>>) -> Self {
        HoverEvent::ShowText {
            value: Box::new(text.into()),
        }
    }

    /// Creates a [HoverEvent] that will show an item.
    /// * `id` - The id of the item
    /// * `count` - If [Some] shows the amount of items
    /// * `components` - An optional stringified version of the item's components
    pub fn show_item(
        id: impl Into<Cow<'a, str>>,
        count: Option<i32>,
        components: Option<impl Into<Cow<'a, str>>>,
    ) -> Self {
        HoverEvent::ShowItem {
            id: id.into(),
            count,
            components: components.map(Into::into),
        }
    }

    /// Creates a [HoverEvent] that will show an entity.
    /// * `id` - The id of the entity's type
    /// * `uuid` - The id of the targeted entity
    /// * `name` - If [Some] the name to display
    pub fn show_entity(
        id: impl Into<Cow<'a, str>>,
        uuid: Uuid,
        name: Option<impl Into<RawTextComponent<'a>>>,
    ) -> Self {
        HoverEvent::ShowEntity {
            name: name.map(|r| Box::new(r.into())),
            id: id.into(),
            uuid,
        }
    }
}
