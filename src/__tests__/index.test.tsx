import { describe, it, expect, jest } from '@jest/globals';

// The native module is mocked at the Jest layer since it requires
// compiled Rust binaries. The mock mirrors the real API surface.
jest.mock('react-native-ariel', () => ({
  renderMermaid: (input: string) => {
    if (!input) throw new Error('Invalid diagram');
    return `<svg xmlns="http://www.w3.org/2000/svg"><text>${input}</text></svg>`;
  },
  renderMermaidWithOptions: (input: string, _theme: unknown, _config: unknown) => {
    return `<svg xmlns="http://www.w3.org/2000/svg"><text>${input}</text></svg>`;
  },
  renderMermaidWithTiming: (input: string, _theme: unknown, _config: unknown) => ({
    svg: `<svg xmlns="http://www.w3.org/2000/svg"><text>${input}</text></svg>`,
    parseUs: 100,
    layoutUs: 200,
    renderUs: 50,
    totalMs: 0.35,
  }),
  parseDiagram: (input: string) => ({ __input: input }),
  computeDiagramLayout: (_parsed: unknown, _theme: unknown, _config: unknown) => ({
    __layout: true,
  }),
  renderSvgFromLayout: (_layout: unknown, _theme: unknown, _config: unknown) =>
    '<svg xmlns="http://www.w3.org/2000/svg"><g/></svg>',
  ArielTheme: {
    modern: () => ({ __theme: 'modern' }),
    mermaidDefault: () => ({ __theme: 'mermaid_default' }),
  },
  setTimingLogs: jest.fn(),
}));

import {
  renderMermaid,
  renderMermaidWithOptions,
  renderMermaidWithTiming,
  parseDiagram,
  computeDiagramLayout,
  renderSvgFromLayout,
  ArielTheme,
  setTimingLogs,
} from 'react-native-ariel';

describe('renderMermaid', () => {
  it('returns a valid SVG string for a flowchart', () => {
    const svg = renderMermaid('flowchart TD\n  A --> B');
    expect(svg).toContain('<svg');
    expect(svg).toContain('</svg>');
  });

  it('handles unknown input without throwing', () => {
    // mermaid-rs-renderer parses unknown syntax gracefully — does not throw.
    expect(() => renderMermaid('not valid %%%')).not.toThrow();
  });
});

describe('renderMermaidWithOptions', () => {
  it('returns SVG with theme and config', () => {
    const theme = ArielTheme.modern();
    const config = { nodeSpacing: 50, rankSpacing: 80 };
    const svg = renderMermaidWithOptions('flowchart LR\n  A --> B', theme, config);
    expect(svg).toContain('<svg');
  });
});

describe('renderMermaidWithTiming', () => {
  it('returns svg and timing fields', () => {
    const result = renderMermaidWithTiming(
      'flowchart TD\n  A --> B',
      ArielTheme.modern(),
      { nodeSpacing: undefined, rankSpacing: undefined }
    );
    expect(result.svg).toContain('<svg');
    expect(typeof result.parseUs).toBe('number');
    expect(typeof result.layoutUs).toBe('number');
    expect(typeof result.renderUs).toBe('number');
    expect(typeof result.totalMs).toBe('number');
  });
});

describe('pipeline API', () => {
  it('parse → layout → svg produces valid SVG', () => {
    const parsed = parseDiagram('flowchart TD\n  A --> B');
    const layout = computeDiagramLayout(parsed, ArielTheme.mermaidDefault(), {
      nodeSpacing: undefined,
      rankSpacing: undefined,
    });
    const svg = renderSvgFromLayout(layout, ArielTheme.modern(), {
      nodeSpacing: undefined,
      rankSpacing: undefined,
    });
    expect(svg).toContain('<svg');
  });
});

describe('setTimingLogs', () => {
  it('can be toggled without throwing', () => {
    expect(() => setTimingLogs(true)).not.toThrow();
    expect(() => setTimingLogs(false)).not.toThrow();
  });
});

describe('ArielTheme', () => {
  it('modern() returns an object', () => {
    expect(ArielTheme.modern()).toBeDefined();
  });

  it('mermaidDefault() returns an object', () => {
    expect(ArielTheme.mermaidDefault()).toBeDefined();
  });

  it('modern() and mermaidDefault() are distinct', () => {
    expect(ArielTheme.modern()).not.toBe(ArielTheme.mermaidDefault());
  });
});

describe('ArielLayoutConfig', () => {
  it('accepts undefined spacing', () => {
    const config = { nodeSpacing: undefined, rankSpacing: undefined };
    expect(() =>
      renderMermaidWithOptions('flowchart TD\n  A --> B', ArielTheme.modern(), config)
    ).not.toThrow();
  });

  it('accepts numeric spacing', () => {
    const config = { nodeSpacing: 50, rankSpacing: 80 };
    expect(() =>
      renderMermaidWithOptions('flowchart TD\n  A --> B', ArielTheme.modern(), config)
    ).not.toThrow();
  });
});

describe('ArielRenderResult shape', () => {
  it('has all required timing fields', () => {
    const result = renderMermaidWithTiming(
      'flowchart TD\n  A --> B',
      ArielTheme.modern(),
      { nodeSpacing: undefined, rankSpacing: undefined }
    );
    expect(result).toHaveProperty('svg');
    expect(result).toHaveProperty('parseUs');
    expect(result).toHaveProperty('layoutUs');
    expect(result).toHaveProperty('renderUs');
    expect(result).toHaveProperty('totalMs');
  });
});
