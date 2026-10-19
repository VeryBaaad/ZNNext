/*
 * This file is part of ZNNext.
 *
 * ZNNext is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZNNext is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;

use object::elf::{
    EM_386, EM_AARCH64, EM_ARM, EM_RISCV, EM_X86_64, ET_DYN, FileHeader32, FileHeader64,
};
use object::read::elf::FileHeader as ElfFileHeader;
use object::{Endianness, FileKind, Object, ObjectKind, ObjectSymbol};

use crate::arch::Arch;

#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    pub is_64: bool,
    pub kind: u16,
    pub machine: u16,
    pub entry: u64,
}

impl ElfHeader {
    pub fn is_position_independent(&self) -> bool {
        self.kind == ET_DYN
    }
}

fn parse_header(bytes: &[u8]) -> Option<ElfHeader> {
    match FileKind::parse(bytes).ok()? {
        FileKind::Elf64 => header_of::<FileHeader64<Endianness>>(bytes),
        FileKind::Elf32 => header_of::<FileHeader32<Endianness>>(bytes),
        _ => None,
    }
}

fn header_of<H: ElfFileHeader>(bytes: &[u8]) -> Option<ElfHeader> {
    let header = H::parse(bytes).ok()?;
    let endian = header.endian().ok()?;
    Some(ElfHeader {
        is_64: header.is_type_64(),
        kind: header.e_type(endian),
        machine: header.e_machine(endian),
        entry: header.e_entry(endian).into(),
    })
}

pub fn header_of_file(path: &Path) -> Option<ElfHeader> {
    let file = File::open(path).ok()?;
    let mut buffer = [0u8; 64];
    let read = file.read_at(&mut buffer, 0).ok()?;
    parse_header(&buffer[..read])
}

pub fn arch_of(header: &ElfHeader) -> Arch {
    if header.is_64 {
        match header.machine {
            EM_AARCH64 => Arch::Arm64,
            EM_X86_64 => Arch::X86_64,
            EM_RISCV => Arch::Riscv64,
            _ => Arch::Unknown,
        }
    } else {
        match header.machine {
            EM_ARM => Arch::Arm32,
            EM_386 => Arch::X86,
            _ => Arch::Unknown,
        }
    }
}

pub fn symbol_address(path: &Path, base: usize, symbol: &str) -> usize {
    let Ok(bytes) = std::fs::read(path) else {
        return 0;
    };
    let Ok(file) = object::File::parse(&*bytes) else {
        return 0;
    };
    let address = file
        .dynamic_symbols()
        .chain(file.symbols())
        .find(|candidate| {
            !candidate.is_undefined() && candidate.name().is_ok_and(|name| name == symbol)
        })
        .and_then(|candidate| usize::try_from(candidate.address()).ok());
    match address {
        Some(value) if file.kind() == ObjectKind::Dynamic => base.wrapping_add(value),
        Some(value) => value,
        None => 0,
    }
}
