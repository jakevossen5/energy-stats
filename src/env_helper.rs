use std::env::{self, VarError};
pub fn get_api_key() -> Result<String, VarError> {
    env::var("EIA_KEY")
}
