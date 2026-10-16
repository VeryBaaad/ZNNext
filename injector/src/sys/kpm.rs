/*
 * This file is part of Zygisk Next Next.
 *
 * Zygisk Next Next is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Zygisk Next Next is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Zygisk Next Next. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

use std::io;

pub const MAGIC: u64 = 0x5A4E4E4B504D0001;
pub const MAX_PAYLOAD: usize = 4096;

pub fn read_records(buffer: &mut [u8], since_seq: u64) -> io::Result<usize> {
    if buffer.is_empty() || buffer.len() > MAX_PAYLOAD {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    // SAFETY: syscall
    let result = unsafe {
        libc::syscall(
            libc::SYS_prctl,
            MAGIC,
            buffer.as_mut_ptr(),
            buffer.len(),
            since_seq,
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result as usize)
}
