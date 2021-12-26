use crate::{str_shrink, BDELIM_ICON};
use spinner as daddy;
use spinners as pretty;
use std::time::Duration;
use terminal_spinners as cute;

static MOON: [&'static str; 8] = [" 🌑", " 🌒", " 🌓", " 🌔", " 🌕", " 🌖", " 🌗", " 🌘"];

pub trait Shrinker {
    fn shrink_pretty(&self) -> String;
}

impl Shrinker for String {
    fn shrink_pretty(&self) -> String {
        str_shrink(&self, 72) + BDELIM_ICON
    }
}

pub struct DummySpinner {}

#[allow(dead_code)]
impl DummySpinner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Spinner for DummySpinner {
    fn message(&self, _line: String) {}
    fn stop(&mut self) {}
}

pub trait Spinner {
    fn message(&self, line: String);
    fn stop(&mut self);
    fn adieu(&self, owner: &str) -> String {
        format!(
            "{}{}::spinner is already None.{}",
            BDELIM_ICON, owner, BDELIM_ICON
        )
    }
}

pub struct DaddySpinner {
    spinner: Option<daddy::SpinnerHandle>,
}

#[allow(dead_code)]
impl DaddySpinner {
    pub fn new() -> Self {
        Self {
            spinner: Some(
                daddy::SpinnerBuilder::new("".into())
                    .spinner(MOON.to_vec())
                    .step(Duration::from_millis(80))
                    .start(),
            ),
        }
    }
}

impl Spinner for DaddySpinner {
    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.update(line.shrink_pretty()),
            _ => panic!("{}", self.adieu("DaddySpinner")),
        };
    }

    fn stop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.close();
        }
        println!("");
    }
}

pub struct PrettySpinner {
    spinner: Option<pretty::Spinner>,
}

#[allow(dead_code)]
impl PrettySpinner {
    pub fn new() -> Self {
        Self {
            spinner: Some(pretty::Spinner::new(&pretty::Spinners::Moon, "".into())),
        }
    }
}

impl Spinner for PrettySpinner {
    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.message(line.shrink_pretty()),
            _ => panic!("{}", self.adieu("PrettySpinner")),
        };
    }

    fn stop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.stop();
        }
        println!("");
    }
}

pub struct CutieSpinner {
    spinner: Option<cute::SpinnerHandle>,
}

#[allow(dead_code)]
impl CutieSpinner {
    pub fn new() -> Self {
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
}

impl Spinner for CutieSpinner {
    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.text(line.shrink_pretty()),
            _ => panic!("{}", self.adieu("CutieSpinner")),
        };
    }

    fn stop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.done();
        }
    }
}
