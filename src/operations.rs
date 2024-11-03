use rand::seq::SliceRandom;
use rodio::Sink;
extern crate rand;
use rand::thread_rng;
use std::path::Path;
use walkdir::WalkDir;

pub fn is_music_file(x: &str) -> bool{
    x .contains(".wav") | x.contains(".mp3") | x.contains(".ogg") |x.contains(".flac")
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Loop{
    NoLoop,
    LoopQueue,
    LoopSong
}

unsafe impl Send for Loop {
    
}

#[derive(Debug)]
pub struct Handler{
    pub islooping: Loop,
    pub cur_song : Option<String>,
    pub queue : Vec<String>,
    pub stack : Vec<String>,
    pub volume : Option<f32>,
    defpath: String
}
impl Handler{
    pub fn new(m:String) -> Self{
        Self{
            islooping: Loop::NoLoop,
            cur_song: None,
            queue: Vec::new(),
            stack : Vec::new(),
            volume: None,
            defpath: m
        }
    }
    pub fn queue_handle(& mut self, s : Vec<&str>) -> Result<(), ()>{
        let filterds: Vec<&str> = s.into_iter().filter(|&i | i != "").collect();
        if (filterds.len() > 0) & (filterds.clone().into_iter().nth(0) != Some("")) {
            match filterds.clone().get(0) {
                Some(&"view") => {
                    match &self.cur_song.clone() {
                        Some(song) => {
                            print!("Currently Playing {}\nUp next -> ", song);
                            if &self.islooping == &Loop::LoopSong{
                                print!("the same song :)\nAfter ->");
                            }
                            println!("{:?}", self.queue.clone());
                            Ok(())
                        },
                        None => Ok(println!("No song is playing right now")),
                    }
                },
                Some(&"shuffle") => {
                    let news:Vec<&str> = filterds.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e).collect();
                    self.queue_folder(news.join(" "), true)
                },
                Some(_) => {
                    let mut request = filterds.clone().into_iter();
                    if request.any(|x |is_music_file(x)){
                        self.queue_file( filterds.join(" "))
                    }
                    else{
                        self.queue_folder(filterds.join(" "), false)
                    }
                }
                _ => Err(()),
            }
        }
        else{
            Err(())
        }
    }
    fn queue_file(& mut self,  s: String)-> Result<(),()>{
        let news = s.replace("\\", "");
        if Path::new(&news).is_file(){
            self.queue.push(news.to_owned());
            println!("Queued {}", news.to_owned());
            Ok(())
        }
        else{
            let path = (self.defpath.clone() + &s).replace("\\", "");
            if Path::new(&path).is_file(){
                self.queue.push(path);
                println!("Queued {}", news.to_owned());
                Ok(())
            }
            else{
                Err(())
            }
        }
    }
    fn queue_folder(& mut self,  s:String, shuffle : bool) -> Result<(), ()>{
        let path:String;
        if !Path::new(&s).is_dir(){
            path = (self.defpath.clone() + &s).replace("\\", "");
        }
        else{
            path = s.clone().replace("\\", "");
        }
        if Path::new(&path).is_dir() && &s != ""{
            let files_and_stuff = WalkDir::new(&path);
            for file in files_and_stuff{
                let fpath = file.as_ref().unwrap().path().to_str().unwrap();
                if Path::new(fpath).is_file() && is_music_file(fpath){
                    self.queue.push(fpath.to_owned());
                    print!("\x1B[1F");
                    println!("Queued {}", fpath.to_owned());
                    print!("\x1B[1E");
                }
            }
            if shuffle{
                let mut rng = thread_rng();
                self.queue.shuffle(& mut rng);
            }
            Ok(())
        }else{
            if shuffle{
                println!("You cannot queue shuffle a file.")
            }
            Err(())
        }
    }
    pub fn loop_handle(& mut self, s : &str){
        match s {
            "song" | "Song" => {self.islooping = Loop::LoopSong;  println!("Now Looping Current Song");},
            "queue" | "Queue" => {self.islooping = Loop::LoopQueue;println!("Now Looping Current Queue");},
            "cancel" | "Cancel" => {self.islooping = Loop::NoLoop;println!("No longer looping");}
            _ => println!("Not a valid loop option.")
        }
    }   
    pub fn play_handle(& mut self, sink : &Sink, s: Vec<&str>) {
        if s.is_empty(){
            sink.play();
        }
        else {
            let mut temp: Vec<String> = self.queue.clone();
            self.queue.clear();
            match self.queue_handle(s.clone()){
                Err(_) => println!("Could not play {}. Verify it exists or path exists", s.join(" ")),
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
    pub fn back_handle(&mut self, sink : &Sink) {
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
            println!("Cannot go back. No songs to go back to!");
        }
    }
    pub fn skip_handle(& mut self, sink : &Sink) {
        match self.islooping{
            Loop::LoopQueue => {
                let current:String = self.cur_song.as_ref().unwrap().clone();
                self.queue.push(current);
            }
            _ => ()
        }
        if self.cur_song.is_some(){
            self.stack.push(self.cur_song.as_ref().clone().unwrap().to_owned());
            self.cur_song = None; 
            sink.skip_one();
            println!("Skipped");
        }
        else{
            println!("No Song to skip");
        }
    }
    pub fn mute(&mut self, sink: &Sink) {
        if self.volume.is_none(){
            self.volume = Some(sink.volume());
            sink.set_volume(0.0);
        }
    }

    pub fn unmute(&mut self, sink : &Sink) {
        if self.volume.is_some(){
            sink.set_volume(self.volume.unwrap());
            self.volume = None;
        }
    }
}