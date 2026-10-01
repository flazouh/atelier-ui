use crate::{
    select::{SelectOption},
    };
use super::structs::PromptModel;

impl From<&PromptModel> for SelectOption {
    fn from(model: &PromptModel) -> Self {
        Self { label: model.label.clone(), icon: model.icon, mark: model.mark.clone(), group: None }
    }
}
