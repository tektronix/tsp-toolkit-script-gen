export const SCIENTIFIC_PREFIXES: { [key: string]: string } = { //engineering prefix m,u,n  scientific prefix
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

export const REVERSE_PREFIXES: { [key: string]: number } = {
  'p': -12,
  'n': -9,
  'µ': -6,
  'm': -3,
  '': 0,
  'k': 3,
  'M': 6,
  'G': 9,
  'T': 12,
};

export function parseScientificInput(input: string): string {
  // Updated regex to handle optional prefix and spaces
  const regex = /^([\d.]+)(e([+-]?\d+))?\s*([a-zA-Zµ]*)?\s*([a-zA-Z]+)$/;
  const match = input.match(regex);

  if (!match) {
    return 'Invalid input';
  }

  const [_, value, , exponent, prefix, baseUnit] = match;

  // Parse the numeric value
  let numericValue = parseFloat(value) * Math.pow(10, parseInt(exponent || '0'));

  // Handle the case where the numeric value is 0
  if (numericValue === 0) {
    return `0 ${baseUnit}`;
  }

  // Get the exponent of the existing prefix
  const existingExponent = REVERSE_PREFIXES[prefix || ''] || 0;

  // Adjust the numeric value based on the existing prefix
  numericValue *= Math.pow(10, existingExponent);

  // Determine the final exponent for engineering notation
  const finalExponent = Math.floor(Math.log10(Math.abs(numericValue)) / 3) * 3;

  // Scale the numeric value to the final exponent
  const scaledValue = numericValue / Math.pow(10, finalExponent);

  // Round the scaled value to 3 digits after the decimal
  const roundedValue = parseFloat(scaledValue.toFixed(3));

  // Find the appropriate SI prefix for the final exponent
  const newPrefix = finalExponent === 0 ? '' : SCIENTIFIC_PREFIXES[finalExponent] || `e${finalExponent}`;

  // Combine the rounded value, new prefix, and base unit
  return `${roundedValue} ${newPrefix}${baseUnit}`;
}

export function parseToDecimal(input: string): number | null {
  // Updated regex to handle optional prefix and spaces
  const regex = /^([\d.]+)(e([+-]?\d+))?\s*([a-zA-Zµ]*)?\s*([a-zA-Z]+)$/;
  const match = input.match(regex);

  if (!match) {
    return null; // Invalid input
  }

  const [_, value, , exponent, prefix, baseUnit] = match;

  // Parse the numeric value
  let numericValue = parseFloat(value) * Math.pow(10, parseInt(exponent || '0'));

  // Get the exponent of the existing prefix
  const existingExponent = REVERSE_PREFIXES[prefix || ''] || 0;

  // Adjust the numeric value based on the existing prefix
  numericValue *= Math.pow(10, existingExponent);

  // Return the numeric value as a decimal
  return numericValue;
}
