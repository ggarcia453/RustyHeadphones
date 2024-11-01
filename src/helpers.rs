use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;


#[derive(Default)]
pub struct HeadphoneHelper{
    pub commands: Vec<String>,
}
impl Hinter for HeadphoneHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}
impl Highlighter for HeadphoneHelper {}
impl Validator for HeadphoneHelper {}
impl Helper for HeadphoneHelper{}

impl Completer for HeadphoneHelper{
    type Candidate = rustyline::completion::Pair;
    fn complete(
            & self, // FIXME should be `&mut self`
            line: &str,
            pos: usize,
            _ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            let start = line[..pos].rfind(' ').map_or(0, |x| x + 1);
            let input = &line[start..pos];
    
            // Find matches for autocompletion
            let matches = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(input))
                .map(|cmd| rustyline::completion::Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
    
            Ok((start, matches))
    }
}

#[derive(Debug)]
enum ReplError {
    Io(std::io::Error),
    Channel(tokio::sync::mpsc::error::SendError<String>),
}

impl From<std::io::Error> for ReplError {
    fn from(err: std::io::Error) -> Self {
        ReplError::Io(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<String>> for ReplError {
    fn from(err: tokio::sync::mpsc::error::SendError<String>) -> Self {
        ReplError::Channel(err)
    }
}
