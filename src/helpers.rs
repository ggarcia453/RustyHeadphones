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
