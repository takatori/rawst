extern crate rustbox;

use rusoto::{
    DefaultCredentialsProvider,
    Region
};
use rusoto::kinesis::Record;

use hyper::Client;
use self::rustbox::Key;
use super::kinesis::KinesisHelper;
use super::screen::Screen;


enum State {
    Root,
    End,
    StreamList(Vec<String>),
    ShardList(String, Vec<String>),
    RecordList(String, Vec<Record>),  // stream_name, iterator_id
}
    
pub struct Controller {
    screen: Screen
}

impl Controller {
    
    pub fn new() -> Controller {
                    
        Controller {
            screen: Screen::new()
        }
    }

    pub fn run(&self) {
        
        let credentials = DefaultCredentialsProvider::new().unwrap();
        let client = Client::new();        
        let kinesis_helper = KinesisHelper::new(client, credentials, Region::ApNortheast1);
        
        let mut state: State = State::Root;

        
        loop {
            
            state = match state {
                State::StreamList(streams) => {
                    
                    self.screen.draw_strem_names(&streams);
                    
                    match self.screen.poll_event() {
                        Key::Char('q') => State::End,                                                
                        Key::Char(i) => {
                            
                            let n = i.to_digit(10).unwrap();
                            let ref stream_name = streams[n as usize];
                            
                            match kinesis_helper.describe_shards(stream_name) {
                                Ok(shards) => State::ShardList(stream_name.to_string(), shards),
                                Err(e) => State::Root,
                            }
                        }
                        _ => State::Root
                            
                    }
                },
                State::ShardList(stream_name, shards) => {
                    
                    self.screen.draw_shards(&shards);                    

                    match self.screen.poll_event() {
                        Key::Char('q') => State::End,                        
                        Key::Char(i) => {
                            
                            let n = i.to_digit(10).unwrap();
                            let ref shard_id = shards[n as usize];
                            
                            match kinesis_helper.get_shard_iterator(&stream_name, shard_id) {
                                Ok(shard_iterator) => State::RecordList(shard_iterator, vec!()),
                                Err(e) => State::Root,
                            }
                        }                        
                        _ => State::Root
                    }                    
                },
                State::RecordList(shard_iterator, records) => {

                    if records.len() != 0 {
                        // println!("{:?}", records[0].data);
                        println!("{:?}", String::from_utf8_lossy(&records[0].data));                        
                    }

                    // let a = kinesis_helper.convert(&records[0]);
                        
                    match kinesis_helper.get_records(&shard_iterator) {
                        Ok((r, i)) =>
                            match i {
                                Some(i) => State::RecordList(i, r),
                                None => {
                                    match self.screen.poll_event() {
                                        Key::Char('q') => State::End,
                                        _ => State::Root
                                    }
                                }
                            },
                        Err(e) => State::Root,
                    }
                    
                },
                State::Root => {
                    
                    self.screen.draw_help();

                    match self.screen.poll_event() {
                        Key::Char('q') => State::End,
                        Key::Char('l') => {
                            match kinesis_helper.list_streams() {
                                Ok(streams) => State::StreamList(streams),
                                Err(e) =>  State::Root
                            }
                        },
                        _ => State::Root
                            
                    }                    
                    
                },
                State::End => {
                    break;
                }
            };
                 
        }
    }
}

