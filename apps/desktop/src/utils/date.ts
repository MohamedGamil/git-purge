/**
 * Safely parses a date string into a Date object.
 * Handles various RFC3339 / ISO-8601 date formats, timezone offsets, and space-separator formats.
 * Falls back gracefully to regex-based extraction on failure.
 */
export function parseSafeDate(dateStr: string | null | undefined): Date {
  if (!dateStr) return new Date(NaN);

  // 1. Try standard Date parsing
  let d = new Date(dateStr);
  if (!isNaN(d.getTime())) {
    return d;
  }

  // 2. Normalize and clean the string
  let cleanStr = dateStr.trim();

  // If there's a space separating date and time (e.g. "YYYY-MM-DD HH:mm:ss")
  if (/^\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}/.test(cleanStr)) {
    // Replace the first space with 'T'
    cleanStr = cleanStr.replace(/\s+/, 'T');
    // Remove space before timezone offset (e.g. "2026-07-12T00:04:30 +03:00" -> "2026-07-12T00:04:30+03:00")
    cleanStr = cleanStr.replace(/T(\d{2}:\d{2}:\d{2}(?:\.\d+)?)?\s+([+-]\d{2}:?\d{2}|Z)$/, 'T$1$2');

    d = new Date(cleanStr);
    if (!isNaN(d.getTime())) {
      return d;
    }
  }

  // 3. Fallback: regex-based manual parser
  // Matches: YYYY-MM-DD [T/space] HH:mm:ss [.ssssss] [timezone offset like Z or +HH:MM or +HHMM]
  const isoRegex = /^(\d{4})-(\d{2})-(\d{2})(?:[ T](\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?)?(?:\s*(Z|[+-]\d{2}:?\d{2}))?$/;
  const match = cleanStr.match(isoRegex);
  if (match) {
    const [_, year, month, day, hour = '0', minute = '0', second = '0', ms = '0', offset] = match;

    const y = parseInt(year, 10);
    const m = parseInt(month, 10) - 1; // 0-indexed month
    const dd = parseInt(day, 10);
    const hh = parseInt(hour, 10);
    const min = parseInt(minute, 10);
    const ss = parseInt(second, 10);
    const millisecond = parseInt((ms + '000').substring(0, 3), 10);

    if (offset && offset !== 'Z') {
      const offsetRegex = /^([+-])(\d{2}):?(\d{2})?$/;
      const offsetMatch = offset.match(offsetRegex);
      if (offsetMatch) {
        const [__, sign, offsetHours, offsetMinutes = '0'] = offsetMatch;
        const oHours = parseInt(offsetHours, 10);
        const oMinutes = parseInt(offsetMinutes, 10);
        const totalOffsetMinutes = (oHours * 60 + oMinutes) * (sign === '-' ? -1 : 1);

        const utcTimestamp = Date.UTC(y, m, dd, hh, min, ss, millisecond);
        const correctedTimestamp = utcTimestamp - (totalOffsetMinutes * 60 * 1000);
        return new Date(correctedTimestamp);
      }
    }

    if (offset === 'Z') {
      return new Date(Date.UTC(y, m, dd, hh, min, ss, millisecond));
    }

    // Default to local timezone if no offset specified
    return new Date(y, m, dd, hh, min, ss, millisecond);
  }

  // Return the original invalid date object if we completely fail
  return new Date(dateStr);
}

/**
 * Formats a date string or Date object to a localized date string (e.g. for display in branches).
 */
export function formatLocalDate(dateInput: string | Date | null | undefined): string {
  if (!dateInput) return '';
  const d = dateInput instanceof Date ? dateInput : parseSafeDate(dateInput);
  if (isNaN(d.getTime())) {
    return typeof dateInput === 'string' ? dateInput : '';
  }
  return d.toLocaleDateString();
}

/**
 * Formats a date string or Date object to a localized date and time string (e.g. for backup list).
 */
export function formatLocalDateTime(dateInput: string | Date | null | undefined): string {
  if (!dateInput) return '';
  const d = dateInput instanceof Date ? dateInput : parseSafeDate(dateInput);
  if (isNaN(d.getTime())) {
    return typeof dateInput === 'string' ? dateInput : '';
  }
  return d.toLocaleString();
}

/**
 * Formats a date string or Date object to a localized time string (e.g. for scan times).
 */
export function formatLocalTime(dateInput: string | Date | null | undefined): string {
  if (!dateInput) return '';
  const d = dateInput instanceof Date ? dateInput : parseSafeDate(dateInput);
  if (isNaN(d.getTime())) {
    return typeof dateInput === 'string' ? dateInput : '';
  }
  return d.toLocaleTimeString();
}

/**
 * Formats a date string or Date object for custom chart labeling.
 */
export function formatChartDate(dateInput: string | Date | null | undefined): string {
  if (!dateInput) return '';
  const d = dateInput instanceof Date ? dateInput : parseSafeDate(dateInput);
  if (isNaN(d.getTime())) {
    return typeof dateInput === 'string' ? dateInput : '';
  }
  return d.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric'
  });
}
