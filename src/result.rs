use response::HyperResponse;
use super::{Response, WritiumError};

pub type WritiumResult = Result<Response, WritiumError>;
impl From<Response> for WritiumResult {
    fn from(res: Response) -> WritiumResult {
        Ok(res)
    }
}
impl From<WritiumError> for WritiumResult {
    fn from(err: WritiumError) -> WritiumResult {
        Err(err)
    }
}
