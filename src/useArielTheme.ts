import { useColorScheme } from 'react-native';
import { ArielTheme, type ArielThemeLike } from './generated/rn/mermaid_wrapper';
import type { ArielThemeRegistry } from './ArielThemeRegistry';

export function useArielTheme(registry?: ArielThemeRegistry): ArielThemeLike {
  const scheme = useColorScheme() ?? 'light';
  if (registry) {
    try { return registry.getTheme(scheme); }
    catch { /* registry doesn't define 'dark'/'light' — fall through */ }
  }
  return ArielTheme.modern();
}
