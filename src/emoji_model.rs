use std::cell::RefCell;

use crate::{
    emoji::{EmojiGroupWrapper, EmojiWrapper},
    EmojiGroupModel, EmojiModel, EmojiSkinToneModel,
};
use slint::{Model, ModelNotify, ModelRc, VecModel};

impl From<EmojiWrapper> for EmojiModel {
    fn from(e: EmojiWrapper) -> EmojiModel {
        use crate::emoji::TwemojiFilename;

        let filename = e.get_filename_path();
        let image = slint::Image::load_from_path(&filename);

        EmojiModel {
            name: e.name().into(),
            code: e.code().into(),
            image: image.unwrap_or_default(),
            skin_tones: ModelRc::new(VecModel::from(match e.skin_tones() {
                Some(iterator) => iterator
                    .map(EmojiSkinToneModel::try_from)
                    .flat_map(|f| f.ok())
                    .collect(),
                None => vec![],
            })),
        }
    }
}

impl TryFrom<EmojiWrapper> for EmojiSkinToneModel {
    type Error = ();

    fn try_from(e: EmojiWrapper) -> Result<EmojiSkinToneModel, ()> {
        use crate::emoji::TwemojiFilename;

        let filename = e.get_filename_path();
        let image = slint::Image::load_from_path(&filename);

        e.skin_tone()
            .map(|skin_tone| EmojiSkinToneModel {
                code: e.code().into(),
                image: image.unwrap_or_default(),
                skin_tone: skin_tone.into(),
            })
            .ok_or(())
    }
}

// Model of a list of emojis.

/// A model that contains a list of emojis.
struct VecEmojiListModel {
    initial: RefCell<Vec<EmojiModel>>,
    vec: RefCell<Vec<EmojiModel>>,
    filter: RefCell<String>,
    notify: ModelNotify,
}

impl VecEmojiListModel {
    pub fn new(emojis: Vec<EmojiWrapper>) -> Self {
        let emojis: Vec<EmojiModel> = emojis.into_iter().map(|x| x.into()).collect();
        Self {
            initial: RefCell::new(emojis.clone()),
            vec: RefCell::new(emojis),
            notify: ModelNotify::default(),
            filter: RefCell::new(String::new()),
        }
    }

    fn filter_down(&self, filter: String) {
        let filter = filter.to_lowercase();
        let mut emojis = self.vec.borrow_mut();
        emojis.retain(|x| x.name.contains(&filter));
        self.filter.replace(filter);
        self.notify.reset()
    }

    pub fn filter_up(&self, filter: String) {
        let filter = filter.to_lowercase();
        let mut emojis = self.initial.borrow().clone();
        emojis.retain(|x| x.name.to_lowercase().contains(&filter));
        self.filter.replace(filter);
        self.vec.replace(emojis);
        self.notify.reset()
    }
}

impl Model for VecEmojiListModel {
    type Data = EmojiModel;

    fn row_count(&self) -> usize {
        self.vec.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.vec.borrow().get(row).cloned()
    }

    fn set_row_data(&self, _: usize, _: Self::Data) {
        unimplemented!("A model should not be modified from the user.")
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Model of emoji groups

/// A model that contains a list of groups of emojis.
pub struct VecEmojiGroupModel {
    vec: RefCell<Vec<EmojiGroupModel>>,
    filter: RefCell<String>,
    notify: ModelNotify,
}

impl VecEmojiGroupModel {
    pub fn new() -> Self {
        let mut vec = Vec::new();
        for group in emojis::Group::iter() {
            let emojis = group.emojis().map(EmojiWrapper).collect();
            let model = VecEmojiListModel::new(emojis);
            let model = ModelRc::new(model);
            let group_model = EmojiGroupModel {
                title: EmojiGroupWrapper(group).group_name().into(),
                image: model
                    .iter()
                    .next()
                    .map(|e| e.image.clone())
                    .unwrap_or_default(),
                emojis: model,
            };
            vec.push(group_model);
        }

        Self {
            vec: RefCell::new(vec),
            filter: RefCell::new(String::new()),
            notify: ModelNotify::default(),
        }
    }

    pub fn filter(&self, filter: String) {
        let is_filter_down = {
            let old_filter = self.filter.borrow();
            let filter = filter.to_lowercase();
            filter.len() > old_filter.len()
        };
        if is_filter_down {
            for group in self.vec.borrow_mut().iter_mut() {
                group
                    .emojis
                    .as_any()
                    .downcast_ref::<VecEmojiListModel>()
                    .unwrap()
                    .filter_down(filter.clone());
            }
        } else {
            for group in self.vec.borrow_mut().iter_mut() {
                group
                    .emojis
                    .as_any()
                    .downcast_ref::<VecEmojiListModel>()
                    .unwrap()
                    .filter_up(filter.clone());
            }
        };
        self.filter.replace(filter);
    }
}

impl Model for VecEmojiGroupModel {
    type Data = EmojiGroupModel;

    fn row_count(&self) -> usize {
        self.vec.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.vec.borrow().get(row).cloned()
    }

    fn set_row_data(&self, _: usize, _: Self::Data) {
        unimplemented!("A model should not be modified from the user.")
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
