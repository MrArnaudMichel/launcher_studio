use gtk4::ListBoxRow;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct UiState {
    pub selected_path: Option<PathBuf>,
    pub in_edit: bool,
    pub temp_row: Option<ListBoxRow>,
}

pub type SharedState = Rc<RefCell<UiState>>;

pub fn new_state() -> SharedState {
    Rc::new(RefCell::new(UiState::default()))
}
