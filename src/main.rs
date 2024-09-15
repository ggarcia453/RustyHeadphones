use std::fs::File;
use std::io::{self, BufRead, BufReader};
// use std::ops::Deref;
use rodio::{Decoder, OutputStream, Sink};
use dotenv::dotenv;

fn queue_file(sink : &Sink, s: String) -> Result<(), String>{
    let file: BufReader<File>;
    let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.");
    let ppath = File::open(path.to_owned() + &s);
    match ppath {
        Err(_) => {println!("Could not queue file"); return Ok(())},
        Ok(_) => file = BufReader::new(File::open(path.to_owned() + &s).unwrap())
    }
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    println!("Queued {}", s.to_owned());
    Ok(())
}

fn queue_folder()-> Result<(), String>{
    Ok(())
}

fn volume_control(sink : &Sink, s: Option<&str>){
    match s.to_owned(){
        None => (),
        Some("View") | Some("view") | Some("VIEW")=> println!("Playing at {}", sink.volume()),
        x => println!("Cannot find volume {}", x.unwrap()),
    }
}

fn queue(sink: &Sink, s : Vec<&str>) -> Result<(), String>{
    let testfile: String = std::env::var("TESTFILE").expect("TESTFILE must be set.");
    if (s.len() > 0) & (s.clone().into_iter().nth(0) != Some("")) {
        if s.clone().into_iter().any(|x |x .contains(".")){
            return queue_file(sink,s.join(" "));
        }
        else{
            return queue_folder();
        }
    }
    else{
        let file = BufReader::new(File::open(testfile.to_owned()).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
    }
    Ok(())
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    dotenv().ok();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let stdin = io::stdin();
    'repl: for line in stdin.lock().lines() {
        let li = line.unwrap().trim().to_owned();
        let s:Vec<&str>  = li.split(" ").collect();
        match s.clone().into_iter().nth(0) {
            Some("exit") | Some("Exit") => break 'repl,
            Some("stop") => sink.stop(),
            Some("pause") => sink.pause(),
            Some("play") => sink.play(),
            Some("skip") => sink.skip_one(),
            Some("queue") => queue(&sink, s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect()).unwrap(),
            Some("volume") => volume_control(&sink, s.into_iter().nth(1)),
            Some ("") => (),
            _ => println!("ERROR: Cannot do {} right now", &s.join(" ").to_string())
        }                
    }
}
