const browserId = localStorage.browserId || Math.floor(Math.random() * 10000);;
localStorage.browserId = browserId;
console.log('browserId:' + browserId)

var canvas_size = Math.min([window.innerWidth, window.innerHeight]);
var socket = null

var end_time = 0;
var score = 0;
var game_started = false;
var rockets = {}, num_players = 1;
var x_pos, y_pos, rocket_height, rocket_width;
var direction = 0;
var explosion = false, explosion_size;
var rocket_speed;
var can_restart = true;
var shots, shot, space_objects = [];
var big_flame = true;
var stars, moving = false;

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
        canvas_size = Math.min(window.innerWidth, window.innerHeight);
        socket.send(`/connection {"browser_id":${browserId}, "x":${canvas_size}, "y":${canvas_size}}`)
    }

    socket.onmessage = (event) => {
        var game_state = JSON.parse(event.data);
        score = game_state["score"];
        space_objects = game_state["asteroids"];
        for (let i =0; i < space_objects.length; i++) {
          space_objects[i].x = space_objects[i].x*width;
          space_objects[i].y = space_objects[i].y*height;

          space_objects[i].diameter = space_objects[i].diameter*width;
        }
        rockets = game_state["rockets"];
        for (let i =0; i < rockets.length; i++) {
          rockets[i].x = rockets[i].x*width;
          rockets[i].y = rockets[i].y*height;
        }
        shots = game_state["shots"];
        if(game_state["screen"] == 2){
          explosion = true;
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

setup = function() {
  keys = 1;
  let canvas_size = Math.min(window.innerWidth, window.innerHeight) - 30;
  createCanvas(canvas_size, canvas_size);
  background(0, 0, 0);
  explosion_size = 0.025 * canvas_size;
  rocket_height = 0.2 * canvas_size;
  rocket_width = rocket_height * (50 / 175);
  x_pos = 0.5 * canvas_size - rocket_width / 2;
  y_pos = 0.8 * canvas_size - rocket_height;
  rocket_speed = 0.015 * canvas_size;
  //Rocket Shots
  stars = [];
  for (var i = 0; i < 50; i++) {
    let diameter = random(canvas_size * 0.002, canvas_size * 0.005);
    stars[i] = [random(0, canvas_size),random(0, canvas_size), diameter];
  }
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
      rocket_height = 0.085 * canvas_size;
      rocket_width = rocket_height * (50 / 175);
      x_pos = 0.5 * width - rocket_width / 2;
      y_pos = 0.9 * height - rocket_height;
      score = 0;
    }
    if (keyCode === 32 || keyCode == 191) {addShot();}
    if (keyCode === 37) {setDirection(3);}
    if (keyCode === 38) {setDirection(0);}
    if (keyCode === 39) {setDirection(1);}
    if (keyCode === 40) {setDirection(2);}
  };
};

function setDirection(dir){
    direction = dir;
    moving = true;
}

function drawAsteroid(x, y, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(i * grayvalues);
    ellipse(x, y, size - i * steps, size - i * steps);
  }
}

function drawPlanet(x, y, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(i * grayvalues, i * grayvalues, 245);
    ellipse(x, y, size - i * steps, size - i * steps);
  }
}

function drawSun(x, y, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(205 + (i * i * grayvalues) / 2, 205 + (i * grayvalues) / 2, i * grayvalues);
    ellipse(x, y, size - i * steps, size - i * steps);
  }
}

function mouseClicked() {
  addShot();
}

// Draws rocket at specified location
var drawRocket = function(x, y) {
  noStroke();
  if (score % 10) {
    big_flame = !big_flame;
  }
  // x*=width;
  // y*=height;

  fill(220, 220, 220);
  rect(x, y, rocket_width, rocket_height);
  fill(200, 200, 200);
  rect(x, y, rocket_width / 6, rocket_height);
  fill(180, 180, 180);
  rect(x + rocket_width / 3, y, (rocket_width * 2) / 3, rocket_height);
  fill(130, 130, 130);
  rect(x + (rocket_width * 2) / 3, y, (rocket_width * 1) / 3, rocket_height);

  fill(160, 1, 70);
  triangle(x,y,x + rocket_width / 2,y - rocket_height * (7 / 16),x + rocket_width,y);
  ellipse(x + rocket_width / 2, y + rocket_height * 0.01, rocket_width, rocket_height * 0.2);
  fill(160 + 20, 1 + 20, 70 + 20);
  triangle(x,y + rocket_height,x - rocket_width / 2,y + rocket_height,x,y + rocket_height / 4);
  fill(160, 1, 70);
  triangle(x + (3 / 2) * rocket_width,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width,y + rocket_height / 4);

  triangle(x + rocket_width / 3,y + rocket_height,x + rocket_width * (2 / 3),y + rocket_height,x + rocket_width / 2,y + rocket_height / 4);

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
};


var update_stars = function() {
    for (var i = 0; i < stars.length; i++) {
        stars[i][1] += 1;
        if(stars[i][1] > height){
            stars[i][1] = 0;
        }
      }
}
// Draws rocket at specified location
var drawColoredRocket = function(x, y, r, g, b) {
  noStroke();
  if (score % 10) {
    big_flame = !big_flame;
  }

  fill(220, 220, 220);
  rect(x, y, rocket_width, rocket_height);
  fill(200, 200, 200);
  rect(x, y, rocket_width / 6, rocket_height);
  fill(180, 180, 180);
  rect(x + rocket_width / 3, y, (rocket_width * 2) / 3, rocket_height);
  fill(130, 130, 130);
  rect(x + (rocket_width * 2) / 3, y, (rocket_width * 1) / 3, rocket_height);

  fill(r, g, b);
  triangle(x,y,x + rocket_width / 2,y - rocket_height * (7 / 16),x + rocket_width,y);
  ellipse(x + rocket_width / 2, y + rocket_height * 0.01, rocket_width, rocket_height * 0.2);
  fill(r + 20, g + 20, b + 20);
  triangle(x,y + rocket_height,x - rocket_width / 2,y + rocket_height,x,y + rocket_height / 4);
  fill(r, g, b);
  triangle(x + (3 / 2) * rocket_width,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width,y + rocket_height / 4);

  triangle(x + rocket_width / 3,y + rocket_height,x + rocket_width * (2 / 3),y + rocket_height,x + rocket_width / 2,y + rocket_height / 4);

  fill(190, 255, 190);
  triangle(x,y + rocket_height,x + rocket_width,y + rocket_height,x + rocket_width / 2,y + rocket_height * (11 / 6));
  fill(255, 255, 255);
  triangle(x + rocket_width / 4,y + rocket_height,x + rocket_width - rocket_width / 4,y + rocket_height,x + rocket_width / 2,y + rocket_height * (10 / 6));
};

var drawOtherRockets = function() {
    let colors = [
      [20,40,180],//Blue
      [0,130,30],//Green
      [160,160,30]//Yellow
      [160,10,160]//Purple
    ];
    let color_count = 0;
    for (var key in rockets) {
        // check if the property/key is defined in the object itself, not in paren
        if (rockets.hasOwnProperty(key)) {     
            let idx = color_count % 4;
            if(key !== browserId){
              drawColoredRocket(rockets[key].x*width, rockets[key].y*height, colors[idx][0],colors[idx][1],colors[idx][2] );
              color_count +=1;
            }
        }
    }
    if (num_players != color_count+1) {
        num_players = color_count+1;
        document.getElementById("num_players").innerHTML = num_players;
    }
}

var addShot = function() {
  let _x = round(x_pos + rocket_width / 2) - 3;
  let _y = round(y_pos - rocket_height * (1 / 2));
  var b = new shot(_x, _y);
  shots.push(b);

  socket.send(`/shot {"browser_id":${browserId},"x":${_x/width}, "y":${_y/height}}`)
};

var displayshots = function() {
  fill(255, 238, 0);
  for (var i = 0; i < shots.length; i++) {
    rect(shots[i].x*width, shots[i].y*height, 6, 20);
  }
};

//Draws all asteroids at their updated positions. When destroyed they will be shot out of view
var drawObjects = function() {
  fill(65, 72, 163);
  for (var i = 0; i < space_objects.length; i++) {
    if (space_objects[i].diameter > 900/7) {
      fill(255, 238, 0);
      drawSun(
        space_objects[i].x,
        space_objects[i].y,
        space_objects[i].diameter * 1.7,
        50
      );
    }else if (space_objects[i].diameter > 900/9) {
      fill(65, 72, 163);
      drawPlanet(
        space_objects[i].x,
        space_objects[i].y,
        space_objects[i].diameter * 1.8,
        50
      );
    } else {
        fill(140, 138, 140);
        drawAsteroid(
          space_objects[i].x,
          space_objects[i].y,
          space_objects[i].diameter * 1.8,
          50
        );
    }
  }
};

var drawEndingScreen = function() {
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
var updateRocket = function() {
  if (keyIsPressed) {
    if (moving) {
      if (direction === 0) {
        y_pos -= rocket_speed;
      } else if (direction === 1) {
        x_pos += rocket_speed;
      } else if (direction === 2) {
        y_pos += rocket_speed;
      } else if (direction === 3) {
        x_pos -= rocket_speed;
      } else {
      }
    }
  } else {
    moving = false;
  }

  if (x_pos < -1 * rocket_width) {
    x_pos = width;
  } else if (x_pos >= width + rocket_width) {
    x_pos = -1 * rocket_width;
  }

  if (y_pos + rocket_height > height) {
    y_pos = height - rocket_height;
  } else if (y_pos < 0) {
    y_pos = 0;
  }
  if (game_started) {
     socket.send(`/rocket {"browser_id":${browserId}, "x":${x_pos/width}, "y":${y_pos/height}}`)
  }
};

var restart = function() {
  socket.send(`/play`)
  explosion_size = 10;
  explosion = false;
  game_started = true;
  score = 0;
  x_pos = 0.4875 * width;
  y_pos = 0.805 * height;
  shots.splice(0);
};

var drawBackground = function() {
    background(0, 0, 0);
    for (var i = 0; i < stars.length; i++) {
        fill(255, 245, 182);
        ellipse(stars[i][0], stars[i][1], stars[i][2]);
    }
}
//Main method of the program
draw = function() {
  drawBackground();
  if (!game_started) {
        // score++;
        textAlign(CENTER);
        fill(255, 255, 100);
        textSize(0.1075 * width);
        textFont("Ubuntu");
        text("LIGHTSPEED", 0.5 * width, 0.3 * height);
        textSize(0.05 * width);
        text("Press any button to begin", 0.5 * width, 0.4 * height);

        drawRocket(x_pos, y_pos);
  } else if (!explosion) {
        socket.send(`/state`);
        can_restart = false;
        // score++;

        for (let i = 0; i < space_objects.length; i++) {
          space_objects[i].y += space_objects[i].speed*height;
        }
        for (let i = 0; i < shots.length; i++) {
          shots[i].y -= 0.015;
        }

        
        updateRocket();
        displayshots();
        drawRocket(x_pos, y_pos);
        drawOtherRockets();
        drawObjects();
        update_stars();
        textSize(width * 0.03);
        fill(255, 0, 0);
        text(score, 0.9 * width, 0.08 * height);
    } else if (explosion_size < canvas_size * 2) {
        drawRocket(x_pos, y_pos);
        drawObjects();
        fill(255, 180, 0);
        ellipse(x_pos, y_pos, explosion_size, explosion_size);
        explosion_size += 0.025 * width;
        if (explosion_size > width / 2) {
        drawEndingScreen();
    }
    if (millis() - end_time > 1500) {
        can_restart = true;
    }
    if (keyIsPressed && can_restart) {
        socket.send(`/play`);
    }
  } else {
        socket.send(`/state`);
        background(255, 180, 0);
        drawEndingScreen();
        if (keyIsPressed) {
            socket.send(`/play`);
        }
        fill(0,0,0);
  }
};

