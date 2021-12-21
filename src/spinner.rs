use crate::{str_shrink, BDELIM_ICON};
use spinners as pretty;
use terminal_spinners as cute;

pub trait Spinner {
    fn new() -> Self;
    fn message(&self, line: String);
    fn stop(&mut self);
}

pub struct PrettySpinner {
    spinner: Option<pretty::Spinner>,
}

impl Spinner for PrettySpinner {
    fn new() -> Self {
        Self {
            spinner: Some(pretty::Spinner::new(&pretty::Spinners::Moon, "".into())),
        }
    }

    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.message(str_shrink(&(line + BDELIM_ICON), 72)),
            _ => panic!(
                "{}PrettySpinner::spinner is already None.{}",
                BDELIM_ICON, BDELIM_ICON,
            ),
        };
    }

    fn stop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.stop();
        }
        println!("");
    }
}

pub struct CuteSpinner {
    spinner: Option<cute::SpinnerHandle>,
}

impl Spinner for CuteSpinner {
    fn new() -> Self {
        Self {
            spinner: Some(
                cute::SpinnerBuilder::new()
                    .spinner(&cute::DOTS)
                    .text("Unicorns!")
                    .prefix("  ")
                    .start(),
            ),
        }
    }

    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.text(str_shrink(&(line + BDELIM_ICON), 72)),
            _ => panic!(
                "{}CuteSpinner::spinner is already None.{}",
                BDELIM_ICON, BDELIM_ICON,
            ),
        };
    }

    fn stop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.done();
        }
    }
}
