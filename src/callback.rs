use super::WritiumResult;

pub trait Callback {
    fn callback(&mut self, req: WritiumResult) -> WritiumResult;
}

impl<F> Callback for F
    where F: FnMut(WritiumResult) -> WritiumResult {

    fn callback(&mut self, req: WritiumResult) -> WritiumResult {
        (*self)(req)
    }
}
