import { useAxiomStore } from '../store/axiomStore'
import { ELEMENT_SYMBOLS } from '../types/axiom'
import { Button } from './ui/Button'

/**
 * Statistics panel (atom counts, element distribution)
 */
export function StatsPanel() {
  const { stats, showStats, toggleStats } = useAxiomStore()

  if (!stats) return null

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold">Statistics</h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={toggleStats}
          className="text-xs"
        >
          {showStats ? 'Hide' : 'Show'}
        </Button>
      </div>

      {showStats && (
        <div className="space-y-3">
          {/* Total atoms */}
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">Total Atoms:</span>
            <span className="font-medium">{stats.total_atoms}</span>
          </div>

          {/* Bonds */}
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">Bonds:</span>
            <span className="font-medium">
              {stats.has_bonds ? stats.bond_count : 'Not computed'}
            </span>
          </div>

          {/* Element distribution */}
          <div className="space-y-2">
            <label className="text-xs text-muted-foreground">
              Element Distribution:
            </label>
            <div className="space-y-1 max-h-40 overflow-y-auto">
              {Object.entries(stats.element_counts)
                .sort(([, a], [, b]) => b - a)
                .map(([atomicNum, count]) => {
                  const element = ELEMENT_SYMBOLS[Number(atomicNum)] || `#${atomicNum}`
                  const percentage = ((count / stats.total_atoms) * 100).toFixed(1)
                  return (
                    <div
                      key={atomicNum}
                      className="flex items-center justify-between text-xs"
                    >
                      <span className="font-mono">{element}</span>
                      <div className="flex items-center gap-2">
                        <div className="w-20 h-2 bg-secondary rounded-full overflow-hidden">
                          <div
                            className="h-full bg-primary"
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                        <span className="w-12 text-right text-muted-foreground">
                          {count}
                        </span>
                      </div>
                    </div>
                  )
                })}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
