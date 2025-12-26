/**
 * Language Detection Utilities
 *
 * Enhanced language detection for syntax highlighting
 */

import type { LanguageDetection } from '../types/diff';

/**
 * Comprehensive language map by file extension
 */
const LANGUAGE_EXTENSIONS: Record<string, string> = {
  // JavaScript/TypeScript
  ts: 'typescript',
  tsx: 'typescript',
  js: 'javascript',
  jsx: 'javascript',
  mjs: 'javascript',
  cjs: 'javascript',

  // Systems
  rs: 'rust',
  go: 'go',
  c: 'c',
  cpp: 'cpp',
  cc: 'cpp',
  cxx: 'cpp',
  h: 'c',
  hpp: 'cpp',
  cs: 'csharp',

  // Scripting
  py: 'python',
  rb: 'ruby',
  php: 'php',
  pl: 'perl',
  lua: 'lua',

  // JVM
  java: 'java',
  kt: 'kotlin',
  scala: 'scala',
  groovy: 'groovy',

  // Mobile
  swift: 'swift',
  m: 'objective-c',
  mm: 'objective-c',
  dart: 'dart',

  // Functional
  hs: 'haskell',
  elm: 'elm',
  ex: 'elixir',
  exs: 'elixir',
  erl: 'erlang',
  ml: 'ocaml',
  fs: 'fsharp',

  // Shell
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  fish: 'bash',

  // Markup/Data
  md: 'markdown',
  markdown: 'markdown',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  xml: 'xml',
  html: 'html',
  htm: 'html',
  svg: 'xml',

  // Styles
  css: 'css',
  scss: 'scss',
  sass: 'sass',
  less: 'less',
  stylus: 'stylus',

  // Config
  dockerfile: 'dockerfile',
  makefile: 'makefile',
  cmake: 'cmake',
  gradle: 'gradle',
  properties: 'properties',
  ini: 'ini',
  conf: 'conf',
  env: 'bash',

  // Database
  sql: 'sql',
  psql: 'sql',
  mysql: 'sql',

  // Other
  graphql: 'graphql',
  gql: 'graphql',
  proto: 'protobuf',
  tf: 'terraform',
  hcl: 'hcl',
  r: 'r',
  jl: 'julia',
  vim: 'vim',
};

/**
 * Special filename patterns that override extension detection
 */
const SPECIAL_FILENAMES: Record<string, string> = {
  Dockerfile: 'dockerfile',
  'Dockerfile.dev': 'dockerfile',
  'Dockerfile.prod': 'dockerfile',
  Makefile: 'makefile',
  Rakefile: 'ruby',
  Gemfile: 'ruby',
  'Gemfile.lock': 'ruby',
  Podfile: 'ruby',
  Vagrantfile: 'ruby',
  '.bashrc': 'bash',
  '.bash_profile': 'bash',
  '.zshrc': 'bash',
  '.gitignore': 'plaintext',
  '.dockerignore': 'plaintext',
  '.env': 'bash',
  '.env.example': 'bash',
  'package.json': 'json',
  'tsconfig.json': 'json',
  'cargo.toml': 'toml',
  'pyproject.toml': 'toml',
};

/**
 * Detect programming language from file path with confidence score
 */
export function detectLanguage(filePath: string): LanguageDetection {
  const fileName = filePath.split('/').pop() || '';
  const extension = fileName.split('.').pop()?.toLowerCase() || '';

  // Check special filenames first
  if (SPECIAL_FILENAMES[fileName]) {
    return {
      language: SPECIAL_FILENAMES[fileName],
      confidence: 1.0,
      fileExtension: extension,
    };
  }

  // Check extension
  const language = LANGUAGE_EXTENSIONS[extension];
  if (language) {
    return {
      language,
      confidence: 0.9,
      fileExtension: extension,
    };
  }

  // Default to plaintext
  return {
    language: 'plaintext',
    confidence: 0.5,
    fileExtension: extension,
  };
}

/**
 * Get syntax highlighting class for a language
 */
export function getSyntaxHighlightClass(language: string): string {
  return `language-${language}`;
}

/**
 * Check if language supports syntax highlighting
 */
export function supportsHighlighting(language: string): boolean {
  return language !== 'plaintext' && language !== 'unknown';
}

/**
 * Get human-readable language name
 */
export function getLanguageDisplayName(language: string): string {
  const displayNames: Record<string, string> = {
    typescript: 'TypeScript',
    javascript: 'JavaScript',
    rust: 'Rust',
    python: 'Python',
    go: 'Go',
    java: 'Java',
    cpp: 'C++',
    csharp: 'C#',
    ruby: 'Ruby',
    php: 'PHP',
    swift: 'Swift',
    kotlin: 'Kotlin',
    scala: 'Scala',
    bash: 'Shell',
    markdown: 'Markdown',
    json: 'JSON',
    yaml: 'YAML',
    xml: 'XML',
    html: 'HTML',
    css: 'CSS',
    scss: 'SCSS',
    sql: 'SQL',
    dockerfile: 'Dockerfile',
    plaintext: 'Plain Text',
  };

  return displayNames[language] || language.toUpperCase();
}
