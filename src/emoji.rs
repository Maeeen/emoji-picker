use std::collections::BTreeMap;
use std::path::PathBuf;


pub type EmojiMap = BTreeMap<String, EmojiWrapper>;
pub struct EmojiWrapper(&'static emojis::Emoji);

pub fn list_emojis() -> EmojiMap {
    let mut emojis = BTreeMap::new();
    for emoji in emojis::iter() {
        emojis.insert(emoji.as_str().into(), EmojiWrapper(emoji));
    };
    emojis
}

pub trait TwemojiFilename {
    fn get_filename(&self) -> String;
    fn get_filename_path(&self) -> PathBuf;
}

impl EmojiWrapper {
    pub fn name(&self) -> &str {
        self.0.name()
    }

    pub fn code(&self) -> &str {
        self.0.as_str()
    }
}

impl TwemojiFilename for EmojiWrapper {
    /// Converts the emoji to a Twemoji filename as described in
    /// the Twemoji repository.
    fn get_filename(&self) -> String {
        const U200D: u32 = 0x200D;
        const UFE0F: u32 = 0xFE0F;

        let chars: Vec<u32> = self.0.to_string().chars().map(u32::from).collect();
        let codes: Vec<u32> = if !chars.contains(&U200D) {
            chars.into_iter().filter(|c| *c != UFE0F).collect()
        } else {
            chars
        };
        let codes: Vec<String> = codes
            .into_iter()
            .map(|e| format!("{:0>4x}", e))
            .collect();
        codes.join("-")
    }

    fn get_filename_path(&self) -> PathBuf {
        let filename = self.get_filename();
        PathBuf::from(format!("./emojis/twemoji/assets/svg/{filename}.svg"))
    }
}