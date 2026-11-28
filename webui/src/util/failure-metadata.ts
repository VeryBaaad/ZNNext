import type { FailedProcess } from "../api/modules";
import { formatDateTime } from "../i18n";

export function formatFailureMetadata(failure: FailedProcess): string {
  const parts: string[] = [];
  if (failure.pid !== undefined) parts.push(`pid=${failure.pid}`);
  if (failure.lastSeenMs !== undefined)
    parts.push(formatDateTime(failure.lastSeenMs));
  return parts.join(" · ");
}
