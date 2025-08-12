export const SCIENTIFIC_PREFIXES: Record<string, string> = { //engineering prefix m,u,n  scientific prefix
  '-12': 'p',
  '-9': 'n',
  '-6': 'µ',
  '-3': 'm',
  '0': '',
  '3': 'k',
  '6': 'M',
  '9': 'G',
  '12': 'T',
};

export const REVERSE_PREFIXES: Record<string, number> = {
  'p': -12,
  'n': -9,
  'µ': -6,
  'u': -6,
  'm': -3,
  '': 0,
  'k': 3,
  'M': 6,
  'G': 9,
  'T': 12,
};

export function parseScientificInput(input: string, unit: string | undefined): string {
  const regex = /^(-?[\d.]+)(e([+-]?\d+))?\s*([a-zA-Zµ]?)\s*([a-zA-Z]+)?$/;
  const match = input.match(regex);

  if (!match) {
    return 'Invalid input';
  }

  const [, value, , exponent, prefix, baseUnit] = match;
  let normalizedPrefix = prefix;
  if (normalizedPrefix === 'µ') normalizedPrefix = 'u';

  let numericValue = parseFloat(value) * Math.pow(10, parseInt(exponent || '0'));

  if (numericValue === 0) {
    return `0 ${baseUnit || unit}`;
  }

  const existingExponent = REVERSE_PREFIXES[normalizedPrefix || ''] || 0;
  numericValue *= Math.pow(10, existingExponent);

  const finalExponent = Math.floor(Math.log10(Math.abs(numericValue)) / 3) * 3;
  const scaledValue = numericValue / Math.pow(10, finalExponent);
  const roundedValue = parseFloat(scaledValue.toFixed(3));
  const newPrefix = finalExponent === 0 ? '' : SCIENTIFIC_PREFIXES[finalExponent] || `e${finalExponent}`;

  return `${roundedValue} ${newPrefix}${baseUnit || unit || ''}`;
}

export function parseToDecimal(input: string, unit?: string | undefined): number | null {
  const regex = /^(-?[\d.]+)(e([+-]?\d+))?\s*([a-zA-Zµ]?)\s*([a-zA-Z]+)?$/;
  const match = input.match(regex);

  if (!match) {
    return null; // Invalid input
  }

  const [, value, , exponent, prefix, baseUnit] = match;
  if (baseUnit && baseUnit !== unit) {
    return null; // Incompatible units
  }
  let normalizedPrefix = prefix;
  if (normalizedPrefix === 'µ') normalizedPrefix = 'u';

  let numericValue = parseFloat(value) * Math.pow(10, parseInt(exponent || '0'));
  const existingExponent = REVERSE_PREFIXES[normalizedPrefix || ''] || 0;
  numericValue *= Math.pow(10, existingExponent);

  return numericValue;
}
