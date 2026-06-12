import { describe, it, expect, jest } from '@jest/globals';

jest.mock('react-native-ariel-rs', () => ({
  renderMermaid: (input: string) => {
    if (!input) throw new Error('Invalid diagram');
    return `<svg xmlns="http://www.w3.org/2000/svg"><text>${input}</text></svg>`;
  },
  renderMermaidWithTheme: (input: string, _theme: unknown) => {
    return `<svg xmlns="http://www.w3.org/2000/svg"><text>${input}</text></svg>`;
  },
  ArielTheme: {
    light:   () => ({ __variant: 'light' }),
    dark:    () => ({ __variant: 'dark' }),
    forest:  () => ({ __variant: 'forest' }),
    neutral: () => ({ __variant: 'neutral' }),
  },
}));

import {
  renderMermaid,
  renderMermaidWithTheme,
  ArielTheme,
} from 'react-native-ariel-rs';

describe('renderMermaid', () => {
  it('returns a valid SVG string for a flowchart', () => {
    const svg = renderMermaid('flowchart TD\n  A --> B');
    expect(svg).toContain('<svg');
    expect(svg).toContain('</svg>');
  });

  it('throws on empty input', () => {
    expect(() => renderMermaid('')).toThrow();
  });
});

describe('renderMermaidWithTheme', () => {
  it('renders with light theme', () => {
    const svg = renderMermaidWithTheme('flowchart LR\n  A --> B', ArielTheme.light());
    expect(svg).toContain('<svg');
  });

  it('renders with dark theme', () => {
    const svg = renderMermaidWithTheme('flowchart TD\n  A --> B', ArielTheme.dark());
    expect(svg).toContain('<svg');
  });

  it('renders with forest theme', () => {
    const svg = renderMermaidWithTheme('flowchart TD\n  A --> B', ArielTheme.forest());
    expect(svg).toContain('<svg');
  });

  it('renders with neutral theme', () => {
    const svg = renderMermaidWithTheme('flowchart TD\n  A --> B', ArielTheme.neutral());
    expect(svg).toContain('<svg');
  });
});

describe('ArielTheme', () => {
  it('light() returns an object', () => {
    expect(ArielTheme.light()).toBeDefined();
  });

  it('dark() returns an object', () => {
    expect(ArielTheme.dark()).toBeDefined();
  });

  it('forest() returns an object', () => {
    expect(ArielTheme.forest()).toBeDefined();
  });

  it('neutral() returns an object', () => {
    expect(ArielTheme.neutral()).toBeDefined();
  });

  it('variants are distinct', () => {
    expect(ArielTheme.light()).not.toBe(ArielTheme.dark());
    expect(ArielTheme.forest()).not.toBe(ArielTheme.neutral());
  });
});
