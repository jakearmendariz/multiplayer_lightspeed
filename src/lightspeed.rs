use rand::Rng;
use std::collections::HashMap;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Rocket {
    pub id:usize,
    pub x:i32,
    pub y:i32,

    pub width:i32,
    pub height:i32,
}

impl Rocket {
    pub fn update(&mut self, _x:i32, _y:i32){
        self.x = _x;
        self.y = _y;
    }
}

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Shot {
    x:i32,
    y:i32
}

impl Shot {
    fn update(&mut self) {
        self.y -= 1;
    }
}

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Asteroid {
    health:u8, //Health refers to how many hits it can take. Asteroids start at 1, planets at 2, suns at 3
    x:i32,
    y:i32,
    radius:i32,
    speed:i32
}

impl Asteroid {
    fn update(&mut self) {
        self.y += self.speed
    }
}

pub const TITLE:u8 = 0;
pub const PLAY:u8 = 1;
pub const END:u8 = 2;

pub const WIDTH:i32 = 900;
pub const HEIGHT:i32 = 900;

//Controls the entire game, will be sent to each of the clients, for them to display
#[derive(Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct GameState {
    pub score:u32,
    pub user_count:u8,
    pub rockets:HashMap<usize, Rocket>,
    pub shots:Vec<Shot>,
    pub asteroids:Vec<Asteroid>,
    pub screen:u8
}
//collisions
impl GameState {

    pub fn update(&mut self){
        for i in 0..self.asteroids.len() {
            self.asteroids[i].update();
        }
        for i in 0..self.shots.len() {
            self.shots[i].update();
        }
        self.collisions();
    }
    fn collisions(&mut self){
        let mut rng = rand::thread_rng();
        for i in 0..self.asteroids.len() {
            
            for j in 0..self.shots.len() {
                self.asteroids[i].x = rng.gen_range(0, WIDTH);
                self.asteroids[i].y = rng.gen_range(-300, 0);

                //Delete shot in the future, for now just move it high up
                //To delete create an array of destroyed indexes, than loop backwards deleting them from vector
                self.shots[j].y = -1*HEIGHT
            } 
            for (_id, rocket) in self.rockets.iter() {
                if (self.asteroids[i].x - rocket.x).abs() < self.asteroids[i].radius && (self.asteroids[i].y - rocket.y).abs() < self.asteroids[i].radius {
                    //Collision detected, return false for game is over
                    self.screen = END;
                    println!("END GAME")
                }
            }
        }
    }

    pub fn build(&mut self) {  
        let mut rng = rand::thread_rng();
        //creates 5 asteroids above the map to begin with
        for _ in 0..5 {
            let radius:i32 = rng.gen_range(WIDTH/30, WIDTH/10);
            let health;
            if radius > WIDTH/7 {
                health = 3;
            }else if radius > WIDTH/9 {
                health = 2;
            }else {
                health = 1;
            }
            self.asteroids.push(Asteroid {
                x:rng.gen_range(0, WIDTH),
                y:rng.gen_range(-1*HEIGHT, -1 *50),
                radius:radius,
                speed: rng.gen_range(1,4),
                health:health
            });
        }
        self.screen = PLAY;
    }

    pub fn add_player(&mut self, id:usize, width:i32, height:i32){
        let rocket:Rocket = Rocket {
            x:width/2,
            y:height*3/4,
            id:id,

            width:width,
            height:height
        };
        self.rockets.insert(id, rocket);
    }

    pub fn num_players(&self) -> usize{
        self.rockets.len()
    }

    fn shoot(&mut self, id:usize){
        let from_rocket:Rocket = match self.rockets.get(&id) {
            Some(&rocket) => rocket,
            _ => return
        };
        let shot = Shot {
            x:from_rocket.x,
            y:from_rocket.y
        };
        self.shots.push(shot);
    }

    pub fn print_state(&self) {
        println!("Rockets:");
        for (id, _rocket) in self.rockets.iter() {
            println!("{}", id);
        }
        println!("Shots count: {}", self.shots.len());
        println!("Asteroids count: {}", self.asteroids.len());
    }

    pub fn is_playing(&self) -> bool {
        return self.screen == 1;
    }

    pub fn to_json_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json_str) => json_str,
            Err(e) => {
                println!("Error while serializing game state to json: {}", e);
                "".to_string()
            }
        }
    }
}

