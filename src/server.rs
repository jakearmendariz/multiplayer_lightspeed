use actix::prelude::*;
use actix_broker::BrokerSubscribe;

use std::collections::HashMap;
use std::mem;

use crate::message::{ChatMessage, JoinRoom, LeaveRoom, ListRooms, SendMessage, GetGame};

type Client = Recipient<ChatMessage>;
type Room = HashMap<usize, Client>;
use std::thread;
use crate::lightspeed::{GameState, Rocket};
use std::fs::File;
use std::io::prelude::*;

extern crate scoped_threadpool;
use scoped_threadpool::Pool;

use std::time::Duration;


#[derive(Default)]
pub struct WsChatServer {
    rooms: HashMap<String, Room>,
    game_state:GameState
}

impl WsChatServer {
    fn take_room(&mut self, room_name: &str) -> Option<Room> {
        let room = self.rooms.get_mut(room_name)?;
        let room = mem::replace(room, HashMap::new());
        Some(room)
    }

    pub fn write_state(&mut self, state:String) -> std::io::Result<()>{
        let mut file = File::create("state.txt")?;
        println!("Writing \"{}\" to state.txt", state);
        file.write_all(&state[..].as_bytes())?;
        Ok(())
    }

    fn add_player(&mut self, id:usize, width:i32, height:i32) {
        if self.game_state.num_players() == 0 {
            self.game_state.build();
            // self.run_game();
        }else {
            println!("Num players = {}\n\n\n", self.game_state.num_players());
        }
        self.game_state.add_player(id, width, height);
        self.game_state.update();
        self.game_state.print_state();
    }

    fn get_game_state(&mut self) -> String{
        // self.game_state.update();
        self.game_state.to_json_string()
    }

    fn run_game(&mut self) {
        let mut pool = Pool::new(4);
        pool.scoped(|scope| {
            loop {
                println!("running game... {}", self.game_state.to_json_string());
                self.game_state.update();
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }

    fn add_client_to_room(&mut self,room_name: &str,id: Option<usize>,client: Client) -> usize {
        let mut id = id.unwrap_or_else(rand::random::<usize>);

        if let Some(room) = self.rooms.get_mut(room_name) {
            loop {
                if room.contains_key(&id) {
                    id = rand::random::<usize>();
                } else {
                    break;
                }
            }

            room.insert(id, client);
            return id;
        }

        // Create a new room for the first client
        let mut room: Room = HashMap::new();

        room.insert(id, client);
        self.rooms.insert(room_name.to_owned(), room);

        id
    }

    fn send_chat_message(&mut self, room_name: &str, msg: &str, _src: usize,) -> Option<()> {
        let mut room = self.take_room(room_name)?;

        for (id, client) in room.drain() {
            if client.do_send(ChatMessage(msg.to_owned())).is_ok() {
                self.add_client_to_room(room_name, Some(id), client);
            }
        }

        Some(())
    }
}

impl Actor for WsChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<LeaveRoom>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);
    }
}

impl Handler<JoinRoom> for WsChatServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        println!("server.rs: join room");
        let JoinRoom(room_name, client_name, client) = msg;

        let id = self.add_client_to_room(&room_name, None, client);
        let join_msg = format!(
            "{} joined {}",
            client_name.unwrap_or_else(|| "anon".to_string()),
            room_name
        );

        self.send_chat_message(&room_name, &join_msg, id);
        MessageResult(id)
    }
}

impl Handler<LeaveRoom> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        if let Some(room) = self.rooms.get_mut(&msg.0) {
            room.remove(&msg.1);
        }
    }
}

impl Handler<ListRooms> for WsChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.rooms.keys().cloned().collect())
    }
}

impl Handler<SendMessage> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(room_name, id, msg) = msg;
        self.send_chat_message(&room_name, &msg, id);
    }
}

impl Handler<Rocket> for WsChatServer {
    type Result = ();

    fn handle(&mut self, rocket: Rocket, _ctx: &mut Self::Context) {
        if self.game_state.num_players() == 0 {
            println!("BEGINING TO RUN GAME\n\n\n");
            self.game_state.build();
            // self.run_game();
        }else {
            println!("Num players = {}\n\n\n", self.game_state.num_players());
        }
        println!("server.rs: rocket");
        self.game_state.rockets.entry(rocket.id).or_insert(Rocket {id:rocket.id, x:rocket.x, y:rocket.y, width:rocket.width, height:rocket.height}).update(rocket.x, rocket.y);
        println!("Updated rocket {} to ({}, {})", rocket.id, rocket.x, rocket.y);
    }
}

impl Handler<GetGame> for WsChatServer {
    type Result = MessageResult<GetGame>;

    fn handle(&mut self, _state: GetGame, _ctx: &mut Self::Context) -> Self::Result{
        println!("server.rs: getgame");
        println!("GetGame: {}", self.game_state.to_json_string());
        match self.write_state(self.game_state.to_json_string()) {
            Ok(ok) => println!("Succesfully wrote the data"),
            Err(e) => println!("Error writing state: {}", e)
        };
        MessageResult(self.game_state.to_json_string())
    }
}

impl SystemService for WsChatServer {}
impl Supervised for WsChatServer {}
