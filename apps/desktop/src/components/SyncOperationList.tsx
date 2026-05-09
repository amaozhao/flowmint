import type { SyncOperation } from "../api/sync";

type SyncOperationListProps = {
  operations: SyncOperation[];
  selectedTarget: string | null;
  onSelect: (operation: SyncOperation) => void;
};

export function SyncOperationList({ operations, selectedTarget, onSelect }: SyncOperationListProps) {
  return (
    <div className="sync-operation-list">
      {operations.map((operation) => (
        <button
          className={
            operation.targetPath === selectedTarget ? "sync-operation active" : "sync-operation"
          }
          key={`${operation.operationType}:${operation.targetPath}`}
          type="button"
          onClick={() => onSelect(operation)}
        >
          <span className={`operation-pill ${operation.operationType}`}>{operation.operationType}</span>
          <span>{operation.targetPath}</span>
        </button>
      ))}
    </div>
  );
}
