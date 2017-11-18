use super::ApiResult;

pub trait Callback {
    fn callback(&mut self, req: ApiResult) -> ApiResult;
}

impl<F> Callback for F
    where F: FnMut(ApiResult) -> ApiResult {

    fn callback(&mut self, req: ApiResult) -> ApiResult {
        (*self)(req)
    }
}
