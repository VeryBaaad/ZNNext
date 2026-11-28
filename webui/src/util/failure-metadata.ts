import type { FailedProcess } from "../api/modules";
import { locale } from "../i18n";

export function formatFailureMetadata(failure: FailedProcess): string {
  const parts: string[] = [];
  if (failure.pid !== undefined) parts.push(`pid=${failure.pid}`);
  if (failure.lastSeenMs !== undefined) {
    const language = locale.value === "lzh" ? "zh-Hant" : locale.value;
    const time = new Date(failure.lastSeenMs).toLocaleString(language);
    parts.push(time);
  }
  return parts.join(" · ");
}
