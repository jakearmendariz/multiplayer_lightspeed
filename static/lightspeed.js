const browserId = localStorage.browserId || Math.floor(Math.random() * 10000);;
localStorage.browserId = browserId;
console.log('browserId:' + browserId)


var canvas_size = Math.min([window.innerWidth, window.innerHeight]);

var socket = null

var startTime = 0;
var counter = 0;
var gameStarted = false;

var rockets = {}, num_players = 1;
var xPos, yPos, aHeight, aWidth;
var direction = 0;
var explosion = false, explosionSize;
var astDestroyed = 0;
var hitCounter = [];
var highScore = 0;
//Increases difficulty
var speedInc, gameSpeed;
var canRestart = true;
var bullets, bullet, spaceObjects = [];
var bigFlame = true;
//View
var drawAsteroid, drawEndingScreen;
var starsX, starsY, keys, ave, moving = false;



function connect() {
    disconnect()

    const { location } = window

    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/ws/`

    // if (localStorage.hasOwnProperty("name")) {
    //     console.log("name:" + localStorage.name)
    //     const $input = document.querySelector('#name')
    //     $input.innerHTML = localStorage.name;
    // }

    socket = new WebSocket(wsUri)

    socket.onopen = () => {
        console.log('Connected to ' + wsUri)
        const $id_element = document.querySelector('#id')
        $id_element.innerHTML = 'id=' + browserId + " "
        canvas_size = Math.min(window.innerWidth, window.innerHeight);
        socket.send(`/connection {"browser_id":${browserId}, "x":${canvas_size}, "y":${canvas_size}}`)
    }

    socket.onmessage = (event) => {
        // console.log("event data: " + event.data)
        var game_state = JSON.parse(event.data);
        counter = game_state["score"];
        spaceObjects = game_state["asteroids"];
        rockets = game_state["rockets"];
        bullets = game_state["shots"];
        if(game_state["screen"] == 2){
          explosion = true;
          startTime = millis();
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

        updateConnectionStatus()
    }
}

window.onbeforeunload = function(){
  disconnect();
}

// const $form = document.querySelector('#nameform')
// const $input = document.querySelector('#name')

// function save_name() {
//     const name = document.getElementById("name").value;

//     localStorage.name = name 
//     console.log("saving " + name + " to local storage");
//     socket.send(`/name ${localStorage.name}`)
// }

//Where to add images and fonts
function preload() {}
setup = function() {
  keys = 1;
  let canvas_size = Math.min(window.innerWidth, window.innerHeight);
  createCanvas(canvas_size, canvas_size);
  background(0, 0, 0);
  ave = (width + height) / 2;
  explosionSize = 0.025 * height;
  aHeight = 0.2 * ave;
  aWidth = aHeight * (50 / 175);
  xPos = 0.5 * width - aWidth / 2;
  yPos = 0.8 * height - aHeight;
  speedInc = 0.0035 * ave;
  gameSpeed = 0.015 * ave;
  //Rocket Shots
  starsX = [];
  starsY = [];
  for (var i = 0; i < 50; i++) {
    starsX[i] = random(0, width);
    starsY[i] = random(0, height);
  }
  bullets = [];
  bullet = function(x, y) {
    this.x = x;
    this.y = y;
  };

  keyPressed = function() {
    if (!gameStarted) {
      gameStarted = true;
      startTime = millis();
      aHeight = 0.085 * ave;
      aWidth = aHeight * (50 / 175);
      xPos = 0.5 * width - aWidth / 2;
      yPos = 0.9 * height - aHeight;
      counter = 0;
    }
    if (keyCode === 32 || keyCode == 191) {
      addShot();
    }
    if (keyCode === 37) {
      direction = 3;
      moving = true;
    }
    if (keyCode === 38) {
      direction = 0;
      moving = true;
    }
    if (keyCode === 39) {
      direction = 1;
      moving = true;
    }
    if (keyCode === 40) {
      direction = 2;
      moving = true;
    }
  };
};

function drawAst(xloc, yloc, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(i * grayvalues);
    ellipse(xloc, yloc, size - i * steps, size - i * steps);
  }
}

function drawPlanet(xloc, yloc, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(i * grayvalues, i * grayvalues, 255);
    ellipse(xloc, yloc, size - i * steps, size - i * steps);
  }
}

function drawSun(xloc, yloc, size, num) {
  const grayvalues = 180 / num;
  const steps = size / num;
  for (let i = 20; i < num; i++) {
    fill(205 + (i * i * grayvalues) / 2,205 + (i * grayvalues) / 2,i * grayvalues);
    ellipse(xloc, yloc, size - i * steps, size - i * steps);
  }
}

function mouseClicked() {
  addShot();
}


var drawAllRockets = function(){
    console.log("Drawing:" + rockets.length + " rockets");
    for (index = 0; index < rockets.length; index++) { 
        console.log(rockets[index]); 
        drawThisRocket(rockets[index]['x'], rockets[index]['y'])
    } 
}

// Draws rocket at specified location
var drawRocket = function(x, y) {
  noStroke();
  if (counter % 10) {
    bigFlame = !bigFlame;
  }

  fill(220, 220, 220);
  rect(x, y, aWidth, aHeight);
  fill(200, 200, 200);
  rect(x, y, aWidth / 6, aHeight);
  fill(180, 180, 180);
  rect(x + aWidth / 3, y, (aWidth * 2) / 3, aHeight);
  fill(130, 130, 130);
  rect(x + (aWidth * 2) / 3, y, (aWidth * 1) / 3, aHeight);

  fill(160, 1, 70);
  triangle(x,y,x + aWidth / 2,y - aHeight * (7 / 16),x + aWidth,y);
  ellipse(x + aWidth / 2, y + aHeight * 0.01, aWidth, aHeight * 0.2);
  fill(160 + 20, 1 + 20, 70 + 20);
  triangle(x,y + aHeight,x - aWidth / 2,y + aHeight,x,y + aHeight / 4);
  fill(160, 1, 70);
  triangle(x + (3 / 2) * aWidth,y + aHeight,x + aWidth,y + aHeight,x + aWidth,y + aHeight / 4);

  triangle(x + aWidth / 3,y + aHeight,x + aWidth * (2 / 3),y + aHeight,x + aWidth / 2,y + aHeight / 4);

  if (bigFlame) {
    fill(255, 100, 10);
    triangle(x,y + aHeight,x + aWidth,y + aHeight,x + aWidth / 2,y + aHeight * (9 / 6));
    fill(210, 230, 0);
    triangle(x + aWidth / 4,y + aHeight,x + aWidth - aWidth / 4,y + aHeight,x + aWidth / 2,y + aHeight * (8 / 6));
  } else {
    fill(255, 100, 10);
    triangle(x,y + aHeight,x + aWidth,y + aHeight,x + aWidth / 2,y + aHeight * (11 / 6));
    fill(210, 230, 0);
    triangle(x + aWidth / 4,y + aHeight,x + aWidth - aWidth / 4,y + aHeight,x + aWidth / 2,y + aHeight * (10 / 6));
  }
};

// Draws rocket at specified location
var drawColoredRocket = function(x, y, r, g, b) {
  noStroke();
  if (counter % 10) {
    bigFlame = !bigFlame;
  }

  fill(220, 220, 220);
  rect(x, y, aWidth, aHeight);
  fill(200, 200, 200);
  rect(x, y, aWidth / 6, aHeight);
  fill(180, 180, 180);
  rect(x + aWidth / 3, y, (aWidth * 2) / 3, aHeight);
  fill(130, 130, 130);
  rect(x + (aWidth * 2) / 3, y, (aWidth * 1) / 3, aHeight);

  fill(r, g, b);
  triangle(x,y,x + aWidth / 2,y - aHeight * (7 / 16),x + aWidth,y);
  ellipse(x + aWidth / 2, y + aHeight * 0.01, aWidth, aHeight * 0.2);
  fill(r + 20, g + 20, b + 20);
  triangle(x,y + aHeight,x - aWidth / 2,y + aHeight,x,y + aHeight / 4);
  fill(r, g, b);
  triangle(x + (3 / 2) * aWidth,y + aHeight,x + aWidth,y + aHeight,x + aWidth,y + aHeight / 4);

  triangle(x + aWidth / 3,y + aHeight,x + aWidth * (2 / 3),y + aHeight,x + aWidth / 2,y + aHeight / 4);

  fill(190, 255, 190);
  triangle(x,y + aHeight,x + aWidth,y + aHeight,x + aWidth / 2,y + aHeight * (11 / 6));
  fill(255, 255, 255);
  triangle(x + aWidth / 4,y + aHeight,x + aWidth - aWidth / 4,y + aHeight,x + aWidth / 2,y + aHeight * (10 / 6));
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
              drawColoredRocket(rockets[key].x, rockets[key].y, colors[idx][0],colors[idx][1],colors[idx][2] );
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
  let _x = round(xPos + aWidth / 2) - 3;
  let _y = round(yPos - aHeight * (1 / 2));
  var b = new bullet(_x, _y);
  bullets.push(b);

  socket.send(`/shot {"browser_id":${browserId},"x":${_x}, "y":${_y}}`)
};

var displayBullets = function() {
  fill(255, 238, 0);
  for (var i = 0; i < bullets.length; i++) {
    rect(bullets[i].x, bullets[i].y, 6, 20);
  }
};

//Draws all asteroids at their updated positions. When destroyed they will be shot out of view
var drawObjects = function() {
  fill(65, 72, 163);
  for (var i = 0; i < spaceObjects.length; i++) {
    // if (spaceObjects[i].health == 2) {
    //   fill(65, 72, 163);
    // }else if (spaceObjects[i].health == 3) {
    //   fill(255, 238, 0);
    // }else  {
    //   fill(140, 138, 140);
    // }
    // let radius = spaceObjects[i].radius;

    if (spaceObjects[i].health == 3) {
      fill(255, 238, 0);
      drawSun(
        spaceObjects[i].x,
        spaceObjects[i].y,
        spaceObjects[i].radius * 1.7,
        50
      );
    }else if (spaceObjects[i].health == 1) {
      fill(140, 138, 140);
      drawAst(
        spaceObjects[i].x,
        spaceObjects[i].y,
        spaceObjects[i].radius * 1.8,
        50
      );
    } else {
      fill(65, 72, 163);
      drawPlanet(
        spaceObjects[i].x,
        spaceObjects[i].y,
        spaceObjects[i].radius * 1.8,
        50
      );
    }
  }
};

//leftArrow = 37 //TopArrow = 38 //rightArrow = 39 //leftArrow = 40
//= 0 north, 1= east, 2 = south, 3 = west

drawEndingScreen = function() {
  fill(255, 180, 0);
  textAlign(CENTER);
  textSize(0.08 * width);
  fill(0, 0, 0);
  text(
    "Game over",
    width / 2,
    Math.round(0.3 * height));
    textSize(0.05 * width);
  text("Score " + highScore, width / 2, Math.round(0.4 * height));
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
        yPos -= gameSpeed;
      } else if (direction === 1) {
        xPos += gameSpeed;
      } else if (direction === 2) {
        yPos += gameSpeed;
      } else if (direction === 3) {
        xPos -= gameSpeed;
      } else {
      }
    }
  } else {
    keys = 0;
    moving = false;
  }

  if (xPos < -1 * aWidth) {
    xPos = width;
  } else if (xPos >= width + aWidth) {
    xPos = -1 * aWidth;
  }

  if (yPos + aHeight > height) {
    yPos = height - aHeight;
  } else if (yPos < 0) {
    yPos = 0;
  }
  if (gameStarted) {
     socket.send(`/rocket {"browser_id":${browserId}, "x":${round(xPos)}, "y":${round(yPos)}}`)
  }
};

var restart = function() {
  socket.send(`/play`)
  explosionSize = 10;
  explosion = false;
  gameStarted = true;
  counter = 0;
  xPos = 0.4875 * width;
  yPos = 0.805 * height;
  bullets.splice(0);
  astDestroyed = 0;
  highScore = 0;
  //I need a way of telling the server to restart the game!
  // socket.send(`/play`)
};

//Main method of the program
draw = function() {
  // console.log(direction);
  background(0, 0, 0);
  fill(255, 245, 92);
  var starSize = ave * 0.003;
  for (var i = 0; i < starsX.length; i++) {
    ellipse(starsX[i], starsY[i], starSize, starSize);
  }

  if (!gameStarted) {
    counter++;
    textAlign(CENTER);
    fill(255, 255, 100);
    textSize(0.1075 * width);
    textFont("Helvetica");
    textFont("Ubuntu");
    text("LIGHTSPEED", 0.5 * width, 0.3 * height);
    textSize(0.05 * width);
    text("Press any button to begin", 0.5 * width, 0.4 * height);

    drawRocket(xPos, yPos);
  } else if (!explosion) {
    socket.send(`/state`);
    canRestart = false;
    counter++;
    updateRocket();
    displayBullets();
    drawRocket(xPos, yPos);
    drawOtherRockets();
    drawObjects();
    textSize(width * 0.03);
    fill(255, 0, 0);
    text(counter, 0.9 * width, 0.08 * height);
    if (counter > highScore){
      highScore = counter;
    }
  } else if (explosionSize < ave * 2) {
    drawRocket(xPos, yPos);
    drawObjects();
    fill(255, 180, 0);
    ellipse(xPos, yPos, explosionSize, explosionSize);
    explosionSize += 0.025 * width;
    if (explosionSize > width / 2) {
      drawEndingScreen();
    }

    if (millis() - startTime > 1500) {
      canRestart = true;
    }
    if (keyIsPressed && canRestart) {
      console.log("waited:" + millis() - startTime); 
      // restart();
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
    text()
  }
};