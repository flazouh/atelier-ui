use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::{
    pr::{PrChipData},
    };

pub(crate) type PrHandler = Rc<dyn Fn(&PrChipData, &mut Window, &mut App)>;
