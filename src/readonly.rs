pub fn help() -> Vec<String> {
        "navigation:
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
quick commands: (in view mode only)
        (number)j - jumps to line number
        rr - deletes current line"
        .lines().map(|x| x.to_string()).collect()
}
