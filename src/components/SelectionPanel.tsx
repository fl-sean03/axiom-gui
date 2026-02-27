import { useState } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { useAxiom } from '../hooks/useAxiom'
import { Button } from './ui/Button'
import { SearchIcon, XIcon } from 'lucide-react'

/**
 * Selection panel (semantic query input)
 */
export function SelectionPanel() {
  const { selection } = useAxiomStore()
  const { applySelection, clearSelection } = useAxiom()
  const [query, setQuery] = useState('')
  const [error, setError] = useState<string | null>(null)

  const handleApply = async () => {
    if (!query.trim()) return

    try {
      setError(null)
      await applySelection(query)
    } catch (err) {
      setError(String(err))
    }
  }

  const handleClear = () => {
    setQuery('')
    setError(null)
    clearSelection()
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleApply()
    }
  }

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold">Selection</h3>

      {/* Query input */}
      <div className="space-y-2">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder='e.g., "element O"'
          className="w-full px-3 py-2 text-sm border border-input rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-ring"
        />

        {/* Action buttons */}
        <div className="flex gap-2">
          <Button
            variant="default"
            size="sm"
            onClick={handleApply}
            disabled={!query.trim()}
            className="flex-1 gap-2"
          >
            <SearchIcon size={14} />
            Apply
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleClear}
            disabled={!selection && !query}
            className="gap-2"
          >
            <XIcon size={14} />
            Clear
          </Button>
        </div>
      </div>

      {/* Error message */}
      {error && (
        <div className="text-xs text-destructive bg-destructive/10 p-2 rounded-md">
          {error}
        </div>
      )}

      {/* Selection info */}
      {selection && selection.count > 0 && (
        <div className="text-xs text-primary bg-primary/10 p-2 rounded-md">
          <strong>{selection.count}</strong> atoms selected
        </div>
      )}

      {/* Example queries */}
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">Examples:</label>
        <div className="text-xs space-y-1">
          {[
            'element O',
            'element C and within 5 of element N',
            'within 10 of resname LIG',
            '(element O or element N) and not resname WAT',
          ].map((example, i) => (
            <button
              key={i}
              onClick={() => setQuery(example)}
              className="block w-full text-left px-2 py-1 hover:bg-accent rounded text-muted-foreground hover:text-foreground"
            >
              {example}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
