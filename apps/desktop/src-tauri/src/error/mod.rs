use whatnull_types::AppError;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppErrorWrapper(pub AppError);

impl From<AppError> for AppErrorWrapper {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl From<std::io::Error> for AppErrorWrapper {
    fn from(err: std::io::Error) -> Self {
        Self(AppError::Io(err.to_string()))
    }
}
