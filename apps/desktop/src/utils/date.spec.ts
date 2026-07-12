import { describe, it, expect } from 'vitest';
import {
  parseSafeDate,
  formatLocalDate,
  formatLocalDateTime,
  formatLocalTime,
  formatChartDate,
} from './date';

describe('Date Utilities', () => {
  describe('parseSafeDate', () => {
    it('should parse standard ISO-8601 / RFC3339 UTC date strings', () => {
      const d = parseSafeDate('2026-07-12T00:04:30Z');
      expect(d.getTime()).not.toBeNaN();
      expect(d.getUTCFullYear()).toBe(2026);
      expect(d.getUTCMonth()).toBe(6); // 0-indexed July
      expect(d.getUTCDate()).toBe(12);
      expect(d.getUTCHours()).toBe(0);
      expect(d.getUTCMinutes()).toBe(4);
      expect(d.getUTCSeconds()).toBe(30);
    });

    it('should parse date strings with timezone offsets (e.g. +03:00)', () => {
      const d = parseSafeDate('2026-07-12T00:04:30+03:00');
      expect(d.getTime()).not.toBeNaN();
      // UTC time should be 3 hours behind local time
      expect(d.getUTCFullYear()).toBe(2026);
      expect(d.getUTCMonth()).toBe(6);
      expect(d.getUTCDate()).toBe(11);
      expect(d.getUTCHours()).toBe(21);
      expect(d.getUTCMinutes()).toBe(4);
      expect(d.getUTCSeconds()).toBe(30);
    });

    it('should parse Rust/Chrono formats with space and timezone offsets', () => {
      const d = parseSafeDate('2026-07-12 00:04:30 +03:00');
      expect(d.getTime()).not.toBeNaN();
      expect(d.getUTCFullYear()).toBe(2026);
      expect(d.getUTCMonth()).toBe(6);
      expect(d.getUTCDate()).toBe(11);
      expect(d.getUTCHours()).toBe(21);
      expect(d.getUTCMinutes()).toBe(4);
      expect(d.getUTCSeconds()).toBe(30);
    });

    it('should parse dates with fractional seconds and microsecond precision', () => {
      const d = parseSafeDate('2026-07-12T00:04:30.123456Z');
      expect(d.getTime()).not.toBeNaN();
      expect(d.getUTCMilliseconds()).toBe(123);
    });

    it('should parse dates with spaces and fractional seconds and offsets', () => {
      const d = parseSafeDate('2026-07-12 00:04:30.987654 -0500');
      expect(d.getTime()).not.toBeNaN();
      // -0500 means local is 5 hours behind UTC, so UTC is 5 hours ahead: 00:04:30 + 5 hours = 05:04:30
      expect(d.getUTCFullYear()).toBe(2026);
      expect(d.getUTCMonth()).toBe(6);
      expect(d.getUTCDate()).toBe(12);
      expect(d.getUTCHours()).toBe(5);
      expect(d.getUTCMinutes()).toBe(4);
      expect(d.getUTCSeconds()).toBe(30);
      expect(d.getUTCMilliseconds()).toBe(987);
    });

    it('should return an invalid date on invalid string', () => {
      const d = parseSafeDate('not-a-date');
      expect(d.getTime()).toBeNaN();
    });

    it('should handle null or undefined input', () => {
      expect(parseSafeDate(null).getTime()).toBeNaN();
      expect(parseSafeDate(undefined).getTime()).toBeNaN();
    });
  });

  describe('Formatting Helpers', () => {
    it('should format date using browser locale fallback when format is "locale"', () => {
      localStorage.setItem('gitpurge-date-format', 'locale');
      const dateStr = '2026-07-12T00:04:30Z';
      expect(formatLocalDate(dateStr)).toBe(new Date(dateStr).toLocaleDateString());
      expect(formatLocalDateTime(dateStr)).toBe(new Date(dateStr).toLocaleString());
      expect(formatLocalTime(dateStr)).toBe(new Date(dateStr).toLocaleTimeString());
    });

    it('should format date using custom pattern "YYYY-MM-DD HH:mm:ss"', () => {
      localStorage.setItem('gitpurge-date-format', 'YYYY-MM-DD HH:mm:ss');
      const d = new Date(2026, 6, 12, 14, 30, 45); // July 12, 2026, 14:30:45
      expect(formatLocalDate(d)).toBe('2026-07-12');
      expect(formatLocalDateTime(d)).toBe('2026-07-12 14:30:45');
      expect(formatLocalTime(d)).toBe('14:30:45');
    });

    it('should format date using default pattern "YYYY-MM-DD h:m a"', () => {
      localStorage.setItem('gitpurge-date-format', 'YYYY-MM-DD h:m a');
      const d = new Date(2026, 6, 12, 9, 5, 6); // July 12, 2026, 09:05:06
      expect(formatLocalDate(d)).toBe('2026-07-12');
      expect(formatLocalDateTime(d)).toBe('2026-07-12 9:5 am');
      expect(formatLocalTime(d)).toBe('9:5 am');

      const d2 = new Date(2026, 6, 12, 14, 15, 0); // July 12, 2026, 14:15:00
      expect(formatLocalDateTime(d2)).toBe('2026-07-12 2:15 pm');
    });

    it('should format date using custom pattern "MM/DD/YYYY h:m a"', () => {
      localStorage.setItem('gitpurge-date-format', 'MM/DD/YYYY h:m a');
      const d = new Date(2026, 6, 12, 14, 30, 0);
      expect(formatLocalDate(d)).toBe('07/12/2026');
      expect(formatLocalDateTime(d)).toBe('07/12/2026 2:30 pm');
    });

    it('should format date for custom chart labeling', () => {
      const dateStr = '2026-07-12T00:04:30Z';
      const formatted = formatChartDate(dateStr);
      expect(formatted).toBe(new Date(dateStr).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric'
      }));
    });

    it('should return fallback if input is invalid string', () => {
      expect(formatLocalDate('invalid')).toBe('invalid');
      expect(formatLocalDateTime('invalid')).toBe('invalid');
      expect(formatLocalTime('invalid')).toBe('invalid');
      expect(formatChartDate('invalid')).toBe('invalid');
    });

    it('should handle empty input', () => {
      expect(formatLocalDate(null)).toBe('');
      expect(formatLocalDateTime(undefined)).toBe('');
      expect(formatLocalTime(null)).toBe('');
      expect(formatChartDate(null)).toBe('');
    });
  });
});
