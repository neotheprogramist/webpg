use salvo::http::{ParseError, StatusError};
use salvo::prelude::*;
use salvo::{Depot, async_trait};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("missing required state or parameter")]
    NotFound,
}

impl AppError {
    pub fn obtain<T: std::any::Any + Send + Sync>(depot: &Depot) -> Result<&T, Self> {
        depot.obtain::<T>().map_err(|_| AppError::NotFound)
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status = match &self {
            AppError::Parse(_) | AppError::NotFound => StatusError::bad_request(),
            _ => StatusError::internal_server_error(),
        };
        res.render(status);
    }
}
