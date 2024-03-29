use log::{info};

use actix::fut;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};

use crate::message::{ChatMessage, JoinRoom, LeaveRoom, ListRooms, SendMessage, GetGame, RemovePlayer, ResetGame};
use crate::server::WsChatServer;
use crate::lightspeed::{Shot};


#[derive(Default)]
pub struct WsChatSession {
    id: usize,
    room: String,
    name: Option<String>,
}

impl WsChatSession {
    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
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
}
   

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.join_room("Main", ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Lightspeed closed for {}({}) in room {}",
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

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct LightspeedConnection {
    pub browser_id:usize,
    pub x:f32,
    pub y:f32
}

#[derive(Serialize, Deserialize)]
struct PlayGame {
    browser_id:usize,
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
                            println!("/name");
                            if let Some(name) = command.next() {
                                self.name = Some(name.to_owned());
                                println!("saved user: {:?}", self.name);
                                ctx.text(format!("{{name:{}}}", name));
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }

                        Some("/rocket") => {
                            if let Some(browser_id) = command.next() {
                                let rocket_json: LightspeedConnection = match serde_json::from_str(browser_id) {
                                    Ok(connection) => connection,
                                    Err(e) => {
                                        println!("Error: could not parse /rocket connection {}", e);
                                        return;
                                    }
                                };
                                //Current session's rocket
                                let rocket = LightspeedConnection {
                                    browser_id:self.id,
                                    x:rocket_json.x,
                                    y:rocket_json.y,
                                };
                                //Sends updated rocket position
                                WsChatServer::from_registry().send(rocket).into_actor(self).then(|_res, _, _ctx| {
                                    fut::ready(())
                                }).wait(ctx);
                            }
                        }
                        Some("/shot") => {
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
                                WsChatServer::from_registry().send(shot).into_actor(self).then(|_res, _, _ctx| {
                                    fut::ready(())
                                }).wait(ctx);
                            }
                            
                        }
                        Some("/state") => {
                            WsChatServer::from_registry().send(GetGame("".to_string())).into_actor(self).then(move |res, _, ctx| {
                                if let Ok(state) = res {
                                    ctx.text(state)
                                }
                                fut::ready(())
                            }).wait(ctx);           
                        }
                        Some("/disconnect") => {
                            println!("Disconnect player to lightspeed");
                            WsChatServer::from_registry().send(RemovePlayer { id:self.id } ).into_actor(self).then(move |_res, _, _ctx| {
                                fut::ready(())
                            }).wait(ctx);       
                        }
                        Some("/play") => {
                            WsChatServer::from_registry().send(ResetGame()).into_actor(self).then(|_res, _, _ctx| {
                                fut::ready(())
                            }).wait(ctx);
                        }
                        Some("/connection") => {
                            println!("new connection to lightspeed");
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
                                //Current session's rocket
                                let rocket = LightspeedConnection {
                                    browser_id:self.id,
                                    x:1.0/2.0,
                                    y:3.0/4.0,
                                };
                                //Sends updated rocket position
                                WsChatServer::from_registry().send(rocket).into_actor(self).then(|_res, _, _ctx| {
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
                println!("disconnect player to lightspeed\n\n");
                WsChatServer::from_registry().send(RemovePlayer { id:self.id } ).into_actor(self).then(move |_res, _, _ctx| {
                    fut::ready(())
                }).wait(ctx);  
                ctx.close(reason); 
                ctx.stop();
            }
            _ => {}
        }
    }
}
