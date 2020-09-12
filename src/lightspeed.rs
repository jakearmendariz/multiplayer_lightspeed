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
    pub x:i32,
    pub y:i32
}

impl Shot {
    fn update(&mut self) {
        self.y -= 5;
    }
}

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Asteroid {
    pub health:u8, //Health refers to how many hits it can take. Asteroids start at 1, planets at 2, suns at 3
    x:i32,
    y:i32,
    radius:i32,
    speed:i32
}

impl Asteroid {
    fn update(&mut self) {
        self.y += self.speed;
        if self.y > 1000 {
            let mut rng = rand::thread_rng();
            self.x = rng.gen_range(0, WIDTH);
            self.y = rng.gen_range(-300, 0);
            self.radius = rng.gen_range(WIDTH/25, WIDTH/7);
        }
    }

    fn assign_health(&mut self) {
        if self.radius > WIDTH/7 {
            self.health = 3;
        }else if self.radius > WIDTH/9 {
            self.health = 2;
        }else {
            self.health = 1;
        }
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
        self.score += 1;
        for i in 0..self.asteroids.len() {
            self.asteroids[i].update();
        }
        for i in 0..self.shots.len() {
            self.shots[i].update();
        }
        if self.score % 10 == 0 {
            self.collisions();
        }
    }
    //Finds the distance between two points. (For object collision)
    fn distance(&mut self, x1:i32,y1:i32,x2:i32,y2:i32) -> i32{
        return (((x2-x1)*(x2-x1) + (y2-y1)*(y2-y1)) as f64).sqrt() as i32;
    }

    fn collisions(&mut self){
        let mut rng = rand::thread_rng();
        for i in 0..self.asteroids.len() {
            let mut delete_index = vec!();
            for j in 0..self.shots.len() {
                // if self.shots[j].x == self.asteroids[i].x && self.shots[j].y == self.asteroids[i].y {
                if self.distance(self.shots[j].x, self.shots[j].y, self.asteroids[i].x, self.asteroids[i].y) <= self.asteroids[i].radius/2 {
                    if self.asteroids[i].health > 1 {
                        self.asteroids[i].health -= 1;
                    }else {
                        self.asteroids[i].x = rng.gen_range(0, WIDTH);
                        self.asteroids[i].y = rng.gen_range(-300, 0);
                        self.asteroids[i].radius = rng.gen_range(WIDTH/25, WIDTH/7);
                        self.asteroids[i].assign_health();
                    }
                    println!("collision");
                    delete_index.push(j);
                }else if self.shots[j].y < -50 {
                    delete_index.push(j);
                }
            } 
            //Deletes in reverse order as to not fuck up the indexes
            let mut idx = delete_index.len();
            while idx > 0 {
                idx -= 1;
                self.shots.remove(delete_index[idx]);
            }
            for (_id, rocket) in self.rockets.iter() {
                if (self.asteroids[i].x - rocket.x).abs() < self.asteroids[i].radius && (self.asteroids[i].y - rocket.y).abs() < self.asteroids[i].radius {
                    //Collision detected, return false for game is over
                    self.screen = END;
                    // println!("END GAME")
                }
            }
        }
    }

    pub fn build(&mut self) {  
        let mut rng = rand::thread_rng();
        //creates 5 asteroids above the map to begin with
        for _ in 0..7 {
            let radius:i32 = rng.gen_range(WIDTH/25, WIDTH/7);
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
                speed: rng.gen_range(3,8),
                health:health
            });
        }
        self.screen = PLAY;
    }

    pub fn num_players(&self) -> usize{
        self.rockets.len()
    }

    fn _shoot(&mut self, id:usize){
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

    pub fn _print_state(&self) {
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

