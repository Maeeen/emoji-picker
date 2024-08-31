use std::path::PathBuf;

// These one may be required when implementing a better search.
// pub type EmojiMap<Internal = EmojiWrapper> = BTreeMap<String, Internal>;
// pub type EmojiGrouppedMap<Internal> = HashMap<emojis::Group, EmojiMap<Internal>>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmojiWrapper(pub &'static emojis::Emoji);

impl EmojiWrapper {
    pub fn name(&self) -> &str {
        self.0.name()
    }

    pub fn code(&self) -> &str {
        self.0.as_str()
    }

    pub fn skin_tones(&self) -> Option<impl Iterator<Item = Self> + Clone> {
        self.0.skin_tones().map(|x| x.map(EmojiWrapper))
    }

    pub fn skin_tone(&self) -> Option<u16> {
        self.0.skin_tone().and_then(EmojiWrapper::skin_tone_idx)
    }

    fn skin_tone_idx(st: emojis::SkinTone) -> Option<u16> {
        match st {
            emojis::SkinTone::Default => Some(0),
            emojis::SkinTone::Light => Some(1),
            emojis::SkinTone::MediumLight => Some(2),
            emojis::SkinTone::Medium => Some(3),
            emojis::SkinTone::MediumDark => Some(4),
            emojis::SkinTone::Dark => Some(5),
            emojis::SkinTone::LightAndMediumLight => Some(6),
            emojis::SkinTone::LightAndMedium => Some(7),
            emojis::SkinTone::LightAndMediumDark => Some(8),
            emojis::SkinTone::LightAndDark => Some(9),
            emojis::SkinTone::MediumLightAndLight => Some(10),
            emojis::SkinTone::MediumLightAndMedium => Some(11),
            emojis::SkinTone::MediumLightAndMediumDark => Some(12),
            emojis::SkinTone::MediumLightAndDark => Some(13),
            emojis::SkinTone::MediumAndLight => Some(14),
            emojis::SkinTone::MediumAndMediumLight => Some(15),
            emojis::SkinTone::MediumAndMediumDark => Some(16),
            emojis::SkinTone::MediumAndDark => Some(17),
            emojis::SkinTone::MediumDarkAndLight => Some(18),
            emojis::SkinTone::MediumDarkAndMediumLight => Some(19),
            emojis::SkinTone::MediumDarkAndMedium => Some(20),
            emojis::SkinTone::MediumDarkAndDark => Some(21),
            emojis::SkinTone::DarkAndLight => Some(22),
            emojis::SkinTone::DarkAndMediumLight => Some(23),
            emojis::SkinTone::DarkAndMedium => Some(24),
            emojis::SkinTone::DarkAndMediumDark => Some(25),
            _ => None
        }
    }
}


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
