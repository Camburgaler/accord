# Accord

I'm no network engineer, please don't judge me

## Usage

What I do is open 3 VS Code terminals; one for each functional piece herein.

1. Accord
    - `cargo build --release`
    - Move `target/release/accord.dll` to your ME2 directory
    - Update the `config_eldenring.toml` file in your ME2 directory to include `accord.dll`
    - Launch Elden Ring using `launchmod_eldenring.bat` in your ME2 directory
2. Bridge
    - `cd crates/bridge`
    - `cargo run --release`
3. GUI
    - `cd crates/gui`
    - `cargo run --release`

After that, I just rebuild/rerun as necessary, ensuring to move the `accord.dll` file again each time it's rebuilt.
