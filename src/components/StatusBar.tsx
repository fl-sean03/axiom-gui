import { useAxiomStore } from '../store/axiomStore'
import { ELEMENT_SYMBOLS } from '../types/axiom'

/**
 * Bottom status bar - shows file info, atom count, status
 */
export function StatusBar() {
  const { fileInfo, atoms, stats, renderState, selection } = useAxiomStore()

  const getStatusColor = () => {
    switch (renderState) {
      case 'error': return 'text-red-600 dark:text-red-400'
      case 'loading':
      case 'rendering': return 'text-blue-600 dark:text-blue-400'
      case 'idle': return atoms ? 'text-green-600 dark:text-green-400' : 'text-gray-500'
      default: return 'text-gray-500'
    }
  }

  return (
    <footer className="h-8 bg-secondary border-t border-border px-4 flex items-center justify-between text-xs" role="contentinfo" aria-label="Status bar">
      {/* Left: File info */}
      <div className="flex items-center gap-4" role="status" aria-live="polite">
        {fileInfo && atoms && (
          <>
            <span className="font-medium" title={fileInfo.path}>{fileInfo.name}</span>
            <span className="text-muted-foreground px-2 py-0.5 bg-muted rounded">
              {fileInfo.format.toUpperCase()}
            </span>
            <span className="text-muted-foreground">
              {atoms.count.toLocaleString()} atoms
            </span>
            {stats && stats.bond_count > 0 && (
              <span className="text-muted-foreground">
                {stats.bond_count.toLocaleString()} bonds
              </span>
            )}
          </>
        )}

        {/* Selection info */}
        {selection && selection.count > 0 && (
          <span className="text-primary font-medium px-2 py-0.5 bg-primary/10 rounded">
            {selection.count.toLocaleString()} selected
          </span>
        )}
      </div>

      {/* Right: Status */}
      <div className="flex items-center gap-2" role="status" aria-live="polite" aria-atomic="true">
        <div className={`flex items-center gap-2 ${getStatusColor()} transition-colors`}>
          {/* Status indicator dot */}
          <div className={`w-2 h-2 rounded-full ${
            renderState === 'error' ? 'bg-red-600' :
            renderState === 'loading' || renderState === 'rendering' ? 'bg-blue-600 animate-pulse' :
            renderState === 'idle' && atoms ? 'bg-green-600' :
            'bg-gray-400'
          }`} aria-hidden="true" />

          {/* Status text */}
          {renderState === 'loading' && <span>Loading...</span>}
          {renderState === 'rendering' && <span>Rendering...</span>}
          {renderState === 'error' && <span className="font-medium">Error</span>}
          {renderState === 'idle' && atoms && <span className="font-medium">Ready</span>}
          {renderState === 'idle' && !atoms && <span>No file loaded</span>}
        </div>
      </div>
    </footer>
  )
}
