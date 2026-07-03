use std::{borrow::Cow, collections::HashMap, sync::Arc};

#[cfg(feature = "custom")]
use chrono::Local;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "nbt")]
use simdnbt::{
    ToNbtTag,
    owned::{BaseNbt, Nbt, NbtCompound, NbtTag},
};
#[cfg(feature = "custom")]
use text_components::custom::{CustomContent, CustomContentExt, CustomData, Payload};
#[cfg(feature = "nbt")]
use text_components::nbt::{NbtBuilder, ToSNBT};
use text_components::{
    Modifier, RawTextComponent,
    content::{NbtSource, ObjectPlayer, Resolvable},
    fmt::set_display_resolutor,
    format::Color,
    interactivity::{ClickEvent, HoverEvent},
    resolving::{ResolutionHelper, TextResolutor, set_resolution_helper},
    translation::{TranslatedContent, Translation, TranslationToken},
};
use uuid::Uuid;

struct GlobalHelper {
    pub translations: HashMap<&'static str, &'static [TranslationToken<'static>]>,
}
impl GlobalHelper {
    fn new() -> Self {
        let mut this = Self {
            translations: HashMap::new(),
        };
        this.register(
            "en",
            "content",
            &[
                TranslationToken::Text(Cow::Borrowed("This is a test TextComponent!\n Color: ")),
                TranslationToken::Arg(1),
                TranslationToken::Text(Cow::Borrowed("\n Bold: ")),
                TranslationToken::Arg(2),
                TranslationToken::Text(Cow::Borrowed("\n Italic: ")),
                TranslationToken::Arg(3),
                TranslationToken::Text(Cow::Borrowed("\n Underline: ")),
                TranslationToken::Arg(4),
                TranslationToken::Text(Cow::Borrowed("\n Strikethrough: ")),
                TranslationToken::Arg(5),
                TranslationToken::Text(Cow::Borrowed("\n Obfuscated: ")),
                TranslationToken::Arg(6),
                TranslationToken::Text(Cow::Borrowed("\n Shadow Color: ")),
                TranslationToken::Arg(7),
                TranslationToken::Text(Cow::Borrowed("\n Translation: ")),
                TranslationToken::Arg(8),
                TranslationToken::Text(Cow::Borrowed("\n Link: ")),
                TranslationToken::Arg(9),
                TranslationToken::Text(Cow::Borrowed(
                    "\n(All the green text is translated with arguments checked at compile time!)",
                )),
            ],
        );
        this.register(
            "en",
            "translated",
            &[TranslationToken::Text(Cow::Borrowed(
                "This text is Translated! (Without compile time check!)",
            ))],
        );
        this.register(
            "en",
            "resoluble",
            &[
                TranslationToken::Text(Cow::Borrowed("\n\nResolubles:\n Object: ")),
                TranslationToken::Arg(1),
                TranslationToken::Text(Cow::Borrowed("\n Scoreboard: ")),
                TranslationToken::Arg(2),
                TranslationToken::Text(Cow::Borrowed("\n Entity: ")),
                TranslationToken::Arg(3),
                TranslationToken::Text(Cow::Borrowed("\n Nbt: ")),
                TranslationToken::Arg(4),
            ],
        );
        this
    }

    fn register(
        &mut self,
        _locale: &'static str,
        key: &'static str,
        translation: &'static [TranslationToken<'static>],
    ) {
        self.translations.insert(key, translation);
    }
}
impl<'a> ResolutionHelper<'a> for GlobalHelper {
    #[cfg(feature = "custom")]
    fn resolve_custom(
        &self,
        resolutor: &dyn TextResolutor<'a>,
        data: &CustomData,
    ) -> Option<RawTextComponent<'a>> {
        if data.id == "time" {
            return Some(TimeContent.resolve(resolutor, (), Payload::Empty));
        }
        None
    }
    fn translate(&self, _locale: &str, key: &str) -> Option<&[TranslationToken<'a>]> {
        self.translations.get(key).copied()
    }
}

struct EmptyResolutor;
impl<'a> TextResolutor<'a> for EmptyResolutor {
    fn resolve_content(&self, resolvable: &Resolvable) -> RawTextComponent<'a> {
        match resolvable {
            Resolvable::Scoreboard { .. } => RawTextComponent::plain("5"),
            Resolvable::Entity { .. } => RawTextComponent::plain("MrMelther")
                .insertion("MrMelther")
                .click_event(ClickEvent::suggest_command("/msg MrMelther "))
                .hover_event(HoverEvent::show_entity(
                    "minecraft:player",
                    Uuid::max(),
                    Some("MrMelther"),
                )),
            #[cfg(feature = "nbt")]
            Resolvable::NBT { .. } => RawTextComponent::plain(
                Nbt::Some(BaseNbt::new(
                    "",
                    NbtCompound::from_values(vec![
                        ("base".into(), NbtTag::Double(3.)),
                        (
                            "id".into(),
                            "minecraft:entity_interaction_range".to_nbt_tag(),
                        ),
                    ]),
                ))
                .to_snbt(),
            ),
            #[cfg(not(feature = "nbt"))]
            Resolvable::NBT { .. } => {
                RawTextComponent::plain("{base:3.0d,id:\"minecraft:entity_interaction_range\"}")
            }
        }
    }
}

const CONTENT: Translation<9> = Translation("content");
const RESOLUBLE: Translation<4> = Translation("resoluble");

#[cfg(feature = "custom")]
struct TimeContent;
#[cfg(feature = "custom")]
impl<'a> CustomContentExt<'a> for TimeContent {
    fn as_data(&self) -> CustomData<'a> {
        CustomData {
            id: std::borrow::Cow::Borrowed("time"),
            payload: Payload::Empty,
        }
    }
}
#[cfg(feature = "custom")]
impl<'a> CustomContent<'a, ()> for TimeContent {
    fn resolve(
        &self,
        _resolutor: &dyn TextResolutor<'a>,
        _context: (),
        _payload: Payload,
    ) -> RawTextComponent<'a> {
        RawTextComponent::plain(Local::now().format("%H:%M").to_string())
    }
}
#[cfg(feature = "custom")]
impl<'a> CustomContent<'a, i8> for TimeContent {
    fn resolve(
        &self,
        _resolutor: &dyn TextResolutor<'a>,
        _context: i8,
        _payload: Payload,
    ) -> RawTextComponent<'a> {
        RawTextComponent::plain(Local::now().format("%H:%M").to_string())
    }
}

fn main() {
    set_display_resolutor(&EmptyResolutor);
    set_resolution_helper(Arc::new(GlobalHelper::new()));
    let resolubles = RESOLUBLE
        .message([
            ObjectPlayer::name("MrMelther").reset(),
            RawTextComponent::scoreboard("MrMelther", "objective").reset(),
            RawTextComponent::entity("@p", None).reset(),
            RawTextComponent::nbt("attributes[2]", NbtSource::entity("@p"), false, None).reset(),
        ])
        .color_hex("#6f00ff");

    let component = CONTENT
        .message([
            "This text is Blue!".reset().color(Color::Blue),
            "This text is Bold!".reset().bold(true),
            "This text is Italic!".reset().italic(true),
            "This text is Underlined!".reset().underlined(true),
            "This text is Strikethrough!".reset().strikethrough(true),
            "This text is Obfuscated!".reset().obfuscated(true),
            "This text is ShadowcoloRED!"
                .reset()
                .shadow_color(255, 128, 0, 0),
            TranslatedContent::new("translated", None).reset(),
            "This text contains a link!"
                .click_event(ClickEvent::open_url(
                    "https://github.com/Steel-Foundation/TextComponents",
                ))
                .reset(),
        ])
        .color(Color::Green)
        .bold(true);

    let component = cfg_select! {
        feature = "custom" => {
            component.add_child(resolubles.add_children(vec!["\n Custom: ".into(), TimeContent.reset()]))
        }
        _ => {
            component.add_child(resolubles)
        }
    };

    println!("\nDebug:\n{:?}", component);
    #[cfg(feature = "serde")]
    {
        let mut vec = vec![];
        let _ = component
            .resolve(&EmptyResolutor)
            .serialize(&mut serde_json::Serializer::new(&mut vec));
        println!("\nSerde (json):\n{}", String::from_utf8(vec).unwrap());
    }
    #[cfg(feature = "nbt")]
    println!(
        "\nNBT (SNBT):\ntellraw @a {}",
        component.build(&EmptyResolutor, NbtBuilder).to_snbt()
    );
    println!("\nText:\n{}", component.to_string());
    println!("\nPretty Text:\n{}", component.log());
}
