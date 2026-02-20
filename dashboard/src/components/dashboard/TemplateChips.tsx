'use client';

import { SpawnTemplate, SpawnTemplateId } from './types';

interface TemplateChipsProps {
  templates: SpawnTemplate[];
  activeTemplate: SpawnTemplateId;
  onSelect: (template: SpawnTemplate) => void;
}

export function TemplateChips({ templates, activeTemplate, onSelect }: TemplateChipsProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {templates.map((template) => {
        const isActive = template.id === activeTemplate;
        return (
          <button
            key={template.id}
            type="button"
            onClick={() => onSelect(template)}
            className={
              isActive
                ? 'rounded-full border border-emerald-300/60 bg-emerald-400/20 px-3 py-1.5 text-xs font-medium text-emerald-100 transition-colors'
                : 'rounded-full border border-border/70 bg-panel-row px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-panel-hover hover:text-foreground'
            }
          >
            {template.label}
          </button>
        );
      })}
    </div>
  );
}
