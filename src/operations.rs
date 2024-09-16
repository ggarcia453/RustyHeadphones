
pub enum Loop{
    NoLoop,
    LoopQueue,
    LoopSong
}

pub struct Handler{
    pub islooping: Loop,
    pub cur_song : Option<String>,
    pub queue : Vec<String>,
}