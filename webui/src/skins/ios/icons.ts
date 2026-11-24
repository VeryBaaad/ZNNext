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

export interface IconSpec {
  strokes?: string[];
  fills?: string[];
  strokeWidth?: number;
}

export interface ResolvedIcon {
  strokes: string[];
  fills: string[];
  strokeWidth: number;
}

const circle = (cx: number, cy: number, r: number): string =>
  `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${r * 2} 0a${r} ${r} 0 1 0 ${-r * 2} 0`;

const GITHUB_MARK =
  "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 " +
  "0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7 " +
  "c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 " +
  "3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 " +
  "1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 " +
  "1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 " +
  "1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 " +
  "2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12";

export const ICONS = {
  refresh: {
    strokes: ["M19.4 12a7.4 7.4 0 1 1-2.17-5.23", "M19.4 4.4v4.2h-4.2"],
  },
  check: {
    strokes: ["M5 12.7l4.4 4.4L19 7.3"],
    strokeWidth: 2.2,
  },
  sun: {
    strokes: [
      circle(12, 12, 3.7),
      "M12 2.9v2.2",
      "M12 18.9v2.2",
      "M2.9 12h2.2",
      "M18.9 12h2.2",
      "M5.6 5.6l1.55 1.55",
      "M16.85 16.85l1.55 1.55",
      "M18.4 5.6l-1.55 1.55",
      "M7.15 16.85l-1.55 1.55",
    ],
    strokeWidth: 1.7,
  },
  moon: {
    fills: ["M20.4 15.1A8.8 8.8 0 0 1 8.9 3.6a8.9 8.9 0 1 0 11.5 11.5z"],
  },
  auto: {
    strokes: [circle(12, 12, 8.2)],
    fills: ["M12 3.8a8.2 8.2 0 0 1 0 16.4z"],
    strokeWidth: 1.7,
  },
  globe: {
    strokes: [
      circle(12, 12, 8.4),
      "M3.6 12h16.8",
      "M12 3.6c2.4 2.4 3.6 5.2 3.6 8.4s-1.2 6-3.6 8.4c-2.4-2.4-3.6-5.2-3.6-8.4S9.6 6 12 3.6z",
    ],
    strokeWidth: 1.6,
  },
  warning: {
    strokes: ["M12 3.7l9.1 16H2.9z", "M12 9.2v4.9", "M12 17.4h.01"],
    strokeWidth: 1.8,
  },
  github: {
    strokes: [],
    fills: [GITHUB_MARK],
  },
} satisfies Record<string, IconSpec>;

export type IconName = keyof typeof ICONS;

export function resolveIcon(name: IconName): ResolvedIcon {
  const spec: IconSpec = ICONS[name];
  return {
    strokes: spec.strokes ?? [],
    fills: spec.fills ?? [],
    strokeWidth: spec.strokeWidth ?? 1.8,
  };
}
