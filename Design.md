## Going to need to redesign architecture of this game for it to work
All positions are changing from direct pixel counts to porporions of screen width and height
Thus, the game state will know the size of every screen playing the game.

In rust, all fractions will be stored, this will refer to the current position of the rockets, asteroids, and shots fired.

Rockets will maintain as a hashmap from browserID:Rocket
- Only a 2D vector of points is needed to maintain the coordinates of the rocket

Shots will only need to be a vector a 2D points
- Thus a queue will be created, after the shots reach a certain altiude (negative altitude) they will be destroyed

Asteroids will need to be x,y,velocity,width,health
- These values will maintain the same, except that these values excluding health will be converted to floating point
