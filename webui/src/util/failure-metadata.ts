import type { FailedProcess, ZnnModule } from "../api/modules";
import { formatDateTime } from "../i18n";

export function formatFailureMetadata(failure: FailedProcess): string {
  const parts: string[] = [];
  if (failure.pid !== undefined) parts.push(`pid=${failure.pid}`);
  if (failure.lastSeenMs !== undefined)
    parts.push(formatDateTime(failure.lastSeenMs));
  return parts.join(" · ");
}

export function formatFailureDetails(
  module: ZnnModule,
  failure: FailedProcess,
): string {
  const lines = [`module=${module.id} (${module.name})`];
  if (module.version) lines.push(`version=${module.version}`);
  lines.push(`process=${failure.name}`);
  if (failure.pid !== undefined) lines.push(`pid=${failure.pid}`);
  if (failure.lastSeenMs !== undefined)
    lines.push(`lastSeen=${new Date(failure.lastSeenMs).toISOString()}`);
  lines.push(`reason=${failure.reason}`);
  return lines.join("\n");
}
