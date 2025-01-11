use rand::seq::SliceRandom;
use rodio::Sink;
use rand::thread_rng;
use std::path::Path;
use walkdir::WalkDir;

pub fn is_music_file(x: &str) -> bool{
    x .contains(".wav") | x.contains(".mp3") | x.contains(".ogg") |x.contains(".flac")
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Loop{
    Straight,
    Queue,
    Song
}

impl std::fmt::Display for Loop{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
            islooping: Loop::Straight,
            cur_song: None,
            queue: Vec::new(),
            stack : Vec::new(),
            volume: None,
            defpath: m
        }
    }
    pub fn queue_handle(& mut self, s : String) -> Result<String, ()>{
        if s.starts_with("view"){
            match &self.cur_song.clone() {
                Some(song) => {
                    let mut return_string :String = "".to_string();
                    return_string.push_str(&format!("Currently Playing {}\nUp next -> ", song));
                    if self.islooping == Loop::Song{
                        return_string+= "the same song :)\nAfter ->";
                    }
                    for i in self.queue.clone(){
                        return_string = format!("{}{}\n", return_string, i);    
                    }
                    Ok(return_string.trim().to_string())
                },
                None => Ok("No song is playing right now".to_string()),
            }
        }
        else if s.starts_with("shuffle "){
            let news = s.strip_prefix("shuffle ").unwrap_or("").to_owned();
            self.queue_folder(news, true)
        }
        else if s.is_empty() {
            Err(())
        }
        else if is_music_file(&s){
            self.queue_file(s)
        }
        else {
            self.queue_folder(s, false)
        }
    
    }
    fn queue_file(& mut self,  s: String)-> Result<String,()>{
        let s = s.replace("\"", "");
        let p = Path::new(&s);
        let final_path = if p.is_relative() {
            self.defpath.clone() + &s
        } else {
            s.clone()
        };
        let newp = Path::new(&final_path);
        if newp.exists() {
            self.queue.push(final_path);
            Ok(format!("Queued {}", s))
        } else {
            Err(())
        }
    }
    fn queue_folder(& mut self,  s:String, shuffle : bool) -> Result<String, ()>{
        let s = s.replace("\"", "");
        let p = Path::new(&s);
        
        let final_path = if p.is_relative() {
            self.defpath.clone() + &s
        } else {
            s.clone()
        };
        
        let final_p = Path::new(&final_path);
        println!("Checking folder path: {}", final_path);
        
        if final_p.exists() {
            let mut rs = String::new();
            let files_and_stuff = WalkDir::new(final_p);
            
            for file in files_and_stuff.into_iter().flatten(){
                    let fpath = file.path();
                    if fpath.is_file() && is_music_file(fpath.to_str().unwrap_or("")) {
                        let file_path = fpath.to_str().unwrap_or("").to_owned();
                        self.queue.push(file_path.clone());
                        
                        if rs.is_empty() {
                            rs = format!("Queued {}", file_path)
                        } else {
                            rs = format!("{rs}\nQueued {}", file_path);
                        }
                    }
                
            }
            
            if shuffle {
                let mut rng = thread_rng();
                self.queue.shuffle(&mut rng);
            }
            
            if rs.is_empty() {
                Err(()) 
            } else {
                Ok(rs)
            }
        } else {
            Err(()) 
        }
    }
    pub fn loop_handle(& mut self, s : &str) -> String{
        match s {
            "song" | "Song" => {
                self.islooping = Loop::Song;  
                "Now Looping Current Song".to_string()
            },
            "queue" | "Queue" => {
                self.islooping = Loop::Queue;
                "Now Looping Current Queue".to_string()
            },
            "cancel" | "Cancel" => {
                self.islooping = Loop::Straight;
                "No longer looping".to_string()
            }
            "view" =>{
                format!("Current Loop option is {}", self.islooping)
            }
            _ => "Not a valid loop option.".to_string()
        }
    }   
    pub fn play_handle(& mut self, sink : &Sink, s: Option<String>)-> String {
        if s.is_none(){
            sink.play();
            String::new()
        }
        else {
            let mut temp: Vec<String> = self.queue.clone();
            self.queue.clear();
            let c = s.as_ref().unwrap().clone();
            if c == *""{
                sink.play();
                return String::new();
            }
            let res = self.queue_handle(s.unwrap());
            self.queue.append(&mut temp);
            match res{
                Err(_) => {
                    format!("Could not play {}. Verify it exists or path exists", c)
                },
                Ok(s)=>{
                    sink.skip_one();
                    if self.cur_song.is_some(){
                        let s = self.cur_song.as_ref().unwrap().clone();
                        self.stack.push(s);
                    }
                    s
                }
            }
        }     
    }
    pub fn back_handle(&mut self, sink : &Sink)-> String{
        if self.stack.is_empty(){
            let mut new_queue: Vec<String> = Vec::new();
            let mut curque :Vec<String> = self.queue.clone();
            let nextsong = self.stack.pop().unwrap();
            new_queue.push(nextsong);
            if self.cur_song.is_some(){
                new_queue.push(self.cur_song.as_ref().unwrap().to_owned());
                self.cur_song = None;
            }
            new_queue.append(&mut curque);
            self.queue.clear();
            self.queue.append(&mut new_queue);
            sink.skip_one();
            String::new()
        }
        else{
            String::from("Cannot go back. No songs to go back to!")
        }
    }
    pub fn skip_handle(& mut self, sink : &Sink)->String{
        if self.islooping == Loop::Queue{
            let current:String = self.cur_song.as_ref().unwrap().clone();
            self.queue.push(current);
        }
        if self.cur_song.is_some(){
            self.stack.push(self.cur_song.as_ref().unwrap().to_owned());
            self.cur_song = None; 
            sink.skip_one();
            String::from("Skipped")
        }
        else{
            String::from("No Song to skip")
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

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_is_music_file(){
        assert!(is_music_file("song.mp3"));
        assert!(is_music_file("song.wav"));
        assert!(is_music_file("song.ogg"));
        assert!(is_music_file("song.flac"));
        assert!(!is_music_file("song.jpg"));
        assert!(!is_music_file("song.mp4"));
        assert!(!is_music_file("song.txt"));
        assert!(!is_music_file("song"));
        assert!(!is_music_file(""));
    }
    #[test]
    fn handler_init(){
        let test_handler = Handler::new(String::from("C:\\Users\\g311\\Music\\"));
        assert!(test_handler.cur_song.is_none());
        assert!(test_handler.volume.is_none());
        assert_eq!(test_handler.defpath, "C:\\Users\\g311\\Music\\");
    }

    #[test]
    fn loop_equals(){
        assert_eq!(Loop::Queue, Loop::Queue);
        assert_eq!(Loop::Song, Loop::Song);
        assert_eq!(Loop::Straight, Loop:: Straight);
    }
    #[test]
    fn loop_not_equals(){
        assert_ne!(Loop::Queue, Loop::Song);
        assert_ne!(Loop::Song, Loop::Straight);
        assert_ne!(Loop::Straight, Loop::Queue);
    }
    #[test]
    fn handler_queue_view(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\g311\\Music\\"));
        let res = test_handler.queue_handle(String::from("view"));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "No song is playing right now".to_string());
    }
    #[test]
    fn handler_queue_file_exists(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        let result = test_handler.queue_file(String::from("Goodbye.mp3"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Queued Goodbye.mp3");
        assert_eq!(test_handler.queue.len(), 1);
    }    
    #[test]
    fn handler_queue_file_no_exists(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        let result = test_handler.queue_file(String::from("song.mp3"));
        assert!(result.is_err());
        assert_eq!(test_handler.queue.len(), 0);
    }
    #[test]
    fn handle_queue_folder_empty(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        //New is an empty folder
        let result = test_handler.queue_folder(String::from("New\\"), false);
        assert!(result.is_err());
        assert_eq!(test_handler.queue.len(), 0);
    }
    #[test]
    fn handler_queue_folder_exists(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        //Test Folder has three songs. All three should be queued.
        let result = test_handler.queue_folder(String::from("TestFolder\\"), false);
        assert!(result.is_ok());
        assert_eq!(test_handler.queue.len(), 3);
    }
    #[test]
    fn handler_queue_folder_no_exists(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        //NonExistentFolder does not exist.
        let result = test_handler.queue_folder(String::from("NonExistentFolder\\"), false);
        assert!(result.is_err());
        assert_eq!(test_handler.queue.len(), 0);
    }
    #[test]
    fn handler_queue_handle_file(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        let result = test_handler.queue_handle(String::from("Goodbye.mp3"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Queued Goodbye.mp3");
        assert_eq!(test_handler.queue.len(), 1);
    }
    #[test]
    fn handler_queue_handle_folder(){
        let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
        //Test Folder has three songs. All three should be queued.
        let result = test_handler.queue_handle(String::from("TestFolder\\"));
        assert!(result.is_ok());
        assert_eq!(test_handler.queue.len(), 3);
   }
   #[test]
   fn play_test_normal(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert_eq!(test_handler.play_handle(&sink, None), "");
   }

   #[test]
   fn play_handle_file(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    let res = test_handler.play_handle(&sink, Some(String::from("Goodbye.mp3")));
    assert_eq!(res, "Queued Goodbye.mp3");
   }
   #[test]
   fn play_handle_folder(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    let res = test_handler.play_handle(&sink, Some(String::from("TestFolder\\")));
    //"\n" means that mutliple files were added to queue
    assert!(res.contains("\n"));
    assert_eq!(test_handler.queue.len(), 3);
   }

   #[test]
   fn play_puts_infront(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert!(test_handler.queue_handle("Goodbye.mp3".to_string()).is_ok());
    test_handler.play_handle(&sink, Some("Noots.mp3".to_string()));
    assert_eq!(test_handler.queue.len(), 2);
    let expect = Some(String::from("C:\\Users\\gg311\\Music\\Noots.mp3"));
    assert_eq!(test_handler.queue.get(0).cloned(), expect);
   }
   
   #[test]
   fn loop_handle_check(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let res = test_handler.loop_handle("view");
    assert_eq!(res, "Current Loop option is Straight");
   }
   #[test]
   fn  loop_handle_set() {
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let res = test_handler.loop_handle("song");
    assert_eq!(res, "Now Looping Current Song");
    let res = test_handler.loop_handle("queue");
    assert_eq!(res, "Now Looping Current Queue");
    let res = test_handler.loop_handle("cancel");
    assert_eq!(res, "No longer looping");
   }
   #[test]
   fn back_handle_nothing(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert_eq!(test_handler.back_handle(&sink), "Cannot go back. No songs to go back to!");
   }
   #[test]
   fn back_handle_something(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert_eq!(test_handler.queue.len(), 0);
    test_handler.stack.push(String::from("Goodbye.mp3"));
    test_handler.back_handle(&sink);
    assert_eq!(test_handler.queue.len(), 1);
   }
   #[test]
   fn skip_nothing(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert_eq!(test_handler.skip_handle(&sink), "No Song to skip");
   }
   #[test]
   fn skip_loop_queue(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    test_handler.loop_handle("queue");
    assert!(test_handler.queue_handle(String::from("TestFolder\\")).is_ok());
    test_handler.cur_song = test_handler.queue.get(0).cloned();
    assert!(test_handler.cur_song.is_some());
    assert_eq!(3, test_handler.queue.len());
    test_handler.skip_handle(&sink);
    assert_eq!(4, test_handler.queue.len());
    assert!(test_handler.cur_song.is_none());
   }

   #[test]
   fn skip_no_loop(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert!(test_handler.queue_handle(String::from("TestFolder\\")).is_ok());
    test_handler.cur_song = test_handler.queue.get(0).cloned();
    assert!(test_handler.cur_song.is_some());
    assert_eq!(3, test_handler.queue.len());
    test_handler.skip_handle(&sink);
    assert_eq!(3, test_handler.queue.len());
    assert!(test_handler.cur_song.is_none());
   }

   #[test]
   fn mute_check(){
    let mut test_handler = Handler::new(String::from("C:\\Users\\gg311\\Music\\"));
    let (sink, _ )= rodio::Sink::new_idle();
    assert!(test_handler.volume.is_none());
    test_handler.mute(&sink);
    assert!(test_handler.volume.is_some());
    test_handler.unmute(&sink);
    assert!(test_handler.volume.is_none());
   }
}