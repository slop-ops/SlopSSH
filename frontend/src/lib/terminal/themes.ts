export interface ITheme {
  name: string
  foreground: string
  background: string
  cursor: string
  cursorAccent?: string
  selectionBackground: string
  selectionForeground?: string
  black: string
  red: string
  green: string
  yellow: string
  blue: string
  magenta: string
  cyan: string
  white: string
  brightBlack: string
  brightRed: string
  brightGreen: string
  brightYellow: string
  brightBlue: string
  brightMagenta: string
  brightCyan: string
  brightWhite: string
}

export interface CustomPalette {
  bgPrimary: string
  bgSecondary: string
  bgTertiary: string
  textPrimary: string
  textSecondary: string
  accent: string
  border: string
}

export const tokyoNightTheme: ITheme = {
  name: 'Tokyo Night',
  foreground: '#c0caf5',
  background: '#1a1b26',
  cursor: '#c0caf5',
  cursorAccent: '#1a1b26',
  selectionBackground: '#33467c88',
  black: '#15161e',
  red: '#f7768e',
  green: '#9ece6a',
  yellow: '#e0af68',
  blue: '#7aa2f7',
  magenta: '#bb9af7',
  cyan: '#7dcfff',
  white: '#a9b1d6',
  brightBlack: '#414868',
  brightRed: '#f7768e',
  brightGreen: '#9ece6a',
  brightYellow: '#e0af68',
  brightBlue: '#7aa2f7',
  brightMagenta: '#bb9af7',
  brightCyan: '#7dcfff',
  brightWhite: '#c0caf5',
}

export const catppuccinMochaTheme: ITheme = {
  name: 'Catppuccin Mocha',
  foreground: '#cdd6f4',
  background: '#1e1e2e',
  cursor: '#f5e0dc',
  cursorAccent: '#1e1e2e',
  selectionBackground: '#585b7066',
  black: '#45475a',
  red: '#f38ba8',
  green: '#a6e3a1',
  yellow: '#f9e2af',
  blue: '#89b4fa',
  magenta: '#cba6f7',
  cyan: '#94e2d5',
  white: '#bac2de',
  brightBlack: '#585b70',
  brightRed: '#f38ba8',
  brightGreen: '#a6e3a1',
  brightYellow: '#f9e2af',
  brightBlue: '#89b4fa',
  brightMagenta: '#cba6f7',
  brightCyan: '#94e2d5',
  brightWhite: '#a6adc8',
}

export const jetbrainsDarkTheme: ITheme = {
  name: 'JetBrains Dark',
  foreground: '#dfe1e5',
  background: '#1e1f22',
  cursor: '#dfe1e5',
  cursorAccent: '#1e1f22',
  selectionBackground: '#3574f055',
  black: '#191a1c',
  red: '#fa6675',
  green: '#579c4b',
  yellow: '#f5d867',
  blue: '#3574f0',
  magenta: '#c77dbb',
  cyan: '#3ebbbf',
  white: '#dfe1e5',
  brightBlack: '#4e5157',
  brightRed: '#fa6675',
  brightGreen: '#579c4b',
  brightYellow: '#f5d867',
  brightBlue: '#3574f0',
  brightMagenta: '#c77dbb',
  brightCyan: '#3ebbbf',
  brightWhite: '#ffffff',
}

export const oneDarkTheme: ITheme = {
  name: 'One Dark',
  foreground: '#abb2bf',
  background: '#282c34',
  cursor: '#528bff',
  cursorAccent: '#282c34',
  selectionBackground: '#3e445188',
  black: '#282c34',
  red: '#e06c75',
  green: '#98c379',
  yellow: '#e5c07b',
  blue: '#61afef',
  magenta: '#c678dd',
  cyan: '#56b6c2',
  white: '#abb2bf',
  brightBlack: '#5c6370',
  brightRed: '#e06c75',
  brightGreen: '#98c379',
  brightYellow: '#e5c07b',
  brightBlue: '#61afef',
  brightMagenta: '#c678dd',
  brightCyan: '#56b6c2',
  brightWhite: '#ffffff',
}

export const githubLightTheme: ITheme = {
  name: 'GitHub Light',
  foreground: '#1f2328',
  background: '#ffffff',
  cursor: '#0969da',
  cursorAccent: '#ffffff',
  selectionBackground: '#0969da33',
  black: '#24292f',
  red: '#cf222e',
  green: '#1a7f37',
  yellow: '#9a6700',
  blue: '#0969da',
  magenta: '#8250df',
  cyan: '#1b7c83',
  white: '#6e7781',
  brightBlack: '#57606a',
  brightRed: '#a40e26',
  brightGreen: '#116329',
  brightYellow: '#4d2d00',
  brightBlue: '#0550ae',
  brightMagenta: '#6639ba',
  brightCyan: '#055d64',
  brightWhite: '#24292f',
}

// Backward compatibility exports
export const darkTheme: ITheme = {
  ...jetbrainsDarkTheme,
  name: 'SlopSSH Dark',
  selectionBackground: '#4a90d966',
}

export const lightTheme: ITheme = {
  ...githubLightTheme,
  name: 'SlopSSH Light',
  selectionBackground: '#4a90d966',
}

export const THEME_REGISTRY: Record<string, ITheme> = {
  'tokyo-night': tokyoNightTheme,
  'catppuccin-mocha': catppuccinMochaTheme,
  'jetbrains-dark': jetbrainsDarkTheme,
  'one-dark': oneDarkTheme,
  'github-light': githubLightTheme,
  'dark': jetbrainsDarkTheme,
  'light': githubLightTheme,
}

export function getTerminalTheme(themeId: string, customPalette?: CustomPalette): ITheme {
  if (themeId === 'custom' && customPalette) {
    return {
      name: 'Custom',
      foreground: customPalette.textPrimary || '#dfe1e5',
      background: customPalette.bgPrimary || '#1e1f22',
      cursor: customPalette.accent || '#3574f0',
      cursorAccent: customPalette.bgPrimary || '#1e1f22',
      selectionBackground: `${customPalette.accent || '#3574f0'}44`,
      black: '#191a1c',
      red: '#fa6675',
      green: '#579c4b',
      yellow: '#f5d867',
      blue: customPalette.accent || '#3574f0',
      magenta: '#c77dbb',
      cyan: '#3ebbbf',
      white: customPalette.textPrimary || '#dfe1e5',
      brightBlack: '#4e5157',
      brightRed: '#fa6675',
      brightGreen: '#579c4b',
      brightYellow: '#f5d867',
      brightBlue: customPalette.accent || '#3574f0',
      brightMagenta: '#c77dbb',
      brightCyan: '#3ebbbf',
      brightWhite: '#ffffff',
    }
  }

  return THEME_REGISTRY[themeId] || jetbrainsDarkTheme
}
