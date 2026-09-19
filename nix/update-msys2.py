"""Pins the MSYS2 packages the Windows build links against and ships with.

Resolves everything ROOTS needs to run from MSYS2's current UCRT64 package
database, then writes the file name and hash of each package to
msys2-packages.json for windows.nix to fetch. Run it again to update them.

This is vibe coded
"""

import io
import json
import re
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path

REPO = "https://repo.msys2.org/mingw/ucrt64"
PREFIX = "mingw-w64-ucrt-x86_64-"
# The same libraries the Linux dev shell uses, except gst-plugins-bad, which
# drags in ImageMagick, Python and Tcl/Tk for plugins nothing here plays. The
# part of it GTK itself needs, gst-plugins-bad-libs, comes in through gtk4.
ROOTS = [
    "gtk4",
    "gstreamer",
    "gst-plugins-base",
    "gst-plugins-good",
    "gst-libav",
]
DEFAULT_OUTPUT = Path(__file__).with_name("msys2-packages.json")


def main():
    output = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_OUTPUT

    packages, providers = load_database()

    wanted = [PREFIX + root for root in ROOTS]
    resolved = {}
    while wanted:
        name = wanted.pop()
        name = name if name in packages else providers[name]
        if name in resolved:
            continue

        package = packages[name]
        resolved[name] = package
        wanted.extend(strip_version(dep) for dep in package.get("DEPENDS", []))

    pins = [
        {
            "filename": resolved[name]["FILENAME"][0],
            "sha256": resolved[name]["SHA256SUM"][0],
        }
        for name in sorted(resolved)
    ]
    output.write_text(json.dumps(pins, indent=2) + "\n")
    print(f"Pinned {len(pins)} packages in {output}")


def load_database():
    """Every package in the repo by name, plus which package provides each
    virtual name (like `cc-libs`) that others depend on."""
    with urllib.request.urlopen(f"{REPO}/ucrt64.db.tar.zst") as response:
        compressed = response.read()
    archive = subprocess.run(
        ["zstd", "--decompress", "--stdout"],
        input=compressed,
        capture_output=True,
        check=True,
    ).stdout

    packages, providers = {}, {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as database:
        for member in database:
            if not member.name.endswith("/desc"):
                continue

            package = parse_desc(database.extractfile(member).read().decode())
            name = package["NAME"][0]
            packages[name] = package
            for provided in package.get("PROVIDES", []):
                providers.setdefault(strip_version(provided), name)

    return packages, providers


def parse_desc(text):
    """A desc file is `%FIELD%` headers, each followed by lines of values."""
    fields, field = {}, None
    for line in text.splitlines():
        if line.startswith("%") and line.endswith("%"):
            field = line.strip("%")
            fields[field] = []
        elif line and field:
            fields[field].append(line)

    return fields


def strip_version(dependency):
    """`glib2>=2.80` depends on `glib2`."""
    return re.split(r"[<>=]", dependency, maxsplit=1)[0]


if __name__ == "__main__":
    main()
