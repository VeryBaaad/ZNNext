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

use crate::kernel::{EventKind, KernelEvent};
use crate::log::logi;
use crate::sys::Pid;
use crate::sys::kpm;

const MAX_ROUNDS: usize = 4;

#[derive(Debug)]
pub struct KpmTracker {
    since_seq: u64,
    buffer: Vec<u8>,
}

impl KpmTracker {
    pub fn start() -> Result<Self, String> {
        let mut buffer = vec![0u8; kpm::MAX_PAYLOAD];
        match kpm::read_records(&mut buffer, 0) {
            Ok(_) => {
                logi!("kernel tracking: KernelPatch znn channel ready");
                Ok(Self {
                    since_seq: 0,
                    buffer,
                })
            }
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn drain(&mut self, events: &mut Vec<KernelEvent>) {
        for _ in 0..MAX_ROUNDS {
            let Ok(read) = kpm::read_records(&mut self.buffer, self.since_seq) else {
                return;
            };
            if read == 0 {
                return;
            }
            let filled = read.min(self.buffer.len());
            let mut last_seq = self.since_seq;
            let text = String::from_utf8_lossy(&self.buffer[..filled]);
            for line in text.lines() {
                let Some(record) = parse_line(line) else {
                    continue;
                };
                last_seq = last_seq.max(record.seq);
                if record.pid <= 0 {
                    continue;
                }
                events.push(KernelEvent {
                    kind: EventKind::Exec,
                    pid: record.pid,
                    uid: 0,
                    ppid: record.ppid,
                    comm: [0; 16],
                    filename: None,
                    pre_exec: true,
                });
            }
            self.since_seq = last_seq;
            if filled < self.buffer.len() {
                return;
            }
        }
    }
}

struct Record {
    seq: u64,
    pid: Pid,
    ppid: Pid,
}

fn parse_line(line: &str) -> Option<Record> {
    let mut seq = None;
    let mut pid = None;
    let mut ppid = None;
    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "seq" => seq = value.parse().ok(),
            "pid" => pid = value.parse::<Pid>().ok(),
            "ppid" => ppid = value.parse::<Pid>().ok(),
            _ => {}
        }
    }
    Some(Record {
        seq: seq?,
        pid: pid?,
        ppid: ppid?,
    })
}
