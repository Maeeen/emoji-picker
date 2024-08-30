use std::path::PathBuf;

// These one may be required when implementing a better search.
// pub type EmojiMap<Internal = EmojiWrapper> = BTreeMap<String, Internal>;
// pub type EmojiGrouppedMap<Internal> = HashMap<emojis::Group, EmojiMap<Internal>>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmojiWrapper(pub &'static emojis::Emoji);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmojiGroupWrapper(pub emojis::Group);

impl EmojiGroupWrapper {
    pub fn group_name(&self) -> &'static str {
        (*self).into()
    }
}

impl From<EmojiGroupWrapper> for &'static str {
    fn from(group: EmojiGroupWrapper) -> &'static str {
        match group.0 {
            emojis::Group::Activities => "Activities",
            emojis::Group::AnimalsAndNature => "Animals & Nature",
            emojis::Group::Flags => "Flags",
            emojis::Group::FoodAndDrink => "Food & Drink",
            emojis::Group::Objects => "Objects",
            emojis::Group::PeopleAndBody => "People & Body",
            emojis::Group::SmileysAndEmotion => "Smileys & Emotion",
            emojis::Group::Symbols => "Symbols",
            emojis::Group::TravelAndPlaces => "Travel & Places",
        }
    }
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
        let codes: Vec<String> = codes.into_iter().map(|e| format!("{:0>4x}", e)).collect();
        codes.join("-")
    }

    fn get_filename_path(&self) -> PathBuf {
        let filename = self.get_filename();
        PathBuf::from(format!("./emojis/twemoji/assets/svg/{filename}.svg"))
    }
}
