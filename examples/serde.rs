use text_components::{Modifier, RawTextComponent, format::Color, translation::TranslatedContent};

fn main() {
    let component: RawTextComponent = TranslatedContent::new("key", None)
        .color(Color::Blue)
        .bold(true);
    println!("{}", serde_json::to_string_pretty(&component).unwrap());
    let component: RawTextComponent = serde_json::from_str(
        "{
            \"text\": \"This is a Serde test\",
            \"color\": \"blue\",
            \"bold\": true
        }",
    )
    .unwrap();
    println!("{}", component.log())
}
