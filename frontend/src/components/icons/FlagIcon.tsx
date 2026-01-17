import * as React from 'react';

/**
 * Maps language codes to country codes for flag display
 * Some languages map to specific regional variants (e.g., pt-BR -> BR)
 */
const LANGUAGE_TO_COUNTRY_CODE: Record<string, string> = {
  // Simple codes (19 languages)
  en: 'US', // English -> United States
  fr: 'FR', // French -> France
  de: 'DE', // German -> Germany
  it: 'IT', // Italian -> Italy
  ru: 'RU', // Russian -> Russia
  ja: 'JP', // Japanese -> Japan
  ko: 'KR', // Korean -> South Korea
  ar: 'SA', // Arabic -> Saudi Arabia
  he: 'IL', // Hebrew -> Israel
  hi: 'IN', // Hindi -> India
  nl: 'NL', // Dutch -> Netherlands
  pl: 'PL', // Polish -> Poland
  sr: 'RS', // Serbian -> Serbia
  th: 'TH', // Thai -> Thailand
  tr: 'TR', // Turkish -> Turkey
  sv: 'SE', // Swedish -> Sweden
  da: 'DK', // Danish -> Denmark
  fi: 'FI', // Finnish -> Finland
  vi: 'VN', // Vietnamese -> Vietnam
  no: 'NO', // Norwegian -> Norway
  cs: 'CZ', // Czech -> Czech Republic

  // Regional variants (6 languages)
  'pt-BR': 'BR', // Portuguese (Brazil)
  'zh-CN': 'CN', // Chinese (Simplified) -> China
  'es-ES': 'ES', // Spanish (Spain)
  'es-MX': 'MX', // Spanish (Mexico)

  // Legacy fallbacks (for backward compatibility)
  'en-US': 'US',
  'en-GB': 'GB',
  'pt-PT': 'PT',
  'zh-TW': 'TW',
};

/**
 * Converts a country code to Unicode regional indicator symbols
 * Example: "US" -> "🇺🇸" (U+1F1FA + U+1F1F8)
 */
const getCountryFlag = (countryCode: string): string => {
  if (!countryCode || countryCode.length !== 2) {
    return '🏳️'; // White flag as fallback
  }

  const codePoints = countryCode
    .toUpperCase()
    .split('')
    .map((char) => 127397 + char.charCodeAt(0));

  return String.fromCodePoint(...codePoints);
};

export interface FlagIconProps {
  /** Language code (e.g., "en", "es-ES", "pt-BR") */
  languageCode: string;
  /** Optional custom country code override */
  countryCode?: string;
  /** CSS class name */
  className?: string;
  /** Accessible label for screen readers */
  ariaLabel?: string;
}

/**
 * FlagIcon component renders a country flag emoji for a given language code
 * Uses Unicode regional indicator symbols for performant rendering
 *
 * @example
 * <FlagIcon languageCode="en" ariaLabel="English (United States)" />
 * <FlagIcon languageCode="pt-BR" ariaLabel="Portuguese (Brazil)" />
 * <FlagIcon languageCode="es-ES" countryCode="MX" ariaLabel="Spanish (Mexico)" />
 */
export const FlagIcon: React.FC<FlagIconProps> = ({
  languageCode,
  countryCode,
  className = '',
  ariaLabel,
}) => {
  const resolvedCountryCode =
    countryCode || LANGUAGE_TO_COUNTRY_CODE[languageCode] || languageCode.split('-')[1] || 'US';

  const flag = React.useMemo(() => getCountryFlag(resolvedCountryCode), [resolvedCountryCode]);

  return (
    <span
      role="img"
      aria-label={ariaLabel || `Flag for ${languageCode}`}
      className={`inline-block text-lg leading-none ${className}`}
      style={{ fontFamily: 'system-ui, -apple-system' }} // Ensures flag emoji renders correctly
    >
      {flag}
    </span>
  );
};

export default FlagIcon;
