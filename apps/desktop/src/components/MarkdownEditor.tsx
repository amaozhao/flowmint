type MarkdownEditorProps = {
  id: string;
  label: string;
  value: string;
  rows?: number;
  onChange: (value: string) => void;
};

export function MarkdownEditor({ id, label, value, rows = 14, onChange }: MarkdownEditorProps) {
  return (
    <label className="field markdown-field" htmlFor={id}>
      <span>{label}</span>
      <textarea
        id={id}
        className="field-input text-area markdown-textarea"
        rows={rows}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}
