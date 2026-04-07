import { describe, it, expect } from 'vitest';
import { formatCurrency, formatDate, formatNumber, formatPercent } from './format';

describe('formatCurrency', () => {
  it('formats COP currency correctly', () => {
    const result = formatCurrency(1250000);
    // es-CO format includes space after currency symbol
    expect(result).toContain('1.250.000');
    expect(result).toContain('$');
  });

  it('handles zero', () => {
    const result = formatCurrency(0);
    expect(result).toContain('0');
    expect(result).toContain('$');
  });

  it('handles negative values', () => {
    const result = formatCurrency(-50000);
    expect(result).toContain('50.000');
    expect(result).toContain('-');
  });

  it('rounds decimal values', () => {
    const result = formatCurrency(1500.5);
    // Should round to 1501 (no decimals for COP)
    expect(result).toContain('1.501');
  });
});

describe('formatDate', () => {
  it('formats date in DD/MM/YYYY format', () => {
    // Use explicit date with time to avoid timezone issues
    const date = new Date(2024, 0, 15); // January 15, 2024
    expect(formatDate(date)).toBe('15/01/2024');
  });

  it('formats string date', () => {
    // Use ISO format with explicit time to avoid timezone issues
    const result = formatDate('2024-12-25T12:00:00');
    expect(result).toBe('25/12/2024');
  });
});

describe('formatNumber', () => {
  it('formats number with thousands separator', () => {
    expect(formatNumber(1000000)).toBe('1.000.000');
  });
});

describe('formatPercent', () => {
  it('formats percentage correctly', () => {
    const result = formatPercent(75);
    expect(result).toContain('75');
    expect(result).toContain('%');
  });
});
