import { DatabaseConnection } from "./envParser";
import SqlExecutionInspector, { ExecutionResult } from "./SqlExecutionInspector";

interface SqlExecutionResultsProps {
  results: ExecutionResult[] | null;
  connections?: DatabaseConnection[];
  isExecuting?: boolean;
  onRetryConnection?: (connection: DatabaseConnection) => void;
  onClear?: () => void;
}

export default function SqlExecutionResults({
  results,
  connections = [],
  isExecuting = false,
  onRetryConnection,
  onClear,
}: SqlExecutionResultsProps) {
  return (
    <SqlExecutionInspector
      results={results}
      connections={connections}
      isExecuting={isExecuting}
      onRetryConnection={onRetryConnection}
      onClear={onClear}
    />
  );
}
