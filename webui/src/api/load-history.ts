/*
 * SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
 *
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

import { execText } from "./ksu";

export interface LoadRecord {
  time: string;
  library: string;
  loaded: boolean;
  elapsedUs: number;
}

export async function readLoadHistory(): Promise<LoadRecord[]> {
  const logs = await execText(
    "set -o pipefail; /system/bin/logcat -b main -d -v time ZNNextLoad:I '*:S' | /system/bin/tail -n 50",
    15000,
  );
  const records: LoadRecord[] = [];
  const entry =
    /^\d{2}-\d{2} ((?:[01]\d|2[0-3]):[0-5]\d):[0-5]\d\.\d+ I\/ZNNextLoad\s*\(\s*\d+\): (loaded|failed) (\d+) ([^/\x00-\x1f\x7f]+)$/;
  for (const line of logs.split(/\r?\n/)) {
    const match = entry.exec(line);
    if (!match) continue;
    const [, time, result, duration, library] = match;
    const elapsedUs = Number(duration);
    if (Number.isSafeInteger(elapsedUs)) {
      records.push({ time, library, loaded: result === "loaded", elapsedUs });
    }
  }
  return records.reverse();
}
