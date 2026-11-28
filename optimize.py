#!/usr/bin/env python3
"""Build optimization profiles shared by every ZNNext project module."""
import json
import os
from dataclasses import asdict, dataclass, fields, replace

PROFILE_NONE = "none"
PROFILE_SIZE = "size"
PROFILE_PERF = "perf"
PROFILE_MAX = "max"
PROFILES = (PROFILE_NONE, PROFILE_SIZE, PROFILE_PERF, PROFILE_MAX)
DEFAULT_PROFILE = PROFILE_SIZE

ENVIRONMENT_KEY = "ZNN_OPT_SETTINGS"


@dataclass(frozen=True)
class Settings:
    profile: str = DEFAULT_PROFILE
    codegen_level: str = "z"
    lto: str = "full"
    icf: str = "all"
    strip: str = "all"
    unwind: str = "compact"
    eh_frame_hdr: bool = True
    rosegment: bool = False
    hash_style: str = "gnu"
    build_id: bool = False
    merge_constants: bool = True
    ident: bool = False
    section_gc: bool = True
    lto_codegen: int = 2
    linker_opt: int = 2
    rust_opt: str = "z"
    rust_lto: str = "fat"
    rust_codegen_units: int = 1
    rust_panic: str = "abort"
    rust_panic_immediate_abort: bool = True
    rust_location_detail: str = "none"
    rust_build_std: bool = True
    rust_debuginfo: str = "none"
    debug_symbols: bool = False
    xz_level: int = 9
    xz_threshold: int = 1024
    webui_minify: bool = True
    webui_drop_icon_font: bool = True
    extra_cflags: tuple = ()
    extra_ldflags: tuple = ()
    extra_rustflags: tuple = ()

    def to_environment(self):
        return json.dumps(asdict(self), separators=(",", ":"))


_BASE = Settings()

_PROFILE_VALUES = {
    PROFILE_NONE: dict(
        profile=PROFILE_NONE,
        codegen_level="",
        lto="off",
        icf="safe",
        strip="none",
        unwind="full",
        eh_frame_hdr=True,
        rosegment=True,
        hash_style="both",
        build_id=True,
        merge_constants=False,
        ident=True,
        section_gc=True,
        lto_codegen=2,
        linker_opt=2,
        rust_opt="z",
        rust_lto="fat",
        rust_codegen_units=1,
        rust_panic="abort",
        rust_panic_immediate_abort=False,
        rust_location_detail="full",
        rust_build_std=False,
        rust_debuginfo="none",
        debug_symbols=True,
        xz_level=9,
        xz_threshold=1024,
        webui_minify=True,
        webui_drop_icon_font=False,
    ),
    PROFILE_SIZE: dict(
        profile=PROFILE_SIZE,
        codegen_level="z",
        lto="full",
        icf="all",
        strip="all",
        unwind="compact",
        eh_frame_hdr=True,
        rosegment=False,
        hash_style="gnu",
        build_id=False,
        merge_constants=True,
        ident=False,
        section_gc=True,
        lto_codegen=2,
        linker_opt=2,
        rust_opt="z",
        rust_lto="fat",
        rust_codegen_units=1,
        rust_panic="abort",
        rust_panic_immediate_abort=True,
        rust_location_detail="none",
        rust_build_std=True,
        rust_debuginfo="none",
        debug_symbols=False,
        xz_level=9,
        xz_threshold=1024,
        webui_minify=True,
        webui_drop_icon_font=True,
    ),
    PROFILE_PERF: dict(
        profile=PROFILE_PERF,
        codegen_level="3",
        lto="thin",
        icf="safe",
        strip="all",
        unwind="full",
        eh_frame_hdr=True,
        rosegment=True,
        hash_style="gnu",
        build_id=True,
        merge_constants=False,
        ident=False,
        section_gc=True,
        lto_codegen=3,
        linker_opt=2,
        rust_opt="3",
        rust_lto="thin",
        rust_codegen_units=1,
        rust_panic="abort",
        rust_panic_immediate_abort=False,
        rust_location_detail="full",
        rust_build_std=False,
        rust_debuginfo="line-tables",
        debug_symbols=False,
        xz_level=9,
        xz_threshold=1024,
        webui_minify=True,
        webui_drop_icon_font=False,
    ),
    PROFILE_MAX: dict(
        profile=PROFILE_MAX,
        codegen_level="z",
        lto="full",
        icf="all",
        strip="all",
        unwind="compact",
        eh_frame_hdr=False,
        rosegment=False,
        hash_style="gnu",
        build_id=False,
        merge_constants=True,
        ident=False,
        section_gc=True,
        lto_codegen=1,
        linker_opt=1,
        rust_opt="z",
        rust_lto="fat",
        rust_codegen_units=1,
        rust_panic="abort",
        rust_panic_immediate_abort=True,
        rust_location_detail="none",
        rust_build_std=True,
        rust_debuginfo="none",
        debug_symbols=False,
        xz_level=9,
        xz_threshold=4096,
        webui_minify=True,
        webui_drop_icon_font=True,
    ),
}


def profile_settings(profile):
    if profile not in PROFILES:
        raise ValueError(f"unknown optimization profile: {profile}")
    return replace(_BASE, **_PROFILE_VALUES[profile])


def from_environment():
    raw = os.environ.get(ENVIRONMENT_KEY)
    if not raw:
        return profile_settings(DEFAULT_PROFILE)
    values = json.loads(raw)
    known = {field.name for field in fields(Settings)}
    unknown = sorted(set(values) - known)
    if unknown:
        raise ValueError(f"unknown optimization settings: {', '.join(unknown)}")
    return replace(_BASE, **values)


def describe(settings):
    return (
        f"opt={settings.profile} O{settings.codegen_level or '-'} "
        f"lto={settings.lto} icf={settings.icf} strip={settings.strip} "
        f"unwind={settings.unwind} rust=O{settings.rust_opt}/{settings.rust_lto} "
        f"xz={settings.xz_level}"
    )


def strip_mode(settings, variant):
    if variant == "debug" and settings.debug_symbols:
        return "none"
    return settings.strip


def source_symbol_settings(settings):
    return replace(settings, strip="none", build_id=True, rust_debuginfo="line-tables")


def loader_compile_flags(settings, variant, legacy):
    """Compiler flags added on top of the project's baseline warnings."""
    flags = []
    if settings.codegen_level:
        flags.append(f"-O{settings.codegen_level}")
    if settings.lto != "off" and not legacy:
        flags.append("-flto=thin" if settings.lto == "thin" else "-flto")
    flags.extend(
        [
            "-Wno-unused",
            "-Wno-unused-parameter",
            "-fvisibility=hidden",
            "-fvisibility-inlines-hidden",
        ]
    )
    if settings.section_gc:
        flags.extend(["-ffunction-sections", "-fdata-sections"])
    if settings.merge_constants:
        flags.append("-fmerge-all-constants")
    if not settings.ident:
        flags.append("-fno-ident")
    if settings.unwind != "full":
        flags.extend(["-fno-unwind-tables", "-fno-asynchronous-unwind-tables"])
    flags.extend(settings.extra_cflags)
    return flags


def loader_linker_flags(settings, variant, legacy):
    strip = strip_mode(settings, variant)
    flags = [f"-O{settings.linker_opt}", "-Wl,--exclude-libs,ALL"]
    if settings.section_gc:
        flags.append("-Wl,--gc-sections")
    if strip == "all":
        flags.append("-Wl,--strip-all")
    elif strip == "debug":
        flags.append("-Wl,--strip-debug")
    if legacy:
        return " ".join(flags)
    if settings.icf != "none":
        flags.append(f"-Wl,--icf={settings.icf}")
    if not settings.eh_frame_hdr:
        flags.append("-Wl,--no-eh-frame-hdr")
    if not settings.rosegment:
        flags.append("-Wl,--no-rosegment")
    flags.append("-Wl,-z,noseparate-code")
    if settings.hash_style == "gnu":
        flags.append("-Wl,--hash-style=gnu")
    if not settings.build_id:
        flags.append("-Wl,--build-id=none")
    if settings.lto != "off":
        flags.append(f"-Wl,--lto-O{settings.lto_codegen}")
    flags.extend(settings.extra_ldflags)
    return " ".join(flags)


def cmake_optimization(settings, variant, legacy):
    """Return the CMake build type plus its per-config optimization string."""
    # -g also makes clang emit .cfi_startproc around inline asm, which some
    # bundled backends rely on; the linker strips it again when requested.
    debug_info = "-g"
    level = f"-O{settings.codegen_level}" if settings.codegen_level else ""
    parts = [part for part in (level, debug_info) if part]
    if variant != "debug":
        parts.append("-DNDEBUG")
    return (
        ("Debug" if variant == "debug" else "RelWithDebInfo"),
        " ".join(parts),
    )


def immediate_abort(settings):
    return settings.rust_panic_immediate_abort and settings.rust_panic == "abort"


def rust_flags(settings, variant, legacy, allow_immediate_abort=True):
    flags = []
    if legacy:
        flags.extend(["-Zunstable-options", "-C linker-flavor=gnu-cc"])
    if immediate_abort(settings) and allow_immediate_abort:
        if "-Zunstable-options" not in flags:
            flags.append("-Zunstable-options")
        flags.append("-Cpanic=immediate-abort")
    if settings.rust_location_detail == "none":
        flags.append("-Zlocation-detail=none")
    if settings.section_gc:
        flags.append("-C link-arg=-Wl,--gc-sections")
    if settings.hash_style == "gnu" and not legacy:
        flags.append("-C link-arg=-Wl,--hash-style=gnu")
    if not settings.build_id:
        flags.append("-C link-arg=-Wl,--build-id=none")
    if strip_mode(settings, variant) == "all":
        flags.append("-C link-arg=-Wl,--strip-all")
    if legacy:
        return " ".join(flags)
    for argument in (
        "-Wl,-z,noseparate-code",
        "-Wl,--no-rosegment",
    ):
        flags.append(f"-C link-arg={argument}")
    if settings.icf != "none":
        flags.append(f"-C link-arg=-Wl,--icf={settings.icf}")
    if not settings.eh_frame_hdr:
        flags.append("-C link-arg=-Wl,--no-eh-frame-hdr")
    if settings.lto != "off":
        flags.append(f"-C link-arg=-Wl,--lto-O{settings.lto_codegen}")
    flags.extend(settings.extra_rustflags)
    flags.extend(f"-C link-arg={flag}" for flag in settings.extra_ldflags)
    return " ".join(flags)


def rust_target_environment(
    settings, variant, target, legacy=False, allow_immediate_abort=True
):
    return {
        rust_target_flags_key(target): rust_flags(
            settings, variant, legacy, allow_immediate_abort=allow_immediate_abort
        )
    }


def rust_target_flags_key(target):
    return "CARGO_TARGET_" + target.upper().replace("-", "_") + "_RUSTFLAGS"


def rust_profile_environment(settings, variant, codegen_units=None):
    profile = "dev" if variant == "debug" else "release"
    prefix = f"CARGO_PROFILE_{profile.upper()}_"
    units = settings.rust_codegen_units if codegen_units is None else codegen_units
    return {
        f"{prefix}OPT_LEVEL": settings.rust_opt,
        f"{prefix}LTO": settings.rust_lto,
        f"{prefix}CODEGEN_UNITS": str(units),
        f"{prefix}PANIC": settings.rust_panic,
        f"{prefix}STRIP": "symbols" if strip_mode(settings, variant) != "none" else "none",
        f"{prefix}DEBUG": (
            "0" if settings.rust_debuginfo == "none" or strip_mode(settings, variant) != "none"
            else "line-tables-only"
        ),
    }

UNSAFE_SINGLE_UNIT_OPT_LEVELS = ("2", "3", "s")
UNSAFE_SINGLE_UNIT_CODEGEN_UNITS = 16


def rust_codegen_units_for(settings, abi):
    if (
        abi == "mips64"
        and settings.rust_codegen_units == 1
        and settings.rust_opt in UNSAFE_SINGLE_UNIT_OPT_LEVELS
    ):
        return UNSAFE_SINGLE_UNIT_CODEGEN_UNITS
    return settings.rust_codegen_units


def rust_build_std_arguments(settings):
    if not settings.rust_build_std or settings.profile == PROFILE_NONE:
        return []
    return ["-Zbuild-std=std,panic_abort"]
