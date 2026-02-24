use xilem::WidgetView;
use xilem::core::{map_action, map_state};
use xilem::view::{MainAxisAlignment, flex_row};

use crate::component::ErrorView;

pub enum Submit {
    No,
    Yes,
    Cancel,
}

pub trait Form
where
    Self: Default + Sized + 'static,
{
    type Output;
    type Error: ErrorView;

    fn last_error(&mut self) -> &mut Option<Self::Error>;
    fn check(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    /// This function should do three things: validate the form, reset it and then return the result.
    /// Ideally, the data returned in the output should be taken directly from memory with `std::mem::take`. If not possible, the method `Self::reset` can be used instead.
    fn validate(&mut self) -> Result<Self::Output, Self::Error>;
    fn reset(&mut self) {
        *self = Self::default();
    }
    /// This function should call `Self::validate`, split the result to store the error in `Self::last_error` and return the output.
    fn submit(&mut self) -> Option<Self::Output> {
        match self.validate() {
            Ok(output) => {
                *self.last_error() = None;
                Some(output)
            }
            Err(error) => {
                *self.last_error() = Some(error);
                None
            }
        }
    }

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<Self>;
    fn error_view(&mut self) -> Option<impl WidgetView<Self, Submit> + use<Self>> {
        self.last_error().as_ref().map(|error| {
            map_action(
                map_state(
                    flex_row(error.view()).main_axis_alignment(MainAxisAlignment::Center),
                    move |state: &mut Self| state.last_error().as_mut().unwrap(),
                ),
                |_, _| Submit::No,
            )
        })
    }
}
