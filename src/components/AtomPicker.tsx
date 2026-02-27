import { useState } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { useClickSelection } from '../hooks/useClickSelection'
import { Button } from './ui/Button'
import { ELEMENT_SYMBOLS, ELEMENT_COLORS } from '../types/axiom'
import { SearchIcon, XIcon, AtomIcon } from 'lucide-react'

/**
 * AtomPicker - Modal for selecting atoms by index/number
 * Allows clicking on atom list to select atoms for measurements
 */
export function AtomPicker() {
  const { atoms, stats } = useAxiomStore()
  const { selectAtom } = useClickSelection()
  const [searchQuery, setSearchQuery] = useState('')
  const [showPicker, setShowPicker] = useState(false)

  if (!atoms || !stats) return null

  // Generate atom list with element info
  const atomList = Array.from({ length: atoms.count }, (_, i) => ({
    index: i,
    element: atoms.elements[i],
    position: [
      atoms.positions[i * 3],
      atoms.positions[i * 3 + 1],
      atoms.positions[i * 3 + 2],
    ] as [number, number, number],
  }))

  // Filter atoms by search query (element symbol or index)
  const filteredAtoms = atomList.filter((atom) => {
    if (!searchQuery) return true
    const symbol = ELEMENT_SYMBOLS[atom.element] || `${atom.element}`
    const indexStr = atom.index.toString()
    const query = searchQuery.toLowerCase()
    return (
      symbol.toLowerCase().includes(query) || indexStr.includes(query)
    )
  })

  const handleAtomClick = (
    index: number,
    e: React.MouseEvent<HTMLButtonElement>,
  ) => {
    selectAtom(index, {
      ctrl: e.ctrlKey || e.metaKey,
      shift: e.shiftKey,
    })
  }

  return (
    <div className="space-y-2">
      {/* Toggle button */}
      <Button
        variant={showPicker ? 'default' : 'outline'}
        size="sm"
        onClick={() => setShowPicker(!showPicker)}
        className="w-full gap-2"
      >
        <AtomIcon size={14} />
        {showPicker ? 'Hide' : 'Show'} Atom Picker
      </Button>

      {/* Picker panel */}
      {showPicker && (
        <div className="border rounded-lg p-3 space-y-2 bg-background">
          {/* Search */}
          <div className="relative">
            <SearchIcon
              size={14}
              className="absolute left-2 top-2 text-muted-foreground"
            />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search by element or index..."
              className="w-full pl-8 pr-8 py-1.5 text-xs border rounded-md bg-background"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="absolute right-2 top-2 text-muted-foreground hover:text-foreground"
              >
                <XIcon size={12} />
              </button>
            )}
          </div>

          {/* Atom count */}
          <div className="text-xs text-muted-foreground">
            Showing {filteredAtoms.length} of {atoms.count} atoms
          </div>

          {/* Atom list */}
          <div className="max-h-64 overflow-y-auto space-y-1">
            {filteredAtoms.slice(0, 100).map((atom) => {
              const symbol =
                ELEMENT_SYMBOLS[atom.element] || `#${atom.element}`
              const color = ELEMENT_COLORS[atom.element] || '#808080'
              return (
                <button
                  key={atom.index}
                  onClick={(e) => handleAtomClick(atom.index, e)}
                  className="w-full flex items-center gap-2 p-1.5 text-xs hover:bg-accent rounded-md transition-colors text-left"
                >
                  <div
                    className="w-2.5 h-2.5 rounded-full shrink-0"
                    style={{ backgroundColor: color }}
                  />
                  <div className="font-mono font-medium w-8">{symbol}</div>
                  <div className="text-muted-foreground">#{atom.index}</div>
                  <div className="ml-auto font-mono text-[10px] text-muted-foreground">
                    ({atom.position[0].toFixed(1)}, {atom.position[1].toFixed(1)}, {atom.position[2].toFixed(1)})
                  </div>
                </button>
              )
            })}
            {filteredAtoms.length > 100 && (
              <div className="text-xs text-muted-foreground text-center p-2">
                ... {filteredAtoms.length - 100} more atoms (refine search)
              </div>
            )}
          </div>

          {/* Instructions */}
          <div className="text-[10px] text-muted-foreground border-t pt-2">
            Click atom to select • Ctrl+click to toggle • Shift+click to add
          </div>
        </div>
      )}
    </div>
  )
}
