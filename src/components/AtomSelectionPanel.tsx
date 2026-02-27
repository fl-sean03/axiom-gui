import { useAxiomStore } from '../store/axiomStore'
import { useClickSelection } from '../hooks/useClickSelection'
import { Button } from './ui/Button'
import {
  AtomIcon,
  XIcon,
  RulerIcon,
  MoveIcon,
  InfoIcon,
} from 'lucide-react'
import { ELEMENT_SYMBOLS, ELEMENT_COLORS } from '../types/axiom'
import {
  formatDistance,
  formatAngle,
} from '../utils/measurements'

/**
 * AtomSelectionPanel - Shows selected atoms and measurements
 * Displays atom details, distance/angle measurements, and selection controls
 */
export function AtomSelectionPanel() {
  const {
    selectedAtoms,
    distanceMeasurements,
    angleMeasurements,
    atoms,
  } = useAxiomStore()
  const { clearSelection } = useClickSelection()

  if (!atoms) return null

  const hasSelection = selectedAtoms.length > 0
  const hasMeasurements =
    distanceMeasurements.length > 0 || angleMeasurements.length > 0

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold flex items-center gap-2">
          <AtomIcon size={14} />
          Selection & Measurements
        </h3>
        {hasSelection && (
          <Button
            variant="outline"
            size="sm"
            onClick={clearSelection}
            className="gap-1"
          >
            <XIcon size={12} />
            Clear
          </Button>
        )}
      </div>

      {/* Instructions */}
      {!hasSelection && (
        <div className="text-xs text-muted-foreground bg-muted/30 p-3 rounded-md space-y-1">
          <div className="flex items-start gap-2">
            <InfoIcon size={12} className="mt-0.5 shrink-0" />
            <div>
              <div className="font-medium mb-1">How to select atoms:</div>
              <div className="space-y-0.5">
                <div>• Click atom number below to select</div>
                <div>• <kbd className="px-1 py-0.5 bg-background rounded text-[10px]">Ctrl</kbd> + click to toggle</div>
                <div>• <kbd className="px-1 py-0.5 bg-background rounded text-[10px]">Shift</kbd> + click to add</div>
                <div>• <kbd className="px-1 py-0.5 bg-background rounded text-[10px]">Esc</kbd> to clear selection</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Selected Atoms */}
      {hasSelection && (
        <div className="space-y-2">
          <label className="text-xs font-medium text-muted-foreground">
            Selected Atoms ({selectedAtoms.length})
          </label>
          <div className="space-y-1">
            {selectedAtoms.map((atom, idx) => {
              const symbol = ELEMENT_SYMBOLS[atom.element] || `#${atom.element}`
              const color = ELEMENT_COLORS[atom.element] || '#808080'
              return (
                <div
                  key={atom.index}
                  className="flex items-center gap-2 p-2 bg-accent/50 rounded-md text-xs"
                >
                  <div className="flex items-center gap-2 flex-1">
                    <div
                      className="w-3 h-3 rounded-full shrink-0"
                      style={{ backgroundColor: color }}
                    />
                    <div className="font-mono font-medium">{symbol}</div>
                    <div className="text-muted-foreground">#{atom.index}</div>
                  </div>
                  <div className="flex items-center gap-1 text-muted-foreground">
                    <MoveIcon size={10} />
                    <span className="font-mono text-[10px]">
                      ({atom.position[0].toFixed(1)}, {atom.position[1].toFixed(1)}, {atom.position[2].toFixed(1)})
                    </span>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Distance Measurements */}
      {distanceMeasurements.length > 0 && (
        <div className="space-y-2">
          <label className="text-xs font-medium text-muted-foreground flex items-center gap-1">
            <RulerIcon size={12} />
            Distances
          </label>
          <div className="space-y-1">
            {distanceMeasurements.map((measurement) => {
              const sym1 = ELEMENT_SYMBOLS[measurement.atom1.element] || `#${measurement.atom1.element}`
              const sym2 = ELEMENT_SYMBOLS[measurement.atom2.element] || `#${measurement.atom2.element}`
              return (
                <div
                  key={measurement.id}
                  className="p-2 bg-primary/10 border border-primary/20 rounded-md text-xs"
                >
                  <div className="flex items-center justify-between">
                    <div className="font-mono">
                      {sym1}#{measurement.atom1.index} ↔ {sym2}#{measurement.atom2.index}
                    </div>
                    <div className="font-bold text-primary">
                      {formatDistance(measurement.distance)}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Angle Measurements */}
      {angleMeasurements.length > 0 && (
        <div className="space-y-2">
          <label className="text-xs font-medium text-muted-foreground flex items-center gap-1">
            <RulerIcon size={12} />
            Angles
          </label>
          <div className="space-y-1">
            {angleMeasurements.map((measurement) => {
              const sym1 = ELEMENT_SYMBOLS[measurement.atom1.element] || `#${measurement.atom1.element}`
              const sym2 = ELEMENT_SYMBOLS[measurement.atom2.element] || `#${measurement.atom2.element}`
              const sym3 = ELEMENT_SYMBOLS[measurement.atom3.element] || `#${measurement.atom3.element}`
              return (
                <div
                  key={measurement.id}
                  className="p-2 bg-primary/10 border border-primary/20 rounded-md text-xs"
                >
                  <div className="flex items-center justify-between mb-1">
                    <div className="font-mono text-[10px]">
                      {sym1}#{measurement.atom1.index} - {sym2}#{measurement.atom2.index} - {sym3}#{measurement.atom3.index}
                    </div>
                    <div className="font-bold text-primary">
                      {formatAngle(measurement.angle)}
                    </div>
                  </div>
                  <div className="text-[10px] text-muted-foreground">
                    Vertex: {sym2}#{measurement.atom2.index}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Auto-measurement hint */}
      {hasSelection && selectedAtoms.length === 1 && (
        <div className="text-xs text-muted-foreground bg-muted/30 p-2 rounded-md">
          Select 1 more atom to measure distance
        </div>
      )}
      {hasSelection && selectedAtoms.length === 2 && (
        <div className="text-xs text-muted-foreground bg-muted/30 p-2 rounded-md">
          Select 1 more atom to measure angle
        </div>
      )}
    </div>
  )
}
