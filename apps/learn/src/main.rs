use kreqo_learn::{AppState, app_logic};
use kreqo_ui::theme::apply_theme;
use xilem::masonry::theme::default_property_set;
use xilem::winit::error::EventLoopError;
use xilem::{EventLoop, Xilem};

fn main() -> Result<(), EventLoopError> {
    server_fn::client::set_server_url("http://localhost:8080");

    let mut def_props = default_property_set();
    apply_theme(&mut def_props);

    let app = Xilem::new(AppState::default(), app_logic).with_default_properties(def_props);
    app.run_in(EventLoop::with_user_event())?;

    Ok(())
}
