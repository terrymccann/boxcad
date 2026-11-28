// Showing and reading lengths. Every stored length is millimeters; inches are
// only ever a way of displaying and typing them.

import type { Units } from "./api";

const MM_PER_INCH = 25.4;

/** A length for display, such as "3 mm" or "0.118 in". */
export function formatLength(mm: number, units: Units, withUnit = true): string {
  const value = units === "in" ? mm / MM_PER_INCH : mm;
  const text = trimZeros(value.toFixed(units === "in" ? 3 : 2));
  return withUnit ? `${text} ${units}` : text;
}

/** Sizes joined for display, such as "200 × 120 × 3 mm". */
export function formatSize(mm: number[], units: Units): string {
  return `${mm.map((v) => formatLength(v, units, false)).join(" × ")} ${units}`;
}

/**
 * Read a typed length and return millimeters, or null if it is not a number.
 * Accepts decimals ("3.2", "3,2"), fractions ("1/8", "1 1/2") and an explicit
 * unit ("3mm", "0.125 in", '1/8"'), which wins over the display units.
 */
export function parseLength(text: string, units: Units): number | null {
  const match = text.trim().toLowerCase().match(/^(.*?)\s*(mm|in|")?$/);
  if (!match) return null;
  const value = parseNumber(match[1]);
  if (value === null) return null;
  const typed: Units = match[2] ? (match[2] === "mm" ? "mm" : "in") : units;
  return typed === "in" ? value * MM_PER_INCH : value;
}

function parseNumber(text: string): number | null {
  const t = text.trim().replace(",", ".");
  const fraction = t.match(/^(?:(\d+)\s+)?(\d+)\s*\/\s*(\d+)$/);
  if (fraction) {
    const [whole, top, bottom] = [fraction[1] ?? "0", fraction[2], fraction[3]].map(Number);
    return bottom === 0 ? null : whole + top / bottom;
  }
  if (!/^-?(\d+\.?\d*|\.\d+)$/.test(t)) return null;
  return Number(t);
}

function trimZeros(text: string): string {
  return text.includes(".") ? text.replace(/\.?0+$/, "") : text;
}
