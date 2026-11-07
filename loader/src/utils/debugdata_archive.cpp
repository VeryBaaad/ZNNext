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

#include <archive.h>
#include <archive_entry.h>

#include "debugdata_internal.h"

namespace znn::debugdata {

bool archiveStream(const uint8_t* data, size_t size,
                   std::vector<uint8_t>& out) {
  if (!data || size == 0) return false;

  archive* reader = archive_read_new();
  if (!reader) return false;

  archive_read_support_filter_gzip(reader);
  archive_read_support_filter_compress(reader);
  archive_read_support_format_raw(reader);

  bool done = false;
  if (archive_read_open_memory(reader, data, size) == ARCHIVE_OK &&
      archive_filter_code(reader, 0) != ARCHIVE_FILTER_NONE) {
    archive_entry* entry = nullptr;
    if (archive_read_next_header(reader, &entry) == ARCHIVE_OK) {
      uint8_t chunk[32u * 1024u];
      size_t used = 0;
      done = true;

      while (done) {
        const la_ssize_t read = archive_read_data(reader, chunk, sizeof(chunk));
        if (read < 0) {
          done = false;
          break;
        }
        if (read == 0) break;

        const size_t count = static_cast<size_t>(read);
        if (used + count > kMaxDecompressedSize) {
          done = false;
          break;
        }
        out.insert(out.end(), chunk, chunk + count);
        used += count;
      }

      if (done && used == 0) done = false;
    }
  }

  archive_read_free(reader);
  return done;
}

}  // namespace znn::debugdata
