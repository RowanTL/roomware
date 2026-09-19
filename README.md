# RoomWare

This is basic escape room management software. The UI is self-explanatory.
There are many configuration options available via two toml files.

## Usage

Run it with the toml files you want it to read:

```
roomware --config config.toml --hints hints.toml
```

Both flags are optional. Without `--config` every setting falls back to its
built in default, and without `--hints` the hint menu is empty.

When you click on a hint in the hint menu, it will appear in the typeable hint
box underneath. You can send them with the send button after this.

## Building for Windows

```
nix build .#windows-installer
```

This cross compiles the program and packs it, along with the GTK and GStreamer
DLLs it needs, into a single `setup.exe` in `./result`. Copy that one file over
to the Windows machine and run it. It installs into `Program Files`, adds a
Start Menu shortcut and can be removed again from Add or Remove Programs.

`nix build .#windows` gives the same build as a plain directory instead, for
running it off a stick without installing. Copy the whole directory over,
keeping its layout, and run `bin\roomware.exe` from inside it.

Those DLLs are MSYS2's Windows builds, pinned in `nix/msys2-packages.json`,
because nixpkgs cannot build GTK for Windows itself. To move to their current
versions, run `nix run .#update-msys2` and build again.

## Configuration

There are two different files used for configuration:

- `config.toml`
- `hints.toml`

`config.toml` holds general configuration for the various buttons around the interface. For example,
you want to change the background color of the player's view to black instead of brown. This is
where you can change it.

`hints.toml` holds pre-typed hints for the room read in order they are found in the file.

Paths inside either file, such as a video, a ping sound or a picture hint, are relative to the file
they are written in. A `./picture.png` in `hints.toml` is the picture sitting next to `hints.toml`,
wherever you keep it. Absolute paths work too.
