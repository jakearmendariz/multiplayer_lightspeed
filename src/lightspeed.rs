use rand::Rng;
use rand_distr::{Normal, Distribution};
use std::cmp;

use std::collections::HashMap;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

const MAX_ASTEROID:i32 = 180;
const MIN_ASTEROID:i32 = 80;

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Rocket {
    pub x:f32,
    pub y:f32,
}

impl Rocket {
    pub fn update(&mut self, _x:f32, _y:f32){
        self.x = _x;
        self.y = _y;
    }

    pub fn reset(&mut self){
        self.x = 0.5;
        self.y = 0.75;
    }
}

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Shot {
    pub x:f32,
    pub y:f32
}

impl Shot {
    fn update(&mut self) {
        self.y -= 0.015;
    }
}

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Asteroid {
    pub health:u8, //Health refers to how many hits it can take. Asteroids start at 1, planets at 2, suns at 3
    x:f32,
    y:f32,
    diameter:f32,
    speed:f32
}

impl Asteroid {
    fn update(&mut self) {
        self.y += self.speed;
        if self.y > 1.0 {
            self.initialize();
        }
    }

    fn initialize(&mut self){
        let mut rng = rand::thread_rng();
        self.x = rng.gen_range(0.0..1.0);
        self.y = rng.gen_range(-0.5..0.0);
        self.speed = rng.gen_range(0.006..0.013);
        self.set_diameter();
    }

    //health => number of hits before exploding
    fn assign_health(&mut self) {
        if self.diameter > 1.0/7.0 {
            self.health = 3;
        }else if self.diameter > 1.0/9.0 {
            self.health = 2;
        }else {
            self.health = 1;
        }
    }

    fn set_diameter(&mut self) {
        //(mean, standard_deviation)
        let normal = Normal::new(0.075, 0.033).unwrap();
        self.diameter = normal.sample(&mut rand::thread_rng()) as f32;
        let temp:i32 = (self.diameter*1000.0).round() as i32;
        //Keep within a range
        self.diameter = (cmp::max(MIN_ASTEROID, temp) as f32)/1000.0;
        self.diameter = (cmp::min(MAX_ASTEROID, temp) as f32)/1000.0;
        self.assign_health();
    }
}

pub const TITLE:u8 = 0;
pub const PLAY:u8 = 1;
pub const END:u8 = 2;

#[derive(Copy, Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Screen {
    pub width:f32,
    pub height:f32,
}

//Controls the entire game, will be sent to each of the clients, for them to display
#[derive(Clone, Message, Default, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct GameState {
    pub score:u32,
    pub user_count:u8,
    pub rockets:HashMap<usize, Rocket>,
    pub _screens:HashMap<usize, Screen>,
    pub shots:Vec<Shot>,
    pub asteroids:Vec<Asteroid>,
    pub screen:u8,
    pub losing_rocket_id:usize
}
//collisions
impl GameState {

    pub fn update(&mut self){
        if self.screen == PLAY {
            self.score += 1;
            for i in 0..self.asteroids.len() {
                self.asteroids[i].update();
            }
            for i in 0..self.shots.len() {
                self.shots[i].update();
            }
            if self.score % 500 == 0 {
                let mut asteroid:Asteroid = Asteroid::default();
                asteroid.initialize();
                self.asteroids.push(asteroid);
            }
            // if self.score % 3 == 0 {
            //     self.collisions();
            // }
            self.collisions();
        }
    }
    //Finds the distance between two points. (For object collision)
    fn distance(&mut self, x1:f32,y1:f32,x2:f32,y2:f32) -> f32{
        return (((x2-x1)*(x2-x1) + (y2-y1)*(y2-y1)) as f64).sqrt() as f32;
    }

    fn collisions(&mut self){
        for i in 0..self.asteroids.len() {
            let mut delete_index = vec!();
            for j in 0..self.shots.len() {
                if self.distance(self.shots[j].x, self.shots[j].y, self.asteroids[i].x, self.asteroids[i].y) <= self.asteroids[i].diameter/2.0 {
                    if self.asteroids[i].health > 1 {
                        self.asteroids[i].health -= 1;
                    }else {
                        self.asteroids[i].initialize();
                    }
                    self.score += 20;
                    delete_index.push(j);
                }else if self.shots[j].y < -0.05 {
                    delete_index.push(j);
                }
            } 
            //Deletes in reverse order as to not fuck up the indexes
            let mut idx = delete_index.len();
            while idx > 0 {
                idx -= 1;
                self.shots.remove(delete_index[idx]);
            }
            
            for (id, rocket) in self.rockets.iter() {
                //Collisions
                let rocket_width:f32 = 1.0/17.5;
                if (self.asteroids[i].x - rocket.x).abs() < self.asteroids[i].diameter/2.0 && (self.asteroids[i].y - rocket.y).abs() < self.asteroids[i].diameter/2.0 
                  || (self.asteroids[i].x - (rocket.x + rocket_width)).abs() < self.asteroids[i].diameter/2.0 && (self.asteroids[i].y - rocket.y).abs() < self.asteroids[i].diameter/2.0{
                    self.screen = END;
                    self.losing_rocket_id = *id;
                }
            }
        }
        if self.screen == END {
            self.clear_game();
        }
    }

    fn clear_game(&mut self){
        self.asteroids = Vec::new();
        self.shots = Vec::new();
        self.score = 0;
    }
    pub fn build(&mut self) {  
        //creates 7 asteroids above the map to begin with
        self.asteroids = Vec::new();
        self.shots = Vec::new();
        self.score = 0;
        for _ in 0..7 {
            let mut asteroid:Asteroid = Asteroid::default();
            asteroid.initialize();
            self.asteroids.push(asteroid);
        }
        for (_id, rocket) in self.rockets.iter_mut() {
            rocket.reset();
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
        println!("Rockets: {}, Shots count: {}, Asteroids count: {}", self.rockets.len(), self.shots.len(), self.asteroids.len());
    }

    pub fn _is_playing(&self) -> bool {
        return self.screen == 1 && self.rockets.len() > 0;
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

