use std::fs::File;
use std::io::BufReader;
use operations::{AddToQueue, Backer, LoopHandle, Muteable, Player, Skipper};
use rand::seq::SliceRandom;
use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use rodio::{Decoder, OutputStream, Sink};
use dotenv::dotenv;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
extern crate rand;
use rand::thread_rng;
mod operations;

struct RustyHeadphones{
    handler: operations::Handler,
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: rodio::OutputStreamHandle,
}

impl RustyHeadphones{
    fn play_song(& self, s: & String){
        let file = BufReader::new(File::open(s).unwrap());
        let source = Decoder::new(file).unwrap();
        self.sink.append(source);
    }
    fn music_idle (& mut self, stdout: & mut SharedWriter){
        if self.sink.empty(){
            if self.handler.cur_song.is_some(){
                self.handler.stack.push(self.handler.cur_song.as_ref().unwrap().clone());
            }
            match self.handler.islooping{
                operations::Loop::NoLoop => {
                    if !self.handler.queue.is_empty(){
                        let next_song = self.handler.queue.get(0);
                        self.play_song(next_song.as_ref().unwrap());
                        self.handler.cur_song = Some(next_song.unwrap().to_owned());
                        self.handler.queue.remove(0);
                    }
                    else{
                        self.handler.cur_song = None;
                    }
                },
                operations::Loop::LoopSong => {
                    if self.handler.cur_song.is_some(){
                        self.play_song(self.handler.cur_song.as_ref().unwrap());
                    }
                    else if !self.handler.queue.is_empty(){
                        let nextsong = self.handler.queue.get(0).unwrap().to_owned();
                        self.play_song(&nextsong);
                        self.handler.cur_song = Some(nextsong);
                        self.handler.queue.remove(0);
                    }
                },
                operations::Loop::LoopQueue => {
                    if self.handler.cur_song.is_some(){
                        let s = self.handler.cur_song.clone().unwrap();
                        self.handler.queue.push(s);
                    }
                    if !self.handler.queue.is_empty(){
                        let nextsong = self.handler.queue.get(0).unwrap().to_owned();
                        self.play_song(&nextsong);
                        self.handler.cur_song = Some(nextsong);
                        self.handler.queue.remove(0);
                    }
                },
            }
            if self.handler.cur_song.as_ref().is_some() &&  self.handler.islooping != operations::Loop::LoopSong{
                writeln!(stdout, "Now Playing {}", self.handler.cur_song.as_ref().unwrap().clone()).unwrap();
            }
        }
    }
    fn volume_control(&mut self, s:Vec<&str>, stdout: & mut SharedWriter){
        match s.get(0).unwrap().to_owned() {
            "View" | "view" | "VIEW" => writeln!(stdout, "Playing at {}", self.sink.volume()).unwrap(),
            "set" | "Set" | "SET" => {
                if s.get(1).is_some(){
                    let val: Result<f32,_> = s.get(1).unwrap().parse();
                    match val{
                        Err(_) => writeln!(stdout, "Error: volume should be set to between 0 and 1").unwrap(),
                        Ok(volume) => self.sink.set_volume(volume),
                    }
                }else{
                    writeln!(stdout, "Error: volume should be set to between 0 and 1").unwrap();
                }
            },
            "down" | "Down" => {
                let cur_volume = self.sink.volume();
                if cur_volume > 0.05{
                    self.sink.set_volume(cur_volume - 0.05);
                }
                else{
                    self.sink.set_volume(0.0);
                }
            }
            "up" | "Up" => {
                let cur_volume = self.sink.volume();
                if cur_volume < 0.95{
                    self.sink.set_volume(cur_volume + 0.05);
                }
                else{
                    self.sink.set_volume(1.0);
                }
            }
            x => writeln!(stdout, "Cannot find volume {}", x).unwrap(),
        }
        writeln!(stdout, "Volume now at {}", self.sink.volume()).unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), ReadlineError>{
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let (mut rl, mut stdout ) = Readline::new(">> ".into())?;
    dotenv().ok();
    let handler = operations::Handler{islooping: operations::Loop::NoLoop,cur_song: None, queue: Vec::new(), stack: Vec::new(), volume: None};
    rl.should_print_line_on(true, false);
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut rustyheadphone:RustyHeadphones = RustyHeadphones{handler : handler, sink:sink, _stream: _stream, _stream_handle: stream_handle};
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) =>{
                rustyheadphone.music_idle(& mut stdout);
            } cmd = rl.readline() => match cmd{
                Ok(ReadlineEvent::Line(command)) => {
                    let command = command.trim().to_string();
                    let s:Vec<&str>  = command.split(" ").collect();
                    match s.clone().get(0){
                        Some(&"exit") | Some(&"Exit") =>break,
                        Some(&"stop") => {
                            rustyheadphone.sink.stop();
                            rustyheadphone.handler.cur_song = None;
                            rustyheadphone.handler.queue.clear();
                            rustyheadphone.handler.stack.clear();
                        },
                        Some(&"pause") => rustyheadphone.sink.pause(),                        
                        Some(&"play") => rustyheadphone.handler.play_handle(& rustyheadphone.sink, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some(&"shuffle") => {
                            let mut rng = thread_rng(); 
                            rustyheadphone.handler.queue.shuffle(& mut rng);
                        }
                        Some(&"skip") => rustyheadphone.handler.skip_handle(&rustyheadphone.sink, & mut stdout),
                        Some(&"queue") => {
                            match rustyheadphone.handler.queue_handle(s.clone().into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout){
                                Err(_) =>{
                                    let vec: Vec<&str> = s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect();
                                    let _ = writeln!(stdout, "Could not queue {}. Verify it exists or path is correct", vec.join(" "));
                                },
                                _ => ()
                            }
                        },
                        Some(&"volume") => rustyheadphone.volume_control(s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some(&"loop") =>rustyheadphone.handler.loop_handle(s.get(1).unwrap().to_owned(), & mut stdout),
                        Some(&"restart") => {
                            if rustyheadphone.handler.cur_song.is_some(){
                                rustyheadphone.sink.skip_one();
                                let s = rustyheadphone.handler.cur_song.clone();
                                rustyheadphone.play_song(s.as_ref().unwrap());
                                writeln!(stdout, "Restarting").unwrap();
                            }
                            else{
                                writeln!(stdout, "No song to restart").unwrap();
                            }
                        },
                        Some(&"back") => rustyheadphone.handler.back_handle(& rustyheadphone.sink, & mut stdout),
                        Some(&"mute") => rustyheadphone.handler.mute(& rustyheadphone.sink),
                        Some(&"unmute") => rustyheadphone.handler.unmute(&rustyheadphone.sink),
                        _ => writeln!(stdout, "Error: Cannot do {} right now", &s.join(" "))?
                    }
                }
                Ok(ReadlineEvent::Eof) => {
                    break;
                }
                Ok(ReadlineEvent::Interrupted) => {
                    continue;
                }
                Err(e) => {
                    writeln!(stdout, "Error: {e:?}")?;
                    break;
                }
            }
        }
    }
    writeln!(stdout, "Goodbye! :)").unwrap();
    rl.flush()?;
    Ok(())
}
