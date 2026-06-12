import { useColorScheme } from 'react-native';
import { ArielTheme, type ArielThemeLike } from './generated/rn/mermaid_wrapper';

export function useArielTheme(): ArielThemeLike {
  const scheme = useColorScheme();
  return scheme === 'dark' ? ArielTheme.dark() : ArielTheme.light();
}
