export function sanitizeNumbers(
  obj: Record<string, unknown>
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v === "string") {
      const n = Number(v);
      out[k] = Number.isFinite(n) ? n : 0;
    } else if (typeof v === "object" && v !== null) {
      out[k] = sanitizeNumbers(v as Record<string, unknown>);
    } else {
      out[k] = v;
    }
  }
  return out;
}
