<div align="center">
<h1>⏳ trackie</h1>

`trackie` is a private, daemon-less time tracker running in your CLI.
</div>


<div align="center">
<a href="https://asciinema.org/a/429400" target="_blank"><img src=".github/media/terminal-session.gif" width="90%" align="center"/></a>
</div>

---

Trackie offers an easy CLI to track the time you spent on your various projects. It bundles up your busy days in easy
digestible reports that you can filter to your liking.

All data is saved to `($XDG_DATA_HOME|%APPDATA%)/trackie/trackie.json` where it can be processed by other tools.

Trackie is optimized to lower its complexity to the absolute minimum.

---

## Usage

Trackie currently consists of three simple commands:

- `trackie start <project-ID>`: Starts time tracking for a project with the given ID.
- `trackie stop`: Stops the time tracking
- `trackie status [-f <format>]`: Prints information about the currently tracked project.
- `trackie report [-d <num-days>] [-i/--include-empty-days]`: Creates a report for the last *n* days (default: 5).

## Shell integration

Trackie's customizable `status` command is a great fit for many shells.


### Starship 

The following starship configuration, for example, leads to a nice element that shows on which project you are working
on and for how long you are already doing that.

```toml
[custom.trackie]
command = 'trackie status -f "%p[%D]"'
# Remove this line if you don't want to hide the trackie block if no project is currently tracked
when = "trackie status"
symbol = "⏳"
style = "bg:cyan fg:black"
format = "[$symbol($output)]($style)[](fg:cyan)"
```

This configuration leads to the following result:

<div>
  <img src=".github/media/shell-integration-screenshot.png" alt="Windows Terminal with starship and trackie extension"/>
</div>

## Installation

#### Download prebuilt release

1. Download the binary for your respective OS from [the latest release](https://github.com/beatbrot/trackie/releases).
2. Copy it to a folder in your `PATH`.

#### Compile from source using cargo

1. Run `cargo install trackie`.

#### Compile from source using make

```
git clone https://github.com/beatbrot/trackie
cd trackie
make
sudo make install
```

