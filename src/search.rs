use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct _SpotifyError(String);
impl fmt::Display for _SpotifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}
impl Error for _SpotifyError{}


