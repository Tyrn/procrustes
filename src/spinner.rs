use spinners;

pub trait Spinner {
    fn new() -> Self;
    fn message(&self, line: String);
    fn stop(&mut self);
}

pub struct PrettySpinner {
    spinner: Option<spinners::Spinner>,
}

impl Spinner for PrettySpinner {
    fn new() -> Self {
        Self {
            spinner: Some(spinners::Spinner::new(&spinners::Spinners::Moon, "".into())),
        }
    }

    fn message(&self, line: String) {
        match &self.spinner {
            Some(spinner) => spinner.message(crate::str_shrink(&(line + crate::BDELIM_ICON), 72)),
            _ => panic!(
                "{}PrettySpinner::spinner is already None.{}",
                crate::BDELIM_ICON,
                crate::BDELIM_ICON,
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
