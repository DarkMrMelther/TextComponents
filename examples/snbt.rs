use text_components::RawTextComponent;

#[cfg(feature = "custom")]
mod custom {
    use text_components::{
        RawTextComponent,
        custom::CustomData,
        resolving::{ResolutionHelper, TextResolutor},
        translation::TranslationToken,
    };

    pub struct GlobalHelper;
    impl<'a> ResolutionHelper<'a> for GlobalHelper {
        fn resolve_custom(
            &self,
            _resolutor: &dyn TextResolutor<'a>,
            data: &CustomData<'a>,
        ) -> Option<RawTextComponent<'a>> {
            match data.id.as_ref() {
                "hello" => Some(RawTextComponent::const_plain("Hello World!")),
                _ => None,
            }
        }

        fn translate(&self, _locale: &str, _key: &str) -> Option<&[TranslationToken<'a>]> {
            None
        }
    }
}

fn main() {
    use std::io::{Write, stdin, stdout};
    #[cfg(feature = "custom")]
    text_components::resolving::set_resolution_helper(std::sync::Arc::new(custom::GlobalHelper));

    let mut s = String::new();
    print!("/tellraw @s ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    let component = RawTextComponent::from_snbt(&s);
    match component {
        Ok(component) => {
            println!("{:?}", component);
            println!("{}", component.log())
        }
        Err(e) => eprintln!("{}", e),
    }
    // ["\"Howdy!\"", { text:"\nThis is a text component!\n", color:'blue', "bold":1b, italic:true }, {text:"Texto" , type: "translatable", fallback:"lol\n", translate:"lmao"}, {sprite:"items/iron_sword"}, "\n", {object:"player", player:{name:"MrMelther"}, hover_event:{action:"show_text",value:{text:"Send msg to MrMelther"}}, click_event:{action:"suggest_command", command:"/msg MrMelther "} }, {object:"player", player:{properties:[{name:"textures", value:"[Put your base64 texture here!]"}]}}]
    // {text:"Test",extra:[" lmao"]}
    // "Workflow test!"
    // {custom:{id:"hello"}, color:"green"}
}
