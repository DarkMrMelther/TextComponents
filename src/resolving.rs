use std::sync::{Arc, OnceLock};

#[cfg(feature = "custom")]
use crate::custom::CustomData;
use crate::{
    RawTextComponent,
    content::{Content, Resolvable},
    translation::TranslationToken,
};

pub(crate) static RESOLUTION_HELPER: OnceLock<Arc<dyn for<'a> ResolutionHelper<'a> + Send + Sync>> =
    OnceLock::new();

pub fn set_resolution_helper(resolutor: Arc<dyn for<'a> ResolutionHelper<'a> + Send + Sync>) {
    RESOLUTION_HELPER.get_or_init(|| resolutor);
}

/// Trait to provide global data for resolving content within a text component.
pub trait ResolutionHelper<'a> {
    /// Resolves a custom data block into an optional `RawTextComponent`.
    ///
    /// Only available when the `custom` feature is enabled. Return `None` if
    /// the custom ID is not recognized or cannot be resolved.
    #[cfg(feature = "custom")]
    fn resolve_custom(
        &self,
        resolutor: &dyn TextResolutor<'a>,
        data: &CustomData<'a>,
    ) -> Option<RawTextComponent<'a>>;

    /// Translates a given translation key into pre-compiled [TranslationToken]s.
    ///
    /// Returns `None` if the key is unknown.
    fn translate(&self, locale: &str, key: &str) -> Option<&[TranslationToken<'a>]>;
}

/// Trait for resolving dynamic content within a text component.
///
/// This trait provides the necessary hooks to replace placeholders (like scoreboards,
/// entity selectors, NBT paths, translations, and custom data) with actual
/// `RawTextComponent` trees. It is intended to be implemented on game-world objects
/// such as a player or the server world, where the resolution logic can access
/// live data.
///
/// # Recommendation
/// Implement this on the `World`, `Player`, and `Entity` types / traits in your application.
pub trait TextResolutor<'a> {
    /// Fallback resolution for any `Content` variant that is not explicitly handled.
    ///
    /// By default it clones the content as-is, converting it into a plain component.
    /// Override this method if you need special handling for other content types.
    fn resolve_other(&self, content: &Content<'a>) -> RawTextComponent<'a> {
        RawTextComponent::from(content.clone())
    }

    /// Resolves a `Resolvable` variant into a `RawTextComponent`.
    ///
    /// This method is called for scoreboards, entity selectors, and NBT paths.
    /// The implementation should query the game state and return a component
    /// representing the resolved value (e.g., a score value, an entity name,
    /// or a formatted NBT tag).
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a>;

    /// Returns the current locale of the resolver
    fn locale(&self) -> &str {
        "en"
    }

    /// Returns the ID of the resolving entity, if available.
    ///
    /// Useful to get data from the receiver in custom contents.
    fn entity_id(&self) -> Option<i32> {
        None
    }
}

impl<'a, T: TextResolutor<'a>> TextResolutor<'a> for Arc<T> {
    fn resolve_other(&self, content: &Content<'a>) -> RawTextComponent<'a> {
        (**self).resolve_other(content)
    }

    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        (**self).resolve_content(resolvable)
    }
}

impl<'a, T: TextResolutor<'a> + ?Sized> TextResolutor<'a> for &T {
    fn resolve_other(&self, content: &Content<'a>) -> RawTextComponent<'a> {
        (**self).resolve_other(content)
    }

    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        (**self).resolve_content(resolvable)
    }
}

/// A `TextResolutor` that does no actual resolution.
///
/// It returns stub text for scoreboards, entity selectors, NBT paths, and custom
/// data, and never translates any key. Useful for testing or when only the
/// static structure of a component is needed.
pub struct NoResolutor;

impl<'a> TextResolutor<'a> for NoResolutor {
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        match resolvable {
            Resolvable::Scoreboard { objective, .. } => {
                RawTextComponent::plain(format!("[Score: {objective}]"))
            }
            Resolvable::Entity { selector, .. } => {
                RawTextComponent::plain(format!("[Entity: {selector}]"))
            }
            Resolvable::NBT { path, .. } => RawTextComponent::plain(format!("[Nbt: {path}]")),
        }
    }
}

impl<'a> RawTextComponent<'a> {
    /// Builds the component into a target format after full resolution.
    ///
    /// This method first resolves all dynamic content using the given `resolutor`,
    /// then uses the `target` builder to convert the resolved component tree into
    /// the desired output type (e.g., plain `String`, coloured terminal output,
    /// NBT, JSON, etc.).
    ///
    /// # Example
    /// ```
    /// # use text_components::RawTextComponent;
    /// # use text_components::resolving::{TextResolutor, BuildTarget, NoResolutor};
    /// # struct MyTarget;
    /// # impl<'a> BuildTarget<'a> for MyTarget {
    /// #     type Result = String;
    /// #     fn build_component<R: TextResolutor<'a> + ?Sized>(&self, _: &R, c: &RawTextComponent<'a>) -> String {
    /// #         "built".to_string()
    /// #     }
    /// # }
    /// let component = RawTextComponent::plain("Hello");
    /// let result = component.build(&NoResolutor, MyTarget);
    /// ```
    pub fn build<R: TextResolutor<'a> + ?Sized, S: BuildTarget<'a>>(
        &self,
        resolutor: &R,
        target: S,
    ) -> S::Result {
        target.build_component(resolutor, &self.resolve(resolutor))
    }
    pub fn batch_build<'b, R: TextResolutor<'a> + ?Sized, S: BuildTarget<'a>>(
        &self,
        resolutors: Vec<&'b R>,
        target: fn() -> S,
    ) -> Vec<(&'b R, S::Result)> {
        resolutors
            .into_iter()
            .map(|resolutor| {
                (
                    resolutor,
                    target().build_component(resolutor, &self.resolve(resolutor)),
                )
            })
            .collect()
    }

    /// Resolves all dynamic parts of the component recursively.
    ///
    /// This replaces `Resolvable` and `Custom` content with actual components
    /// obtained from the `resolutor`, and also resolves arguments inside
    /// `TranslatedContent`, separators for entity/NBT resolvables, and children.
    /// Formatting and interactivity are merged appropriately.
    ///
    /// The returned component is fully static (no more `Resolvable` leaves)
    /// and can be serialized or built without further resolution.
    pub fn resolve<R: TextResolutor<'a> + ?Sized>(&self, resolutor: &R) -> RawTextComponent<'a> {
        let mut component = match &self.content {
            #[cfg(feature = "custom")]
            Content::Custom(data) => RESOLUTION_HELPER
                .get()
                .and_then(|h| h.resolve_custom(&resolutor, data))
                .unwrap_or_default(),
            Content::Resolvable(resolvable) => resolutor.resolve_content(resolvable),
            content => resolutor.resolve_other(content),
        };

        match &mut component.content {
            Content::Translate(message) => {
                message.args = message.args.as_ref().map(|args| {
                    args.iter()
                        .map(|arg| arg.resolve(resolutor))
                        .collect::<Vec<RawTextComponent>>()
                        .into_boxed_slice()
                });
            }
            Content::Resolvable(Resolvable::Entity { separator, .. }) => {
                **separator = separator.resolve(resolutor);
            }
            Content::Resolvable(Resolvable::NBT { separator, .. }) => {
                **separator = separator.resolve(resolutor);
            }
            _ => (),
        }

        component.children.append(
            &mut self
                .children
                .iter()
                .map(|child| child.resolve(resolutor))
                .collect(),
        );
        self.interactions.mix(&mut component.interactions);
        component.format = self.format.mix(&component.format);

        component
    }
    pub fn batch_resolve<'b, R: TextResolutor<'a> + ?Sized>(
        &self,
        resolutors: Vec<&'b R>,
    ) -> Vec<(&'b R, RawTextComponent<'a>)> {
        resolutors
            .into_iter()
            .map(|resolutor| (resolutor, self.resolve(resolutor)))
            .collect()
    }
}

/// A target format for building a resolved text component.
///
/// Implement this trait to convert a resolved `RawTextComponent` tree into
/// a concrete representation, such as a plain `String`, an ANSI‑coloured string,
/// an NBT tag, or a JSON value.
///
/// # Type Parameter
/// * `Result` – The type produced by the builder (e.g., `String`, `NbtTag`).
pub trait BuildTarget<'a> {
    /// The type produced by this builder.
    type Result;

    /// Converts a single resolved component into the target representation.
    ///
    /// The method is called recursively for the whole component tree.
    fn build_component<R: TextResolutor<'a> + ?Sized>(
        &self,
        resolutor: &R,
        component: &RawTextComponent<'a>,
    ) -> Self::Result
    where
        Self: Sized;
}

impl<'a, T: BuildTarget<'a>> BuildTarget<'a> for Box<T> {
    type Result = T::Result;

    fn build_component<R: TextResolutor<'a> + ?Sized>(
        &self,
        resolutor: &R,
        component: &RawTextComponent<'a>,
    ) -> Self::Result
    where
        Self: Sized,
    {
        (**self).build_component(resolutor, component)
    }
}
