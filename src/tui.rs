use ratatui::widgets::{StatefulWidget, Widget};

use crate::app::app::App;

pub struct AppWidget<'a>(pub &'a App);

pub struct AppState {

}

impl<'a> StatefulWidget for AppWidget<'a> {
    type State = AppState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        
    }
}
