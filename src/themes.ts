import type { ArielThemeConfig } from './generated/rn/mermaid_wrapper';

export const ArielLightTheme: ArielThemeConfig = {
  // Nodes
  nodeColor:            '#F8FAFC',
  secondaryColor:       '#E2E8F0',
  tertiaryColor:        '#FFFFFF',
  // Node border
  nodeBorderColor:      '#94A3B8',
  clusterBorder:        '#CBD5E1',
  // Arrows
  arrowColor:           '#64748B',
  edgeLabelBackground:  '#FFFFFF',
  // Text
  textColor:            '#0F172A',
  // Background — omitted so from_config() defaults to transparent.
  // Pass background: '#FFFFFF' explicitly if you need a solid canvas.
  clusterBackground:    '#F1F5F9',
  // Sequence
  sequenceActorFill:    '#F8FAFC',
  sequenceActorBorder:  '#94A3B8',
  sequenceActorLine:    '#64748B',
  sequenceNoteFill:     '#FFF7ED',
  sequenceNoteBorder:   '#FDBA74',
  sequenceActivationFill:   '#E2E8F0',
  sequenceActivationBorder: '#94A3B8',
  // Typography
  fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif',
  fontSize:   14,
};

export const ArielDarkTheme: ArielThemeConfig = {
  // Nodes
  nodeColor:            '#1E293B',
  secondaryColor:       '#334155',
  tertiaryColor:        '#0F172A',
  // Node border
  nodeBorderColor:      '#475569',
  clusterBorder:        '#475569',
  // Arrows
  arrowColor:           '#94A3B8',
  edgeLabelBackground:  '#1E293B',
  // Text
  textColor:            '#F1F5F9',
  // Background — omitted so from_config() defaults to transparent.
  // Pass background: '#0F172A' explicitly if you need a solid dark canvas.
  clusterBackground:    '#1E293B',
  // Sequence
  sequenceActorFill:    '#1E293B',
  sequenceActorBorder:  '#475569',
  sequenceActorLine:    '#64748B',
  sequenceNoteFill:     '#1C2033',
  sequenceNoteBorder:   '#475569',
  sequenceActivationFill:   '#334155',
  sequenceActivationBorder: '#475569',
  // Typography
  fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif',
  fontSize:   14,
};
