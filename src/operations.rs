use rand::seq::SliceRandom;
use rustyline_async::SharedWriter;
use rodio::Sink;
use std::io::Write;
extern crate rand;
use rand::thread_rng;
use std::path::Path;
use walkdir::WalkDir;

fn is_music_file(x: &str) -> bool{
    x .contains(".wav") | x.contains(".mp3") | x.contains(".ogg") |x.contains(".flac")
}
#[derive(PartialEq, PartialOrd)]
pub enum Loop{
    NoLoop,
    LoopQueue,
    LoopSong
}

pub struct Handler{
    pub islooping: Loop,
    pub cur_song : Option<String>,
    pub queue : Vec<String>,
    pub stack : Vec<String>,
}

trait QueueHelper {
    fn queue_file(& mut self, s: String, stdout : & mut SharedWriter) -> Result<(), ()>;
    fn queue_folder(& mut self,  s:String, stdout: & mut SharedWriter, shuffle : bool) -> Result<(), ()>;
}
impl QueueHelper for Handler{
    fn queue_file(& mut self,  s: String, stdout : & mut SharedWriter)-> Result<(),()>{
        if Path::new(&s).is_file(){
            self.queue.push(s.clone());
            writeln!(stdout, "Queued {}", s.to_owned()).unwrap();
            Ok(())
        }
        else{
            let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.") + &s;
            if Path::new(&path).is_file(){
                self.queue.push(path);
                writeln!(stdout, "Queued {}", s.to_owned()).unwrap();
                Ok(())
            }
            else{
                Err(())
            }
        }
    }
    fn queue_folder(& mut self,  s:String, stdout: & mut SharedWriter, shuffle : bool) -> Result<(), ()>{
        let path:String;
        if !Path::new(&s).is_dir(){
            path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.") + &s;
        }
        else{
            path = s.clone();
        }
        if Path::new(&path).is_dir(){
            let files_and_stuff = WalkDir::new(&path);
            for file in files_and_stuff{
                let fpath = file.as_ref().unwrap().path().to_str().unwrap();
                if Path::is_file(Path::new(fpath)) && is_music_file(fpath){
                    self.queue.push(fpath.to_owned());
                    writeln!(stdout, "Queued {}", fpath.to_owned()).unwrap();
                }
            }
            if shuffle{
                let mut rng = thread_rng();
                self.queue.shuffle(& mut rng);
            }
            Ok(())
        }else{
            Err(())
        }
    }
}
pub trait AddToQueue {
    fn queue_handle(& mut self, s: Vec<&str>, stdout:& mut SharedWriter) -> Result<(), ()>;
}

impl AddToQueue for Handler {
    fn queue_handle(& mut self, s : Vec<&str>, stdout: & mut SharedWriter) -> Result<(), ()>{
        let filterds: Vec<&str> = s.into_iter().filter(|&i | i != "").collect();
        if (filterds.len() > 0) & (filterds.clone().into_iter().nth(0) != Some("")) {
            match filterds.clone().get(0) {
                Some(&"view") => {
                    match &self.cur_song.clone() {
                        Some(song) => {
                            write!(stdout, "Currently Playing{}\nUp next -> ", song).unwrap();
                            if &self.islooping == &Loop::LoopSong{
                                write!(stdout, "the same song :)\nAfter ->").unwrap();
                            }
                            writeln!(stdout, "{:?}", self.queue.clone()).unwrap();
                            Ok(())
                        },
                        None => Ok(writeln!(stdout, "No song is playing right now").unwrap()),
                    }
                },
                Some(&"shuffle") => {
                    let news:Vec<&str> = filterds.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect();
                    self.queue_folder(news.join(" "), stdout, true)
                },
                Some(_) => {
                    let mut request = filterds.clone().into_iter();
                    if request.any(|x |is_music_file(x)){
                        self.queue_file( filterds.join(" "), stdout)
                    }
                    else{
                        self.queue_folder(filterds.join(" "), stdout, false)
                    }
                }
                _ => Err(()),
            }
        }
        else{
            Err(())
        }
    }
}

pub trait LoopHandle {
    fn loop_handle(&mut self, s : &str, stdout: & mut SharedWriter);
}
impl LoopHandle for Handler{
    fn loop_handle(& mut self, s : &str, stdout: & mut SharedWriter){
        match s {
            "song" | "Song" => {self.islooping = Loop::LoopSong;  writeln!(stdout, "Now Looping Current Song").unwrap();},
            "queue" | "Queue" => {self.islooping = Loop::LoopQueue;writeln!(stdout, "Now Looping Current Queue").unwrap();},
            "cancel" | "Cancel" => {self.islooping = Loop::NoLoop;writeln!(stdout, "No longer looping").unwrap();}
            _ => ()
        }
    }   
}

pub trait Player {
    fn play_handle(& mut self, sink : &Sink, s: Vec<&str>, stdout: & mut SharedWriter);
}

impl Player for Handler{
    fn play_handle(& mut self, sink : &Sink, s: Vec<&str>, stdout: & mut SharedWriter) {
        if s.is_empty(){
            sink.play();
        }
        else {
            let mut temp: Vec<String> = self.queue.clone();
            self.queue.clear();
            match self.queue_handle(s.clone(), stdout){
                Err(_) => writeln!(stdout, "Could not play {}. Verify it exists or path exists", s.join(" ")).unwrap(),
                _ => {
                    sink.skip_one();
                    if self.cur_song.is_some(){
                        let s = self.cur_song.as_ref().unwrap().clone();
                        self.stack.push(s);
                    }
                },
            }
            self.queue.append(&mut temp);
        }     
    }
}

pub trait Backer{
    fn back_handle(&mut self, sink : &Sink, stdout: & mut SharedWriter);
}

impl Backer for Handler{
    fn back_handle(&mut self, sink : &Sink, stdout: & mut SharedWriter) {
        if self.stack.len() > 0{
            let mut new_queue: Vec<String> = Vec::new();
            let nextsong :String;
            let mut curque :Vec<String> = self.queue.clone();
            nextsong = self.stack.pop().unwrap();
            new_queue.push(nextsong);
            if self.cur_song.is_some(){
                new_queue.push(self.cur_song.as_ref().clone().unwrap().to_owned());
                self.cur_song = None;
            }
            new_queue.append(&mut curque);
            self.queue.clear();
            self.queue.append(&mut new_queue);
            sink.skip_one();
        }
        else{
            writeln!(stdout, "Cannot go back. No songs to go back to!").unwrap();
        }
    }
}

pub trait Skipper {
    fn skip_handle(& mut self, sink : &Sink, stdout : & mut SharedWriter);
}
impl Skipper for Handler{
    fn skip_handle(& mut self, sink : &Sink, stdout : & mut SharedWriter) {
        match self.islooping{
            Loop::LoopQueue => {
                let current:String = self.cur_song.as_ref().unwrap().clone();
                self.queue.push(current);
            }
            _ => ()
        }
        self.stack.push(self.cur_song.as_ref().clone().unwrap().to_owned());
        self.cur_song = None; 
        sink.skip_one();
        writeln!(stdout, "Skipped").unwrap();
    }
}