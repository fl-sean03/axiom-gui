import { useAxiomStore } from '../store/axiomStore'
import { ELEMENT_SYMBOLS, ELEMENT_COLORS } from '../types/axiom'
import { formatDistance, formatAngle } from '../utils/measurements'

/**
 * MeasurementOverlay - Renders measurement labels on top of the canvas
 * Shows distance and angle measurements with connecting lines
 */
export function MeasurementOverlay() {
  const { selectedAtoms, distanceMeasurements, angleMeasurements } =
    useAxiomStore()

  // For Phase 4, we'll show a simplified overlay
  // Full implementation would require projecting 3D positions to 2D screen coordinates
  // For now, we show labels in a fixed overlay position

  const hasContent =
    selectedAtoms.length > 0 ||
    distanceMeasurements.length > 0 ||
    angleMeasurements.length > 0

  if (!hasContent) return null

  return (
    <div className="absolute inset-0 pointer-events-none">
      {/* Top-left corner: Selected atoms indicator */}
      {selectedAtoms.length > 0 && (
        <div className="absolute top-4 left-4 bg-black/80 text-white text-xs px-3 py-2 rounded-lg shadow-lg">
          <div className="font-semibold mb-1">
            {selectedAtoms.length} atom{selectedAtoms.length > 1 ? 's' : ''} selected
          </div>
          <div className="space-y-0.5">
            {selectedAtoms.map((atom) => {
              const symbol =
                ELEMENT_SYMBOLS[atom.element] || `#${atom.element}`
              const color = ELEMENT_COLORS[atom.element] || '#808080'
              return (
                <div
                  key={atom.index}
                  className="flex items-center gap-2 font-mono"
                >
                  <div
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: color }}
                  />
                  <span>
                    {symbol} #{atom.index}
                  </span>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Bottom-left corner: Measurements */}
      {(distanceMeasurements.length > 0 || angleMeasurements.length > 0) && (
        <div className="absolute bottom-20 left-4 bg-black/80 text-white text-xs px-3 py-2 rounded-lg shadow-lg max-w-xs">
          <div className="font-semibold mb-2">Measurements</div>

          {/* Distance measurements */}
          {distanceMeasurements.map((measurement) => {
            const sym1 =
              ELEMENT_SYMBOLS[measurement.atom1.element] ||
              `#${measurement.atom1.element}`
            const sym2 =
              ELEMENT_SYMBOLS[measurement.atom2.element] ||
              `#${measurement.atom2.element}`
            return (
              <div
                key={measurement.id}
                className="mb-1 pb-1 border-b border-white/20 last:border-0"
              >
                <div className="flex items-center justify-between">
                  <span className="font-mono text-[11px]">
                    {sym1}#{measurement.atom1.index} ↔ {sym2}#
                    {measurement.atom2.index}
                  </span>
                  <span className="font-bold ml-2 text-yellow-400">
                    {formatDistance(measurement.distance)}
                  </span>
                </div>
              </div>
            )
          })}

          {/* Angle measurements */}
          {angleMeasurements.map((measurement) => {
            const sym1 =
              ELEMENT_SYMBOLS[measurement.atom1.element] ||
              `#${measurement.atom1.element}`
            const sym2 =
              ELEMENT_SYMBOLS[measurement.atom2.element] ||
              `#${measurement.atom2.element}`
            const sym3 =
              ELEMENT_SYMBOLS[measurement.atom3.element] ||
              `#${measurement.atom3.element}`
            return (
              <div
                key={measurement.id}
                className="mb-1 pb-1 border-b border-white/20 last:border-0"
              >
                <div className="flex items-center justify-between mb-0.5">
                  <span className="font-mono text-[11px]">
                    {sym1}#{measurement.atom1.index} - {sym2}#
                    {measurement.atom2.index} - {sym3}#{measurement.atom3.index}
                  </span>
                  <span className="font-bold ml-2 text-green-400">
                    {formatAngle(measurement.angle)}
                  </span>
                </div>
                <div className="text-[10px] text-white/60">
                  Vertex: {sym2}#{measurement.atom2.index}
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
