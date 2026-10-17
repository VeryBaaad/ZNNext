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
use std::os::fd::{BorrowedFd, RawFd};

use nix::poll::{PollFd, PollFlags, PollTimeout, poll};

pub fn poll_readable(descriptors: &[RawFd], timeout_ms: i32) -> io::Result<Vec<RawFd>> {
    if descriptors.is_empty() {
        return Ok(Vec::new());
    }

    let mut poll_descriptors: Vec<PollFd> = descriptors
        .iter()
        .map(|descriptor| {
            // SAFETY: the caller keeps every descriptor open for the whole call
            PollFd::new(
                unsafe { BorrowedFd::borrow_raw(*descriptor) },
                PollFlags::POLLIN,
            )
        })
        .collect();

    let timeout = PollTimeout::try_from(timeout_ms).unwrap_or(PollTimeout::NONE);
    poll(&mut poll_descriptors, timeout).map_err(io::Error::from)?;

    Ok(poll_descriptors
        .iter()
        .enumerate()
        .filter(|(_, descriptor)| {
            descriptor
                .revents()
                .is_some_and(|events| !events.is_empty())
        })
        .map(|(index, _)| descriptors[index])
        .collect())
}
