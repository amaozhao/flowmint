import { useEffect, useState } from "react";
import { parseTags } from "../pages/assetEditorModel";

type TagInputProps = {
  id: string;
  label: string;
  value: string[];
  onChange: (value: string[]) => void;
};

export function TagInput({ id, label, value, onChange }: TagInputProps) {
  const [rawValue, setRawValue] = useState(value.join(", "));

  useEffect(() => {
    setRawValue(value.join(", "));
  }, [value]);

  function handleChange(nextValue: string) {
    setRawValue(nextValue);
    onChange(parseTags(nextValue));
  }

  return (
    <label className="field" htmlFor={id}>
      <span>{label}</span>
      <input
        id={id}
        className="field-input"
        type="text"
        value={rawValue}
        onChange={(event) => handleChange(event.target.value)}
      />
    </label>
  );
}
