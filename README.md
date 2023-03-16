
# vcte 

vcte or very cool text editor is a work in progress terminal based text editor written in rust.

![total lines](https://img.shields.io/tokei/lines/github/DARKDRAGON532/vcte?style=flat-square)
![stars](https://img.shields.io/github/stars/DARKDRAGON532/vcte?style=flat-square)


## Installation

vcte is still in beta, you can try it out by

```bash
  git clone https://github.com/DARKDRAGON532/vcte.git
  cd vcte
  cargo run --release
```
    
## Features

- tab system or multiple file support
- home screen
- some basic commands
- hotkeys
- has some colour


## Screenshots

![App Screenshot](https://i.imgur.com/4DQXrLg.png)
![App Screenshot](https://i.imgur.com/1GxhEZQ.png)


## Documentation
```
navigation:
        (in view mode) wasd or arrow keys
        (in insert mode) arrow keys
commands (press : to enter command mode):
        :q or :quit - quit
        :s :save - save current file
        :o <path> or :open <path> - open file 
        :h or :help - show help (this menu)
hotkeys:
        i - enter insert mode
        n - go to next tab
        shift + n - move tab to the right
        b - go to previous tab
        shift + b - move tab to the left
        x - close current tab
        esc - escape almost everything
```
## Authors

- [@DARKDRAGON532](https://www.github.com/DARKDRAGON532)

