use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::env;

use crate::operations::is_music_file;

#[derive(Default)]
pub struct HeadphoneHelper{
    pub commands: Vec<String>,
    pub filenames: FilenameCompleter,
    pub specialcommands:Vec<String>,
    musicpath: String

}
impl HeadphoneHelper {
    pub fn new(m:String) -> Self{
        let commands = vec![
        "exit",
        "stop",
        "pause",
        "play",
        "shuffle",
        "skip",
        "queue",
        "volume",
        "loop",
        "restart",
        "back",
        "mute",
        "unmute",
        "help",
        "speed",
        ].into_iter().map(|s|s.to_string()).collect();
        let specialcommands = vec![
            "shuffle",
            "view"
        ].into_iter().map(|s|s.to_string()).collect();
        HeadphoneHelper{
            commands, filenames: FilenameCompleter::new(), specialcommands, musicpath:m
        }
    }
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

fn validfile(s:&str)-> bool{
    (s.contains(".") && is_music_file(s)) || !s.contains(".")
}

impl Completer for HeadphoneHelper{
    type Candidate = rustyline::completion::Pair;
    fn complete(
            & self, // FIXME should be `&mut self`
            line: &str,
            pos: usize,
            ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            let start = line[..pos].rfind(' ').map_or(0, |x| x + 1);
            let input = &line[start..pos];
            if line.contains(char::is_whitespace){
                if line.starts_with("loop"){
                    let options:Vec<String> = vec!["song", "queue", "cancel", "view"].into_iter().map(|s|s.to_string()).collect();
                    let partners:Vec<Pair> = options.iter().filter(|cmd| cmd.starts_with(input))
                    .map(|cmd| rustyline::completion::Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    })
                    .collect();
                    Ok((start, partners))
                }
                else if (line.starts_with("volume") || line.starts_with("speed")) && line.chars().into_iter().filter(|ch | ch.is_whitespace() && ch != &'\n').count() < 2{
                    let options:Vec<String> = vec!["up", "down", "set", "view"].into_iter().map(|s|s.to_string()).collect();
                    let partners:Vec<Pair> = options.iter().filter(|cmd| cmd.starts_with(input))
                    .map(|cmd| rustyline::completion::Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    })
                    .collect();
                    Ok((start, partners))
                }
                else if line.starts_with("queue") && !line.contains("view") && !is_music_file(line){
                    let path = &self.musicpath;
                    env::set_current_dir(path)?;
                    let (new_pos, mut filename_completions) = self.filenames.complete(line, pos, ctx)?;
                    if !line.contains("shuffle") && !line.contains("view"){
                        let ff: Vec<Pair> = self.specialcommands
                        .iter()
                        .filter(|cmd| cmd.starts_with(input))
                        .map(|cmd| rustyline::completion::Pair {
                            display: cmd.clone(),
                            replacement: cmd.clone(),
                        })
                        .collect();
                        filename_completions.extend(ff);
                    }
                    Ok((new_pos, filename_completions.into_iter().filter(|x|validfile(&x.display)).collect()))
                }
                else if line.starts_with("play") && !is_music_file(line){
                    let path = &self.musicpath;
                    env::set_current_dir(path)?;
                    let (new_pos, filename_completions) = self.filenames.complete(line, pos, ctx)?;
                    Ok((new_pos, filename_completions.into_iter().filter(|x|validfile(&x.display)).collect()))
                }
                else{
                    Ok((start, Vec::new()))
                }
            }
            else{
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::Context;
    use rustyline::history::DefaultHistory;

    fn setup() -> (HeadphoneHelper, Context<'static>) {
        let music_path = String::from("C:\\Users\\gg311\\Music"); // Use a test directory
        let helper = HeadphoneHelper::new(music_path);
        let history = Box::leak(Box::new(DefaultHistory::new()));
        let ctx = Context::new(history);
        (helper, ctx)
    }

    #[test]
    fn command_completion() {
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("pl", 2, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(completions.iter().any(|p| p.display == "play"));
        let (pos, completions) = helper.complete("pa", 2, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(completions.iter().any(|p| p.display == "pause"));
        let (pos, completions) = helper.complete("ex", 2, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(completions.iter().any(|p| p.display == "exit"));
    }
    #[test]
    fn empty_completion(){
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert_eq!(completions.len(), helper.commands.len());
    }

    #[test]
    fn loop_subcommands() {
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("loop s", 6, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.iter().any(|p| p.display == "song"));
        let (pos, completions) = helper.complete("loop q", 6, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.iter().any(|p| p.display == "queue"));
        let (pos, completions) = helper.complete("loop c", 6, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.iter().any(|p| p.display == "cancel"));
        let (pos, completions) = helper.complete("loop v", 6, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.iter().any(|p| p.display == "view"));
    }
    #[test]
    fn loop_allcommands(){
        let (helper, ctx) = setup();
        let (_, completions) = helper.complete("loop ", 5, &ctx).unwrap();
        let expected = vec!["song", "queue", "cancel", "view"];
        assert!(expected.iter().all(|&cmd| 
            completions.iter().any(|p| p.display == cmd)));
    }

    #[test]
    fn volume_subcommands(){
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("volume u", 8, &ctx).unwrap();
        assert_eq!(pos, 7);
        assert!(completions.iter().any(|p| p.display == "up"));
        let (pos, completions) = helper.complete("volume d", 8, &ctx).unwrap();
        assert_eq!(pos, 7);
        assert!(completions.iter().any(|p| p.display == "down"));
        let (pos, completions) = helper.complete("volume v", 8, &ctx).unwrap();
        assert_eq!(pos, 7);
        assert!(completions.iter().any(|p| p.display == "view"));
        let (pos, completions) = helper.complete("volume s", 8, &ctx).unwrap();
        assert_eq!(pos, 7);
        assert!(completions.iter().any(|p| p.display == "set"));
    }

    #[test]
    fn volume_allcommands(){
        let (helper, ctx) = setup();
        let (_, completions) = helper.complete("volume ", 7, &ctx).unwrap();
        let expected = vec!["up", "down", "set", "view"];
        assert!(expected.iter().all(|&cmd| 
            completions.iter().any(|p| p.display == cmd)));
    }

    #[test]
    fn speed_subcommands(){
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("speed u", 7, &ctx).unwrap();
        assert_eq!(pos, 6);
        assert!(completions.iter().any(|p| p.display == "up"));
        let (pos, completions) = helper.complete("speed d", 7, &ctx).unwrap();
        assert_eq!(pos, 6);
        assert!(completions.iter().any(|p| p.display == "down"));
        let (pos, completions) = helper.complete("speed v", 7, &ctx).unwrap();
        assert_eq!(pos, 6);
        assert!(completions.iter().any(|p| p.display == "view"));
        let (pos, completions) = helper.complete("speed s", 7, &ctx).unwrap();
        assert_eq!(pos, 6);
        assert!(completions.iter().any(|p| p.display == "set"));
    }

    #[test]
    fn speed_allcommands(){
        let (helper, ctx) = setup();
        let (_, completions) = helper.complete("speed ", 6, &ctx).unwrap();
        let expected = vec!["up", "down", "set", "view"];
        assert!(expected.iter().all(|&cmd| 
            completions.iter().any(|p| p.display == cmd)));
    }

    #[test]
    fn queue_command_tests() {
        let (helper, ctx) = setup();
        let (pos, completions) = helper.complete("queue sh", 8, &ctx).unwrap();
        assert_eq!(pos, 6);
        assert!(completions.iter().any(|p| p.display == "shuffle"));
        let (_, completions) = helper.complete("queue view ", 11, &ctx).unwrap();
        assert!(completions.iter().all(|p| !p.display.contains("shuffle")));
    }

    #[test]
    fn helper_init_tests() {
        let music_path = String::from("C:\\Users\\gg311\\Music\\");
        let helper = HeadphoneHelper::new(music_path.clone());
        assert!(helper.commands.contains(&String::from("play")));
        assert!(helper.commands.contains(&String::from("pause")));
        assert!(helper.commands.contains(&String::from("stop")));
        assert!(helper.specialcommands.contains(&String::from("shuffle")));
        assert!(helper.specialcommands.contains(&String::from("view")));
        assert_eq!(helper.musicpath, music_path);
    }
}