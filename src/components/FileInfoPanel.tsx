import { useState } from 'react'
import { FileIcon, ChevronDownIcon, ChevronUpIcon, BoxIcon, LinkIcon } from 'lucide-react'
import { useAxiomStore } from '../store/axiomStore'
import { ELEMENT_SYMBOLS, ELEMENT_COLORS } from '../types/axiom'

/**
 * Enhanced File Information panel with detailed statistics
 * Shows atom counts, element breakdown, bond count, structure dimensions, and file path
 */
export function FileInfoPanel() {
  const { fileInfo, atoms, stats } = useAxiomStore()
  const [expanded, setExpanded] = useState(true)

  // Don't render if no file loaded
  if (!fileInfo || !atoms) {
    return null
  }

  // Calculate structure dimensions from bounding box
  const dimensions = atoms.bounds
    ? {
        x: atoms.bounds.max[0] - atoms.bounds.min[0],
        y: atoms.bounds.max[1] - atoms.bounds.min[1],
        z: atoms.bounds.max[2] - atoms.bounds.min[2],
      }
    : null

  // Sort elements by count (most abundant first)
  const elementBreakdown = stats?.element_counts
    ? Object.entries(stats.element_counts)
        .map(([atomicNum, count]) => ({
          atomicNum: parseInt(atomicNum),
          symbol: ELEMENT_SYMBOLS[parseInt(atomicNum)] || `E${atomicNum}`,
          color: ELEMENT_COLORS[parseInt(atomicNum)] || '#808080',
          count,
        }))
        .sort((a, b) => b.count - a.count)
    : []

  // Truncate path for display
  const truncatePath = (path: string, maxLength: number = 40) => {
    if (path.length <= maxLength) return path
    const fileName = path.split('/').pop() || ''
    const dirPath = path.substring(0, path.length - fileName.length)
    const availableLength = maxLength - fileName.length - 3 // 3 for "..."
    if (availableLength < 10) {
      return `...${fileName}`
    }
    return `${dirPath.substring(0, availableLength)}...${fileName}`
  }

  return (
    <div className="space-y-2">
      {/* Header with expand/collapse */}
      <div
        className="flex items-center justify-between cursor-pointer hover:opacity-80 transition-opacity"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center gap-2">
          <FileIcon size={16} className="text-primary" />
          <h3 className="text-sm font-semibold">File Information</h3>
        </div>
        {expanded ? <ChevronUpIcon size={16} /> : <ChevronDownIcon size={16} />}
      </div>

      {expanded && (
        <div className="text-xs space-y-3">
          {/* Basic file info */}
          <div className="space-y-1.5">
            <div className="flex justify-between items-center">
              <span className="text-muted-foreground">Name:</span>
              <span className="font-medium truncate ml-2" title={fileInfo.name}>
                {fileInfo.name}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-muted-foreground">Format:</span>
              <span className="font-medium uppercase">
                {fileInfo.format}
              </span>
            </div>
            <div
              className="flex justify-between items-center group cursor-help"
              title={fileInfo.path}
            >
              <span className="text-muted-foreground">Path:</span>
              <span className="font-mono text-[10px] truncate ml-2 group-hover:text-primary transition-colors">
                {truncatePath(fileInfo.path)}
              </span>
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-border/50" />

          {/* Atom statistics */}
          <div className="space-y-1.5">
            <div className="flex justify-between items-center">
              <span className="text-muted-foreground">Total Atoms:</span>
              <span className="font-semibold text-primary">{atoms.count.toLocaleString()}</span>
            </div>

            {/* Element breakdown */}
            {elementBreakdown.length > 0 && (
              <div className="space-y-1 mt-2">
                <span className="text-muted-foreground block mb-1">Elements:</span>
                <div className="space-y-0.5 pl-2">
                  {elementBreakdown.map((element) => (
                    <div
                      key={element.atomicNum}
                      className="flex justify-between items-center group"
                    >
                      <div className="flex items-center gap-1.5">
                        {/* Color indicator */}
                        <div
                          className="w-2.5 h-2.5 rounded-full ring-1 ring-black/10"
                          style={{ backgroundColor: element.color }}
                          title={`${element.symbol} (Atomic #${element.atomicNum})`}
                        />
                        <span className="font-medium">{element.symbol}</span>
                      </div>
                      <span className="text-muted-foreground">
                        {element.count.toLocaleString()}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Bond information */}
          {stats && (
            <>
              <div className="border-t border-border/50" />
              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-1.5">
                    <LinkIcon size={12} className="text-muted-foreground" />
                    <span className="text-muted-foreground">Bonds:</span>
                  </div>
                  <span className="font-medium">
                    {stats.has_bonds
                      ? stats.bond_count.toLocaleString()
                      : 'Not computed'}
                  </span>
                </div>
              </div>
            </>
          )}

          {/* Structure dimensions */}
          {dimensions && (
            <>
              <div className="border-t border-border/50" />
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 mb-1">
                  <BoxIcon size={12} className="text-muted-foreground" />
                  <span className="text-muted-foreground">Dimensions (Å):</span>
                </div>
                <div className="pl-2 space-y-0.5 font-mono text-[10px]">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">X:</span>
                    <span>{dimensions.x.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Y:</span>
                    <span>{dimensions.y.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Z:</span>
                    <span>{dimensions.z.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between text-primary font-semibold pt-0.5 border-t border-border/30">
                    <span>Volume:</span>
                    <span>
                      {(dimensions.x * dimensions.y * dimensions.z).toFixed(2)} ų
                    </span>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  )
}
