// lightspeed graphics and user controls

function build_stars() {
  stars = [];
  for (let i = 0; i < 50; i++) {
    let diameter = random(CANVAS_SIZE * 0.002, CANVAS_SIZE * 0.005);
    stars[i] = [random(0, CANVAS_SIZE),random(0, CANVAS_SIZE), diameter];
  }
  return stars;
}
const TITLE_SCREEN = 0;
const PLAY_SCREEN = 1;
const END_SCREEN = 2;
const ROCKET_COLORS = [
  [20,40,180],//Blue
  [0,130,30],//Green
  [160,160,30],//Yellow
  [160,10,160]//Purple
];

let CANVAS_SIZE;
let LOADING_ROCKET_HEIGHT;
let LOADING_ROCKET_WIDTH;

let ROCKET_HEIGHT;
let ROCKET_WIDTH;
let ROCKET_SPEED;
let rockets = {};

let browserId;
let socket;

setup = function() {
  browserId = localStorage.browserId || Math.floor(Math.random() * 10000);;
  localStorage.browserId = browserId;
  console.log('browserId:' + browserId)
  socket = null;

  keys = 1;
  CANVAS_SIZE = Math.min(window.innerWidth, window.innerHeight) - 30;
  LOADING_ROCKET_HEIGHT = CANVAS_SIZE/5;
  LOADING_ROCKET_WIDTH = LOADING_ROCKET_HEIGHT*(50 / 175);

  ROCKET_HEIGHT = 0.085 * CANVAS_SIZE;
  ROCKET_WIDTH = ROCKET_HEIGHT*(50 / 175);
  ROCKET_SPEED = 0.015 * CANVAS_SIZE;
  rockets[browserId]= {'x':CANVAS_SIZE/2, 'y':CANVAS_SIZE*(3/4)};
  
  createCanvas(CANVAS_SIZE, CANVAS_SIZE);
  background(0, 0, 0);
  explosion_size = 0.025 * CANVAS_SIZE;
  screen = TITLE_SCREEN;
  display_data = initial_display_data();  
  shots = [];
  shot = function(x, y) 
  {
    this.x = x;
    this.y = y;
  };

  keyPressed = function() {
    if (!game_started) 
    {
      game_started = true;
      screen = PLAY_SCREEN;
      score = 0;
    }
    if (keyCode === 32 || keyCode == 191) {addShot();}
    if (keyCode === 37) {setDirection(3);}
    if (keyCode === 38) {setDirection(0);}
    if (keyCode === 39) {setDirection(1);}
    if (keyCode === 40) {setDirection(2);}
  };
};


let screen;

let end_time = 0;
let score = 0;
let game_started = false;
let num_players = 1;
let direction = 0;
let explosion = false, explosion_size;
let can_restart = true;
let shots, shot, space_objects = [];
let big_flame = true;
let stars, moving = false;
let losing_rocket_id;

let display_data;

function initial_display_data() {
  return {
    'screen':TITLE_SCREEN,
    'end_time': 0,
    'explosion':false,
    'explosion_size':0,
    'can_restart':true,
    'big_flame':true,
    'stars':build_stars()
  };

}

function connect() {
    disconnect()

    const { location } = window
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/ws/`

    socket = new WebSocket(wsUri)
    socket.onopen = () => {
        console.log('Connected to ' + wsUri)
        const $id_element = document.querySelector('#id')
        $id_element.innerHTML = 'id=' + browserId + " "
        socket.send(`/connection {"browser_id":${browserId}, "x":${CANVAS_SIZE}, "y":${CANVAS_SIZE}}`)
    }

    socket.onmessage = (event) => {
        let game_state = JSON.parse(event.data);
        score = game_state["score"];
        space_objects = game_state["asteroids"];
        for (let i =0; i < space_objects.length; i++) {
          space_objects[i].x = space_objects[i].x*width;
          space_objects[i].y = space_objects[i].y*height;

          space_objects[i].diameter = space_objects[i].diameter*width;
        }
        updated_rockets = game_state["rockets"];
        for (let id in updated_rockets) {
          if (!(id === browserId || id == localStorage.browserId)) {
            if (!(id in rockets)) {
              rockets[id] = {'x':0,'y':0}
            }
            rockets[id].x = updated_rockets[id].x*width;
            rockets[id].y = updated_rockets[id].y*height;
          }
        }
        shots = game_state["shots"];
        if(game_state["screen"] == 2){
          explosion = true;
          screen = END_SCREEN;
          losing_rocket_id = game_state['losing_rocket_id'];
          end_time = millis();
        }else if(explosion){
          if(game_state["screen"] == 1){
            restart();
          }
        }
    }

    socket.onclose = () => {
        socket.send(`/disconnect {"browser_id":${browserId}, "x":${0}, "y":${0}}`)
        const $id_element = document.querySelector('#id')
        $id_element.innerHTML = 'disconnected  '
        socket = null
    }
}

function disconnect() {
    if (socket) {
        socket.close()
        socket = null
    }
}

window.onbeforeunload = function(){
  disconnect();
}

function setDirection(dir){
    direction = dir;
    moving = true;
}

const CIRCLES = 50;
const SHADES = 180/CIRCLES;

function fillForObj(ratio, i) {
  if(ratio > 1/7){
    fill(205 + (i * i * SHADES) / 2, 205 + (i * SHADES) / 2, i * SHADES);
  }else if(ratio > 1/9){
    fill(i * SHADES, i * SHADES, 245);
  }else {
    fill(i * SHADES);
  }
}
function drawSpaceObj(x, y, size) {
  const steps = size / CIRCLES;
  let ratio = (size/1.8)/width;
  for (let i = 20; i < CIRCLES; i++) {
    fillForObj(ratio, i);
    ellipse(x, y, size - i * steps, size - i * steps);
  }
}

//Draws all asteroids at their updated positions. When destroyed they will be shot out of view
let drawObjects = function() {
  fill(65, 72, 163);
  for (let i = 0; i < space_objects.length; i++) {
    drawSpaceObj(space_objects[i].x, space_objects[i].y, space_objects[i].diameter * 1.8);
  }
};

function mouseClicked() {
  addShot();
}

let update_stars = function() {
    for (let i = 0; i < stars.length; i++) {
        stars[i][1] += 1;
        if(stars[i][1] > height){
            stars[i][1] = 0;
        }
      }
}
// Draws rocket at specified location
let drawColoredRocket = function(id, x, y, r, g, b) {
  noStroke();
  let rocket_width, rocket_height;
  if(screen == TITLE_SCREEN){
    rocket_height = LOADING_ROCKET_HEIGHT;
    rocket_width = LOADING_ROCKET_WIDTH;
  }else {
    rocket_height = ROCKET_HEIGHT;
    rocket_width = ROCKET_WIDTH;
  }
  //shaded rectangle
  fill(220, 220, 220);
  rect(x, y, rocket_width, rocket_height);
  fill(200, 200, 200);
  rect(x, y, rocket_width / 6, rocket_height);
  fill(180, 180, 180);
  rect(x + rocket_width / 3, y, (rocket_width * 2) / 3, rocket_height);
  fill(130, 130, 130);
  rect(x + (rocket_width * 2) / 3, y, (rocket_width * 1) / 3, rocket_height);
  //wings and tip
  fill(r, g, b);
  triangle(x,y,x + rocket_width / 2,y - rocket_height * (7 / 16),x + rocket_width,y);
  ellipse(x + rocket_width / 2, y + rocket_height * 0.01, rocket_width, rocket_height * 0.2);
  fill(r + 20, g + 20, b + 20);
  triangle(x,y + rocket_height,x - rocket_width / 2,y + rocket_height,x,y + rocket_height / 4);
  fill(r, g, b);
  triangle(x + (3 / 2) * rocket_width,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width,y + rocket_height / 4);
  triangle(x + rocket_width / 3,y + rocket_height,x + rocket_width * (2 / 3),y + rocket_height,x + rocket_width / 2,y + rocket_height / 4);
  //flame
  if (id === browserId || id == localStorage.browserId) {
    if (score % 10) {
      big_flame = !big_flame;
    }
    if (big_flame) {
      fill(255, 100, 10);
      triangle(x,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width / 2,y + rocket_height * (9 / 6));
      fill(210, 230, 0);
      triangle(x + rocket_width / 4,y + rocket_height,x + rocket_width - rocket_width / 4,y + rocket_height,x + rocket_width / 2,y + rocket_height * (8 / 6));
    } else {
      fill(255, 100, 10);
      triangle(x,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width / 2,y + rocket_height * (11 / 6));
      fill(210, 230, 0);
      triangle(x + rocket_width / 4,y + rocket_height,x + rocket_width - rocket_width / 4,y + rocket_height,x + rocket_width / 2,y + rocket_height * (10 / 6));
    }
  }else {
    fill(190, 255, 190);
    triangle(x,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width / 2,y + rocket_height * (11 / 6));
    fill(255, 255, 255);
    triangle(x + rocket_width / 4,y + rocket_height,x + rocket_width - rocket_width / 4,y + rocket_height,x + rocket_width / 2,y + rocket_height * (10 / 6));
  }
};

let drawRockets = function() {
    let color_count = 0;
    for (let key in rockets) {
        // check if the property/key is defined in the object itself, not in paren
        if (rockets.hasOwnProperty(key)) { 
            if (key === browserId || key == localStorage.browserId) {
              console.log("rocket at " + rockets[key].x)
              drawColoredRocket(key, rockets[key].x, rockets[key].y, 160, 1, 70);
              color_count +=1;
            }else {
              let idx = color_count % 5;
              drawColoredRocket(key, rockets[key].x, rockets[key].y, 
                ROCKET_COLORS[idx][0], ROCKET_COLORS[idx][1], ROCKET_COLORS[idx][2]);
              color_count +=1;
            }
        }
    }
    if (num_players != color_count) {
        num_players = color_count;
        document.getElementById("num_players").innerHTML = num_players;
    }
}

let addShot = function() {
  let x = round(rockets[browserId].x + ROCKET_WIDTH / 2) - 3;
  let y = round(rockets[browserId].y - ROCKET_HEIGHT * (1 / 2));
  let b = new shot(x, y);
  shots.push(b);

  socket.send(`/shot {"browser_id":${browserId},"x":${x/width}, "y":${y/height}}`)
};

let drawShots = function() {
  fill(255, 238, 0);
  for (let i = 0; i < shots.length; i++) {
    rect(shots[i].x*width, shots[i].y*height, 6, 20);
  }
};


let drawEndingScreen = function() {
  fill(255, 180, 0);
  textAlign(CENTER);
  textSize(0.08 * width);
  fill(0, 0, 0);
  text(
    "Game over",
    width / 2,
    Math.round(0.3 * height));
    textSize(0.05 * width);
  text("Score " + score, width / 2, Math.round(0.4 * height));
};
/**
 * updateRocket
 *
 * Depending on the direction it will increase the speed
 */
let updateRocket = function() {
  if (keyIsPressed) {
    if (moving) {
      if (direction === 0) {
        rockets[browserId].y -= ROCKET_SPEED;
      } else if (direction === 1) {
        rockets[browserId].x += ROCKET_SPEED;
      } else if (direction === 2) {
        rockets[browserId].y += ROCKET_SPEED;
      } else if (direction === 3) {
        rockets[browserId].x -= ROCKET_SPEED;
      } else {
      }
    }
  } else {
    moving = false;
  }

  if (rockets[browserId].x < -1 * ROCKET_WIDTH) {
    rockets[browserId].x = width;
  } else if (rockets[browserId].x >= width + ROCKET_WIDTH) {
    rockets[browserId].x = -1 * ROCKET_WIDTH;
  }

  if (rockets[browserId].y + ROCKET_HEIGHT > height) {
    rockets[browserId].y = height - ROCKET_HEIGHT;
  } else if (rockets[browserId].y < 0) {
    rockets[browserId].y = 0;
  }
  if (game_started) {
    let x = rockets[browserId].x/width;
    let y = rockets[browserId].y/height;
     socket.send(`/rocket {"browser_id":${browserId}, "x":${x}, "y":${y}}`)
  }
};

let restart = function() {
  socket.send(`/play`)
  explosion_size = 10;
  explosion = false;
  game_started = true;
  screen = PLAY_SCREEN;
  score = 0;
  shots.splice(0);
};

let drawBackground = function() {
    background(0, 0, 0);
    for (let i = 0; i < stars.length; i++) {
        fill(255, 245, 182);
        ellipse(stars[i][0], stars[i][1], stars[i][2]);
    }
}
//Main method of the program
draw = function() {
  drawBackground();
  if (screen == TITLE_SCREEN) {
        textAlign(CENTER);
        fill(255, 255, 100);
        textSize(0.1075 * width);
        textFont("Ubuntu");
        text("LIGHTSPEED", 0.5 * width, 0.3 * height);
        textSize(0.05 * width);
        text("Press any button to begin", 0.5 * width, 0.4 * height);
        drawColoredRocket(browserId, rockets[browserId]['x'], rockets[browserId]['y'],160,1,70);
  } else if (screen == PLAY_SCREEN) {
        socket.send(`/state`);
        can_restart = false;

        for (let i = 0; i < space_objects.length; i++) {
          space_objects[i].y += space_objects[i].speed*height;
        }
        for (let i = 0; i < shots.length; i++) {
          shots[i].y -= 0.015;
        }

        updateRocket();
        drawShots();
        drawRockets();
        drawObjects();
        update_stars();
        textSize(width * 0.03);
        fill(255, 0, 0);
        text(score, 0.9 * width, 0.08 * height);
    } else if (explosion_size < CANVAS_SIZE * 2) {
        drawObjects();
        fill(255, 180, 0);
        explosion_size += 0.025 * width;
        if(losing_rocket_id in rockets) {
          ellipse(rockets[losing_rocket_id].x, rockets[losing_rocket_id].y, explosion_size, explosion_size);
        }
        if (explosion_size > width / 2) {
        drawEndingScreen();
    }
    if (millis() - end_time > 1500) {
        can_restart = true;
    }
    if (keyIsPressed && can_restart) {
        socket.send(`/play`);
    }
  } else if(screen == END_SCREEN) {
        socket.send(`/state`);
        background(255, 180, 0);
        drawEndingScreen();
        if (keyIsPressed) {
            socket.send(`/play`);
        }
        fill(0,0,0);
  }else {
    console.log("Error, no such screen");
  }
};

