mod error;
pub use error::*;
use keyring::Entry;

const DEFAULT_SERVICE_NAME: &str = "figx-auth-service";
const DEFAULT_USER_NAME: &str = "figx-default-user";

pub fn set_token(token: &str) -> Result<()> {
    let entry = Entry::new(DEFAULT_SERVICE_NAME, DEFAULT_USER_NAME)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token() -> Result<Option<String>> {
    let entry = Entry::new(DEFAULT_SERVICE_NAME, DEFAULT_USER_NAME)?;

    let token = match entry.get_password() {
        Ok(token) => Some(token),
        Err(e) => match e {
            keyring::Error::NoEntry => None,
            e => return Err(e.into()),
        },
    };
    Ok(token)
}
