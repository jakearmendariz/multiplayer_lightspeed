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
use crate::lightspeed::{GameState, Rocket, Shot};


#[derive(Default)]
pub struct WsChatSession {
    id: usize,
    room: String,
    name: Option<String>,
    width:i32,
    height:i32

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
    x:i32,
    y:i32
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
        match msg {
            ws::Message::Text(text) => {
                let msg = text.trim();
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

                        Some("/rocket") => {
                            println!("Rocket");
                            if let Some(browser_id) = command.next() {
                                println!("{}", browser_id);
                                let rocket_json: LightspeedConnection = match serde_json::from_str(browser_id) {
                                    Ok(connection) => connection,
                                    Err(e) => {
                                        println!("Error: could not parse /rocket connection {}", e);
                                        return;
                                    }
                                };
                                //Current session's rocket
                                let rocket = Rocket {
                                    id:self.id,
                                    x:rocket_json.x,
                                    y:rocket_json.y,

                                    width:self.width,
                                    height:self.height
                                };
                                //Sends updated rocket position
                                WsChatServer::from_registry().send(rocket).into_actor(self).then(|res, _, ctx| {
                                    fut::ready(())
                                }).wait(ctx);
                            }
                        }
                        Some("/shot") => {
                            println!("shot fired! {}", msg);
                            if let Some(browser_id) = command.next() {
                                let shot_json: LightspeedConnection = match serde_json::from_str(browser_id) {
                                    Ok(connection) => connection,
                                    Err(e) => {
                                        println!("Error: could not parse /shot connection {}", e);
                                        return;
                                    }
                                };
                                //New shot fired
                                let shot = Shot {
                                    x:shot_json.x,
                                    y:shot_json.y,
                                };
                                //Sends updated rocket position
                                WsChatServer::from_registry().send(shot).into_actor(self).then(|res, _, ctx| {
                                    fut::ready(())
                                }).wait(ctx);
                            }
                            
                        }
                        Some("/state") => {
                            println!("/state");
                            WsChatServer::from_registry().send(GetGame("".to_string())).into_actor(self).then(move |res, _, ctx| {
                                if let Ok(state) = res {
                                    println!("session.rs: state {}", state);
                                    ctx.text(state)
                                }
                                fut::ready(())
                            }).wait(ctx);           

                            println!("sending state to clients");
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
                                self.id = connection.browser_id;
                                self.width = connection.x;
                                self.height = connection.y;
                                //Current session's rocket
                                let rocket = Rocket {
                                    id:self.id,
                                    x:connection.x/2,
                                    y:connection.y*3/4,

                                    width:self.width,
                                    height:self.height
                                };
                                //Sends updated rocket position
                                WsChatServer::from_registry().send(rocket).into_actor(self).then(|res, _, ctx| {
                                    fut::ready(())
                                }).wait(ctx);
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
