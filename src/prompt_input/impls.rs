use crate::select::SelectOption;
use super::structs::PromptModel;

impl From<&PromptModel> for SelectOption {
    fn from(model: &PromptModel) -> Self {
        Self { label: model.label.clone(), icon: model.icon, icon_color: None, mark: model.mark.clone(), group: None }
    }
}
