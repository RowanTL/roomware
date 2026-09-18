# RoomWare

This is basic escape room management software. The UI is self-explanatory.
There are many configuration options available via two toml files.

## Usage

When you click on a hint in the hint menu, the long version will appear in
typeable hint box. You can send them with the send button after this.

## Configuration

There are two different files used for configuration:

- `config.toml`
- `hints.toml`

`config.toml` holds general configuration for the various buttons around the interface. For example,
you want to change the background color of the player's view to black instead of brown. This is
where you can change it.

`hints.toml` holds pre-typed hints for the room read in order they are found in the file.
