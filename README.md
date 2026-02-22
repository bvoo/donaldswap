# DonaldSwap

A lightweight, automated window swapper designed. DonaldSwap randomly rotates focus between a set of configured game windows and configured time intervals, simulating pause/unpause inputs between swaps.

## Features

- **Randomized Rotation**: Seamlessly swaps between an unlimited number of tracked game windows based on a configurable min/max timer.
- **Auto-Pausing**: Automatically sends an `ESC` key input to games when swapping away to pause them, and another `ESC` input when swapping back to unpause them (configurable per-game).
- **Web Dashboard**: Clean, dark-mode web interface to manage your rotation, monitor timers, and manually force/pause the swap sequence.
- **OBS Browser Source**: Built-in HUD specifically designed to be added as an OBS browser source, showing your viewers the current game, time elapsed, and time until the next swap.
- **Focus Stealing Bypass**: Bypasses Windows' built-in foreground window locks using low-level API input simulation to ensure the games reliably pop up.
- **Swap History**: Keeps track of exactly how long you spent in games during the session.

## Installation

### Pre-built Binaries (Windows only)
1. Go to the [Releases](https://github.com/yourusername/donaldswap/releases) page.
2. Download the latest `donaldswap-windows-x64.zip`.
3. Extract the folder.
4. Run `donaldswap.exe`.

### From Source
You will need [Rust and Cargo installed](https://rustup.rs/).
```bash
git clone https://github.com/yourusername/donaldswap.git
cd donaldswap
cargo build --release
# Run it
./target/release/donaldswap.exe
```

*Note: The `static/` directory must be in the same directory as where you execute the binary.*

## Usage

When you launch `donaldswap.exe`, a terminal window will open and display the URLs for the interfaces:

```text
===============================================
DonaldSwap is running!
Dashboard:       http://127.0.0.1:3000
OBS Browser Src: http://127.0.0.1:3000/obs.html
===============================================
```

### 1. Setup Your Games
1. Open the games you want to rotate between.
2. Go to the **Dashboard** (`http://127.0.0.1:3000`).
3. In the "Add from Open Windows" panel on the right, click **Refresh**.
4. Find your games in the list and click **Add**.
5. Once added, you can click on the game's title in the "Rotation List" to rename it (this is what shows up on stream).

### 2. Configure Settings
- By default, the swapper picks a random time between **5 and 15 minutes**. You can adjust this in the "Configuration" panel.
- For each game, you can toggle `ESC on Leave` and `ESC on Enter`. If a game automatically pauses when it loses focus, you might want to turn off `ESC on Leave` so the swapper doesn't accidentally unpause it.

### 3. Add to OBS
1. In OBS, add a new **Browser Source**.
2. Set the URL to `http://127.0.0.1:3000/obs.html`.
3. Set the width/height to your preference (e.g., Width: 600, Height: 150).
4. (Optional) Check "Hide Next Swap Information" in the web dashboard if you want the next swap time to be a surprise to chat!

## Configuration File

The app stores your settings in a `config.toml` file generated in the same directory as the executable. It auto-updates whenever you change settings in the web UI, but you can also edit it manually.

## Platform Support

Currently, DonaldSwap only supports **Windows**. It relies heavily on Win32 APIs for precise window enumeration, foreground locking workarounds, and input hooking. 

## License

MIT
