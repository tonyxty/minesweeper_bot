# minesweeper_bot

A telegram bot of various board games, utilizing its [inline keyboard](https://core.telegram.org/bots#inline-keyboards-and-on-the-fly-updating) feature.

### Source code structure
Each game is factored as "game logic" and "interaction logic":
  - _Game logic_ is the interface-independent part of the game.  Think of it this way: it is the part of code that can be reused without change if someone decides to write a Gtk version or TUI version of the game.
  - _Interaction logic_ defines how the game renders itself and responds to input events.  For telegram bots, the only user input is a click on a square, and the game responds with an updated inline keyboard and a text message.

  Interaction logic is further divided into _player control logic_ and _abstract play logic_.  For example, any two-player competitive board game will need to check if the click event come from the current player, and that part is handled by the player control logic.  The abstract play logic can then assume players always make moves in turn.
