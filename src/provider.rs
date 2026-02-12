use rust_embed::Embed;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self},
};

#[derive(Clone)]
pub struct Emoji {
    unicode: String,
    name: String,
}

impl fmt::Debug for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.unicode.as_str())
    }
}

#[derive(Default)]
pub struct EmojiCategory {
    name: String,
    emojis: Vec<Emoji>,
}

impl EmojiCategory {
    fn insert(&mut self, emoji: Emoji) {
        self.emojis.push(emoji)
    }

    pub fn len(&self) -> usize {
        self.emojis.len()
    }
}

impl fmt::Debug for EmojiCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Category: {} ({} entries)",
            self.name,
            self.emojis.len()
        ))
    }
}

#[derive(Debug)]
pub struct EmojiProvider {
    pub categories: HashMap<String, EmojiCategory>,
    pub categories_order: Vec<String>,
    emojis: Vec<Emoji>,
}

#[derive(Embed)]
#[folder = "assets/default-emojis"]
struct DefaultEmojis;

#[derive(serde::Deserialize, Debug)]
struct SerializedEmoji {
    emoji: String,
    name: String,
    category: String,
    subcategory: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Failed to read default emojis.")]
    DefaultsNotAvailable,
    #[error("The emoji file is invalid")]
    InvalidFile,
    #[error("Failed to parse the JSON emoji file.")]
    NotJSON,
}

impl EmojiProvider {
    pub fn new() -> EmojiProvider {
        EmojiProvider {
            categories: HashMap::new(),
            categories_order: vec![],
            emojis: vec![],
        }
    }

    fn get_categories_order(categories: &HashMap<String, EmojiCategory>) -> Vec<String> {
        categories.keys().cloned().collect()
    }

    fn from_json(str: &str) -> Result<EmojiProvider, ProviderError> {
        let emojis_serialized: Vec<SerializedEmoji> =
            serde_json::from_str(str).map_err(|_| ProviderError::NotJSON)?;

        let categories: HashSet<String> = emojis_serialized
            .iter()
            .map(|s| s.category.clone())
            .collect();

        let mut emojis: Vec<Emoji> = Vec::with_capacity(emojis_serialized.len());

        let mut categories: HashMap<String, EmojiCategory> = categories
            .into_iter()
            .map(|x| {
                (
                    x.clone(),
                    EmojiCategory {
                        name: x,
                        emojis: vec![],
                    },
                )
            })
            .collect();

        for semoji in emojis_serialized {
            let emoji = Emoji {
                name: semoji.name,
                unicode: semoji.emoji,
            };
            emojis.push(emoji.clone());
            categories
                .get_mut(&semoji.category)
                .expect("Can not get category")
                .insert(emoji);
        }

        let categories_order = EmojiProvider::get_categories_order(&categories);

        Ok(EmojiProvider {
            categories,
            categories_order,
            emojis,
        })
    }

    pub fn from_default() -> Result<EmojiProvider, ProviderError> {
        let json = DefaultEmojis::get("emojis.json").ok_or(ProviderError::DefaultsNotAvailable)?;
        let json = json.data;
        let json = json.into_owned();
        let json = String::from_utf8(json).map_err(|_| ProviderError::InvalidFile)?;
        let json = json.as_str();
        EmojiProvider::from_json(json)
    }
}
