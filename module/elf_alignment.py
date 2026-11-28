# SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
#
# This file is part of ZNNext.
#
# ZNNext is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# ZNNext is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
#
# Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>

import struct


PAGE_SIZE = 16 * 1024


def load_segments_aligned_16k(path):
    with path.open("rb") as stream:
        ident = stream.read(16)
        if len(ident) != 16 or ident[:4] != b"\x7fELF" or ident[6] != 1:
            return None
        order = {1: "<", 2: ">"}.get(ident[5])
        if order is None or ident[4] not in (1, 2):
            return None
        if ident[4] == 1:
            header = struct.Struct(order + "HHIIIIIHHHHHH")
            segment = struct.Struct(order + "III4xII4xI")
        else:
            header = struct.Struct(order + "HHIQQQIHHHHHH")
            segment = struct.Struct(order + "I4xQQ8xQQQ")
        data = stream.read(header.size)
        if len(data) != header.size:
            return None
        (
            elf_type, _, version, _, table, _, _, header_size, stride, count, *_,
        ) = header.unpack(data)
        if elf_type not in (2, 3) or version != 1 or header_size != 16 + header.size:
            return None
        file_size = stream.seek(0, 2)
        if not 0 < count < 0xFFFF or stride < segment.size or table < header_size:
            return None
        if table + stride * count > file_size:
            return None

        found_load = False
        aligned = True
        for index in range(count):
            stream.seek(table + index * stride)
            data = stream.read(segment.size)
            kind, offset, address, disk_size, memory_size, alignment = segment.unpack(data)
            if kind != 1:
                continue
            found_load = True
            if disk_size > memory_size or (disk_size and offset + disk_size > file_size):
                return None
            if alignment & (alignment - 1):
                return None
            if alignment < PAGE_SIZE or (address - offset) % PAGE_SIZE:
                aligned = False
        return aligned if found_load else None
