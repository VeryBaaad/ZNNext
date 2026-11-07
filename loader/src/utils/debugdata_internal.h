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

#pragma once

#include <stddef.h>
#include <stdint.h>

#include <vector>

namespace znn::debugdata {

constexpr size_t kMaxDecompressedSize = 64u * 1024u * 1024u;

constexpr size_t kInitialOutputSize = 256u * 1024u;

bool lzmaAlone(const uint8_t* data, size_t size, std::vector<uint8_t>& out);

bool xzStream(const uint8_t* data, size_t size, std::vector<uint8_t>& out);

bool archiveStream(const uint8_t* data, size_t size, std::vector<uint8_t>& out);

}  // namespace znn::debugdata
