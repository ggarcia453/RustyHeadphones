use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use rodio::{Decoder, OutputStream, Sink};
use dotenv::dotenv;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

mod operations;

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
            },
            operations::Loop::LoopQueue => (),
        }
    }
}


fn queue_file(handler: &mut operations::Handler,  s: String, stdout : & mut SharedWriter) -> Result<(), String>{
    let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.") + &s;
    if Path::new(&path).exists(){
        handler.queue.push(path);
    }
    // let ppath = File::open(path.to_owned() + &s);
    // match ppath {
    //     Err(_) => {writeln!(stdout, "Could not queue file").unwrap(); return Ok(())},
    //     Ok(_) => file = BufReader::new(File::open(path.to_owned() + &s).unwrap())
    // }
    // let source = Decoder::new(file).unwrap();
    // sink.append(source);
    let _ = writeln!(stdout, "Queued {}", s.to_owned());
    Ok(())    
}

fn queue_folder()-> Result<(), String>{
    Ok(())
}

fn volume_control(sink : &Sink, s: Option<&str>, stdout : & mut SharedWriter){
    match s.to_owned(){
        None => (),
        Some("View") | Some("view") | Some("VIEW")=> writeln!(stdout, "Playing at {}", sink.volume()).unwrap(),
        x => writeln!(stdout, "Cannot find volume {}", x.unwrap()).unwrap(),
    }
}

fn queue(handler: &mut operations::Handler,  s : Vec<&str>, stdout: & mut SharedWriter) -> Result<(), String>{
    let _testfile: String = std::env::var("TESTFILE").expect("TESTFILE must be set.");
    if (s.len() > 0) & (s.clone().into_iter().nth(0) != Some("")) {
        if s.clone().into_iter().any(|x |x .contains(".")){
            return queue_file(handler, s.join(" "), stdout);
        }
        else{
            return queue_folder();
        }
    }
    else{
        // let file = BufReader::new(File::open(testfile.to_owned()).unwrap());
        // let source = Decoder::new(file).unwrap();
        // sink.append(source);
    }
    Ok(())
}

fn loop_handle(handler: &mut operations::Handler, s : &str, stdout: & mut SharedWriter){
    match s {
        "song" | "Song" => {handler.islooping = operations::Loop::LoopSong; let _ =  writeln!(stdout, "Now Looping Current Song");},
        "queue" | "Queue" => {handler.islooping = operations::Loop::LoopQueue;let _ = writeln!(stdout, "Now Looping Current Queue");},
        "cancel" | "Cancel" => {handler.islooping = operations::Loop::NoLoop;let _ = writeln!(stdout, "No longer looping");}
        _ => ()
    }
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
                        Some("skip") => sink.skip_one(),
                        Some("queue") => queue(& mut handler, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect(), & mut stdout).unwrap(),
                        Some("volume") => volume_control(&sink, s.into_iter().nth(1), & mut stdout),
                        Some("loop") => loop_handle(& mut handler, s.get(1).unwrap().to_owned(), & mut stdout),
                        Some("detach") => {sink.sleep_until_end(); break;},
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
