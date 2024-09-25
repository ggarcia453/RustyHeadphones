use std::fs::File;
use std::io::BufReader;
use operations::{AddToQueue, Backer, LoopHandle, Player, Skipper};
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
mod search;
pub fn volume_control(sink : &Sink, s: Vec<&str>, stdout : & mut SharedWriter){
    match s.get(0).unwrap().to_owned(){
        "View" | "view" | "VIEW" => writeln!(stdout, "Playing at {}", sink.volume()).unwrap(),
        "set" | "Set" | "SET" => {
            if s.get(1).is_some(){
                let val: Result<f32,_> = s.get(1).unwrap().parse();
                match val{
                    Err(_) => writeln!(stdout, "Error: volume should be set to between 0 and 1").unwrap(),
                    Ok(volume) => sink.set_volume(volume),
                }
            }else{
                writeln!(stdout, "Error: volume should be set to between 0 and 1").unwrap();
            }
        }
        "down" | "Down" => {
            let cur_volume = sink.volume();
            if cur_volume > 0.05{
                sink.set_volume(cur_volume - 0.05);
            }
            else{
                sink.set_volume(0.0);
            }
        }
        "up" | "Up" => {
            let cur_volume = sink.volume();
            if cur_volume < 0.95{
                sink.set_volume(cur_volume + 0.05);
            }
            else{
                sink.set_volume(1.0);
            }
        }
        x => writeln!(stdout, "Cannot find volume {}", x).unwrap(),
    }
    writeln!(stdout, "Volume now at {}", sink.volume()).unwrap();
}

fn play_song(sink : &Sink, s: String){
    let file = BufReader::new(File::open(s).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);
}

fn music_idle (sink : &Sink, handler: &mut operations::Handler, stdout: & mut SharedWriter){
    if sink.empty(){
        if handler.cur_song.is_some(){
            handler.stack.push(handler.cur_song.as_ref().unwrap().clone());
        }
        match handler.islooping{
            operations::Loop::NoLoop => {
                if !handler.queue.is_empty(){
                    play_song(sink, handler.queue.get(0).unwrap().clone());
                    handler.cur_song = Some(handler.queue.get(0).unwrap().to_owned());
                    handler.queue.remove(0);
                }
                else{
                    handler.cur_song = None;
                }
            },
            operations::Loop::LoopSong => {
                if handler.cur_song.is_some(){
                    let s = handler.cur_song.clone();
                    play_song(sink, s.unwrap());
                }
                else if !handler.queue.is_empty(){
                    play_song(sink, handler.queue.get(0).unwrap().clone());
                    let consume_queue: bool = handler.cur_song.is_none();
                    handler.cur_song = Some(handler.queue.get(0).unwrap().to_owned());
                    if consume_queue{
                        handler.queue.remove(0);
                    }
                }
            },
            operations::Loop::LoopQueue => {
                if handler.cur_song.is_some(){
                    let s = handler.cur_song.clone();
                    handler.queue.push(s.unwrap());
                }
                if !handler.queue.is_empty(){
                    play_song(sink, handler.queue.get(0).unwrap().clone());
                    handler.cur_song = Some(handler.queue.get(0).unwrap().to_owned());
                    handler.queue.remove(0);
                }
            },
        }
        if handler.cur_song.as_ref().is_some() &&  handler.islooping != operations::Loop::LoopSong{
            writeln!(stdout, "Now Playing {}", handler.cur_song.as_ref().unwrap().clone()).unwrap();
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), ReadlineError>{
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let (mut rl, mut stdout ) = Readline::new(">> ".into())?;
    dotenv().ok();
    let mut handler = operations::Handler{islooping: operations::Loop::NoLoop,cur_song: None, queue: Vec::new(), stack: Vec::new()};
    rl.should_print_line_on(true, false);
    let sink = Sink::try_new(&stream_handle).unwrap();
    loop{
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                music_idle(&sink, &mut handler, &mut stdout);
            }cmd = rl.readline() => match cmd {
                Ok(ReadlineEvent::Line(command)) => {
                    let command = command.trim().to_string();
                    let s:Vec<&str>  = command.split(" ").collect();
                    match s.clone().into_iter().nth(0) {
                        Some("exit") | Some("Exit") =>break,
                        Some("stop") => {sink.stop();handler.cur_song = None; handler.queue.clear();handler.stack.clear();},
                        Some("pause") => sink.pause(),
                        Some("play") => handler.play_handle(& sink, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some("shuffle") => {let mut rng = thread_rng(); handler.queue.shuffle(& mut rng);}
                        Some("skip") => handler.skip_handle(&sink, & mut stdout),
                        Some("queue") => {
                            match handler.queue_handle(s.clone().into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout){
                                Err(_) =>{
                                    let vec: Vec<&str> = s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect();
                                    let _ = writeln!(stdout, "Could not queue {}. Verify it exists or path is correct", vec.join(" "));
                                },
                                _ => ()
                            }
                        },
                        Some("volume") => volume_control(&sink, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some("loop") =>handler.loop_handle(s.get(1).unwrap().to_owned(), & mut stdout),
                        Some("restart") => {
                            if handler.cur_song.is_some(){
                                sink.skip_one();
                                let s = handler.cur_song.clone();
                                play_song(&sink, s.unwrap());
                                writeln!(stdout, "Restarting").unwrap();
                            }
                            else{
                                writeln!(stdout, "No song to restart").unwrap();
                            }
                        },
                        Some("back") => handler.back_handle(& sink, & mut stdout),
                        Some("spotify") => (),
                        Some ("") => (),
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
    let _ = writeln!(stdout, "Goodbye! :)");
    rl.flush()?;
    Ok(())
}
