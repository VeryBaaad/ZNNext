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

use nix::unistd::{SysconfVar, sysconf};

pub fn size() -> u64 {
    sysconf(SysconfVar::PAGE_SIZE)
        .ok()
        .flatten()
        .filter(|value| *value > 0)
        .map_or(4096, |value| value as u64)
}

pub fn round_up(value: usize) -> usize {
    let page = size() as usize;
    if page <= 1 || value.is_multiple_of(page) {
        return value;
    }
    value.div_ceil(page) * page
}

pub fn pad(content: &[u8]) -> Vec<u8> {
    let aligned = round_up(content.len());
    if aligned == content.len() {
        return content.to_vec();
    }
    let mut out = Vec::with_capacity(aligned);
    out.extend_from_slice(content);
    out.resize(aligned, 0);
    out
}
