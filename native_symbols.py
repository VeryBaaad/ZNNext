import re
import shutil
import stat
import sys
import tempfile
from pathlib import Path

import build as znn


BUILD_ID = re.compile(r"^[ \t]*Build ID: ([0-9a-f]+)[ \t]*$", re.IGNORECASE | re.MULTILINE)
LINE_ENTRY = re.compile(r"^0x[0-9a-f]+[ \t]+[1-9][0-9]*[ \t]+(?![^\n]*\bend_sequence\b)", re.MULTILINE)


def _llvm_tool(ndk, name):
    suffix = ".exe" if sys.platform == "win32" else ""
    return Path(ndk) / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin" / f"{name}{suffix}"


def _build_id(path, readelf):
    try:
        with path.open("rb") as stream:
            if stream.read(4) != b"\x7fELF":
                raise znn.BuildError(f"expected an ELF file at {path}")
    except OSError as error:
        raise znn.BuildError(f"cannot read ELF file {path}: {error}") from error
    notes = znn.run_capture([readelf, "--notes", path])
    ids = {value.lower() for value in BUILD_ID.findall(notes)}
    if len(ids) != 1:
        raise znn.BuildError(f"expected one GNU Build ID in {path}")
    return ids.pop()


def validate_pair(binary, debug_file, ndk):
    binary = Path(binary)
    debug_file = Path(debug_file)
    readelf = _llvm_tool(ndk, "llvm-readelf")
    build_id = _build_id(binary, readelf)
    debug_id = _build_id(debug_file, readelf)
    if build_id != debug_id:
        raise znn.BuildError(
            f"GNU Build ID mismatch: {binary} has {build_id}, {debug_file} has {debug_id}"
        )
    lines = znn.run_capture([_llvm_tool(ndk, "llvm-dwarfdump"), "--debug-line", debug_file])
    if not LINE_ENTRY.search(lines):
        raise znn.BuildError(f"DWARF line information is empty or missing in {debug_file}")
    return build_id


def split_binary(raw_path, destination, ndk, strip_mode):
    if strip_mode not in ("none", "debug", "all"):
        raise znn.BuildError(f"unknown native strip mode: {strip_mode}")

    raw_path = Path(raw_path)
    destination = Path(destination)
    debug_file = destination.with_name(destination.name + ".dbg")
    if raw_path.resolve() in (destination.resolve(), debug_file.resolve()):
        raise znn.BuildError(f"native symbol splitting requires a separate output from {raw_path}")

    objcopy = _llvm_tool(ndk, "llvm-objcopy")
    try:
        mode = stat.S_IMODE(raw_path.stat().st_mode)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=destination.parent) as directory:
            root = Path(directory)
            binary = root / destination.name
            debug = root / debug_file.name
            znn.run([objcopy, "--only-keep-debug", raw_path, debug], prefix="symbols")
            if strip_mode == "none":
                shutil.copy2(raw_path, binary)
            else:
                znn.run([objcopy, f"--strip-{strip_mode}", raw_path, binary], prefix="symbols")
            validate_pair(binary, debug, ndk)
            binary.chmod(mode)
            debug.chmod(mode)
            debug.replace(debug_file)
            binary.replace(destination)
    except OSError as error:
        raise znn.BuildError(f"cannot write native symbols for {destination}: {error}") from error
    return debug_file
