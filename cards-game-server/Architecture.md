# cards-game-server

## [connection.rs](./src/api/connection.rs)

Handles incoming requests and passes them to the respective module(s) responsible for the request.

## [websocket.rs](./src/api/websocket.rs)

Handles communication and keep-alive with the client, and passes any other messages to the game.

## [game.rs](./src/game/game.rs)

When a websocket connection is opened they also connect to a game, multiple clients can be connected to a single game.
The game recieves all messages from all connected clients, and can send messages to multiple or all clients.
