use storage_contracts::client::error::ClientError;

pub fn should_skip(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::ServiceNotAvailable | ClientError::Connection(_)
    )
}

pub fn skip_or_panic<T>(result: std::result::Result<T, ClientError>, context: &str) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) if should_skip(&error) => {
            eprintln!("SKIP {}: {}", context, error);
            None
        }
        Err(error) => panic!("{}: {}", context, error),
    }
}
