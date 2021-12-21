pub trait Spinner {
    fn new() -> Self;
    fn message(&self, line: String);
    fn stop(&self);
}

pub struct PrettySpinner {

}