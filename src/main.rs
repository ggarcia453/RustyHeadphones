use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rand::seq::SliceRandom;
use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use rodio::{Decoder, OutputStream, Sink};
use dotenv::dotenv;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
use walkdir::WalkDir;
extern crate rand;
use rand::thread_rng;

mod operations;

fn is_music_file(x: &str) -> bool{
    x .contains(".wav") | x.contains(".mp3") | x.contains(".ogg") |x.contains(".flac")
}

fn play_song(sink : &Sink, s: String){
    let file = BufReader::new(File::open(s).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);
}

fn music_idle (sink : &Sink, handler: &mut operations::Handler){
    if sink.empty(){
        match handler.islooping{
            operations::Loop::NoLoop => {
                if !handler.queue.is_empty(){
                    play_song(sink, handler.queue.get(0).unwrap().clone());
                    handler.cur_song = Some(handler.queue.get(0).unwrap().to_owned());
                    handler.queue.remove(0);
                }
            },
            operations::Loop::LoopSong => {
                if handler.cur_song.is_some(){
                    let s = handler.cur_song.clone();
                    play_song(sink, s.unwrap());
                }
                else if !handler.queue.is_empty(){
                    play_song(sink, handler.queue.get(0).unwrap().clone());
                    handler.cur_song = Some(handler.queue.get(0).unwrap().to_owned());
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
    }
}


fn queue_file(handler: &mut operations::Handler,  s: String, stdout : & mut SharedWriter){
    let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.") + &s;
    if Path::new(&path).is_file(){
        handler.queue.push(path);
        let _ = writeln!(stdout, "Queued {}", s.to_owned());
    }
    else{
        writeln!(stdout, "Could not queue file").unwrap();
    }
}

fn queue_folder(handler: &mut operations::Handler,  s:String, stdout: & mut SharedWriter, shuffle : bool){
    if shuffle{
    }
    let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.") + &s;
    if Path::new(&path).is_dir(){
        let files_and_stuff = WalkDir::new(&path);
        for file in files_and_stuff{
            let fpath = file.as_ref().unwrap().path().to_str().unwrap();
            if Path::is_file(Path::new(fpath)) && is_music_file(fpath){
                handler.queue.push(fpath.to_owned());
                let _ = writeln!(stdout, "Queued {}", fpath.to_owned());
            }
        }
        if shuffle{
            let mut rng = thread_rng();
            handler.queue.shuffle(& mut rng);
        }
    }else{
        let _ = writeln!(stdout, "Could not queue that folder. If you meant to queue a file makke sure you have a valid file extension");
    }
}

fn volume_control(sink : &Sink, s: Vec<&str>, stdout : & mut SharedWriter){
    match s.get(0).unwrap().to_owned(){
        "View" | "view" | "VIEW" => writeln!(stdout, "Playing at {}", sink.volume()).unwrap(),
        "set" | "Set" | "SET" => {
            if s.get(1).is_some(){
                let val: Result<f32,_> = s.get(1).unwrap().parse();
                match val{
                    Err(_) => writeln!(stdout, "Error: volume should be set to between 0 and 1").unwrap(),
                    Ok(volume) => {sink.set_volume(volume); writeln!(stdout, "Volume set to {}", volume).unwrap()},
                }
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
}

fn queue(handler: &mut operations::Handler,  s : Vec<&str>, stdout: & mut SharedWriter) {
    let _testfile: String = std::env::var("TESTFILE").expect("TESTFILE must be set.");
    if (s.len() > 0) & (s.clone().into_iter().nth(0) != Some("")) {
        if s.clone().into_iter().nth(0).unwrap()== "view" {
            if handler.cur_song.clone().is_some(){
                let _ = write!(stdout, "Currently Playing{}\nUp next -> ", handler.cur_song.clone().unwrap());
                match &handler.islooping{
                    operations::Loop::LoopSong => writeln!(stdout, "the same song :)").unwrap(),
                    _ => writeln!(stdout, "{:?}", handler.queue.clone()).unwrap()
                }
            }
            else{
                let _ = writeln!(stdout, "No song is playing right now");
            }
        }
        else if s.clone().into_iter().nth(0).unwrap()== "shuffle"{
            let news:Vec<&str> = s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect();
            queue_folder(handler, news.join(" "), stdout, true);
        }
        else if s.clone().into_iter().any(|x |is_music_file(x)){
            queue_file(handler, s.join(" "), stdout);
        }
        else{
            queue_folder(handler, s.join(" "), stdout, false);
        }
    }
    else{
        let _ = writeln!(stdout, "Error: Please provide a file or folder to queue");
    }
}

fn loop_handle(handler: &mut operations::Handler, s : &str, stdout: & mut SharedWriter){
    match s {
        "song" | "Song" => {handler.islooping = operations::Loop::LoopSong; let _ =  writeln!(stdout, "Now Looping Current Song");},
        "queue" | "Queue" => {handler.islooping = operations::Loop::LoopQueue;let _ = writeln!(stdout, "Now Looping Current Queue");},
        "cancel" | "Cancel" => {handler.islooping = operations::Loop::NoLoop;let _ = writeln!(stdout, "No longer looping");}
        _ => ()
    }
}

fn skip_handle(handler: & mut operations::Handler,sink : &Sink, stdout: & mut SharedWriter){
    match handler.islooping{
        operations::Loop::LoopQueue => {
            let current:String = handler.cur_song.as_ref().unwrap().clone();
            handler.queue.push(current);
        }
        _ => ()
    }
    handler.cur_song = None; 
    sink.skip_one();
    let _ = writeln!(stdout, "Skipped");
}

#[tokio::main]
async fn main() -> Result<(), ReadlineError>{
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let (mut rl, mut stdout ) = Readline::new(">> ".into())?;
    let mut handler = operations::Handler{islooping: operations::Loop::NoLoop,cur_song: None, queue: Vec::new()};
    rl.should_print_line_on(true, false);
    dotenv().ok();
    let sink = Sink::try_new(&stream_handle).unwrap();
    loop{
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                music_idle(&sink, &mut handler);
            }cmd = rl.readline() => match cmd {
                Ok(ReadlineEvent::Line(command)) => {
                    let command = command.trim().to_string();
                    let s:Vec<&str>  = command.split(" ").collect();
                    match s.clone().into_iter().nth(0) {
                        Some("exit") | Some("Exit") =>break,
                        Some("stop") => sink.stop(),
                        Some("pause") => sink.pause(),
                        Some("play") => sink.play(),
                        Some("shuffle") => {let mut rng = thread_rng(); handler.queue.shuffle(& mut rng);}
                        Some("skip") => skip_handle(&mut handler, &sink, & mut stdout),
                        Some("queue") => queue(& mut handler, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some("volume") => volume_control(&sink, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout),
                        Some("loop") => loop_handle(& mut handler, s.get(1).unwrap().to_owned(), & mut stdout),
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
