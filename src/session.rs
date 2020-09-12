use log::{debug, info};

use actix::fut;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};

// mod lightspeed;
// use std::rc::Rc;
use std::fs::File;
use std::io::prelude::*;

use crate::message::{ChatMessage, JoinRoom, LeaveRoom, ListRooms, SendMessage, GetGame};
use crate::server::WsChatServer;
use crate::lightspeed::{GameState, Rocket};


#[derive(Default)]
pub struct WsChatSession {
    id: usize,
    room: String,
    name: Option<String>,
    game_state: GameState,
    data:String

}

impl WsChatSession {
    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        println!("JOIN ROOM");
        let room_name = room_name.to_owned();

        // First send a leave message for the current room
        let leave_msg = LeaveRoom(self.room.clone(), self.id);

        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_msg, ctx);

        // Then send a join message for the new room
        let join_msg = JoinRoom(
            room_name.to_owned(),
            self.name.clone(),
            ctx.address().recipient(),
        );

        WsChatServer::from_registry()
            .send(join_msg)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.id = id;
                    act.room = room_name;
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    pub fn list_rooms(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        WsChatServer::from_registry()
            .send(ListRooms)
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Ok(rooms) = res {
                    for room in rooms {
                        ctx.text(room);
                    }
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    // pub fn get_game_state(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> String {
    //     // let mut data = Vec::new();
    //     // data.push("".to_string());
    //     WsChatServer::from_registry().send(GetGame("".to_string())).into_actor(self).then(|res, _, ctx| {
    //         if let Ok(state) = res {
    //             println!("session.rs: state {}", state);
    //             self.data = state.clone();
    //         }
    //         fut::ready(())
    //     }).wait(ctx);
    //     // let state = data.pop().unwrap();
    //     return self.data
    // }

    pub fn read_state(&mut self) -> std::io::Result<()>{
        let mut file = File::open("state.txt")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        println!("state contents: {}", contents);
        self.send_data(contents);
        Ok(())
    }


    pub fn send_msg(&self, msg: &str) {
        let content = format!(
            "{}: {}",
            self.name.clone().unwrap_or_else(|| "anon".to_string()),
            msg
        );

        let msg = SendMessage(self.room.clone(), self.id, content);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(msg);
    }

    pub fn send_data(&self, data: String) {
        let msg = SendMessage(self.room.clone(), self.id, data);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(msg);
    }

    pub fn update_game(&mut self) {
        
        while self.game_state.is_playing() {
            self.game_state.update();
        }
    }
}
   

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.join_room("Main", ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "WsChatSession closed for {}({}) in room {}",
            self.name.clone().unwrap_or_else(|| "anon".to_string()),
            self.id,
            self.room
        );
    }
}

impl Handler<ChatMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

#[derive(Serialize, Deserialize)]
struct LightspeedConnection {
    browser_id:usize,
    width:i32,
    height:i32
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        debug!("WEBSOCKET MESSAGE: {:?}", msg);
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Text(text) => {
                let msg = text.trim();

                println!("msg text.trim: {:?}", msg);

                if msg.starts_with('/') {
                    let mut command = msg.splitn(2, ' ');

                    match command.next() {
                        Some("/list") => self.list_rooms(ctx),

                        Some("/join") => {
                            if let Some(room_name) = command.next() {
                                self.join_room(room_name, ctx);
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }

                        Some("/name") => {
                            if let Some(name) = command.next() {
                                self.name = Some(name.to_owned());
                                ctx.text(format!("name changed to: {}", name));
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }

                        Some("/connection") => {
                            println!("connection to lightspeed");
                            if let Some(browser_id) = command.next() {
                                println!("{}", browser_id);
                                let connection: LightspeedConnection = match serde_json::from_str(browser_id) {
                                    Ok(connection) => connection,
                                    Err(e) => {
                                        println!("Error: could not parse connection {}", e);
                                        return;
                                    }
                                };
                                //TODO: Only create game state if it hasn't been built already
                                self.id = connection.browser_id;
                                self.game_state.build();
                                let rocket = Rocket {
                                    id:self.id,
                                    x:connection.width/2,
                                    y:connection.height*3/4,

                                    width:connection.width,
                                    height:connection.height
                                };
                                WsChatServer::from_registry().send(rocket).into_actor(self).then(|res, _, ctx| {
                    
                                    fut::ready(())
                                }).wait(ctx);
                                // self.game_state.add_player(self.id, connection.width, connection.height);
                                // self.game_state.print_state();
                                
                                // let mut data:String = self.get_game_state(ctx);
                                WsChatServer::from_registry().send(GetGame("".to_string())).into_actor(self).then(move |res, _, _ctx| {
                                    if let Ok(state) = res {
                                        println!("session.rs: state {}", state);
                                    }
                                    fut::ready(())
                                }).wait(ctx);           

                                match self.read_state() {
                                    Ok(ok) => println!("Succesfully read the data"),
                                    Err(e) => println!("Error reading state: {}", e)
                                };
                                println!("sending data to client");
                            } else {
                                println!("Connection needs id!")
                            }
                        }

                        _ => ctx.text(format!("!!! unknown command: {:?}", msg)),
                    }
                    return;
                }
                
                self.send_msg(msg);
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
