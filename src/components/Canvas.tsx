import { useEffect, useRef } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { useMouseControls } from '../hooks/useMouseControls'
import { useAxiom } from '../hooks/useAxiom'
import { useDragAndDrop } from '../hooks/useDragAndDrop'
import { LoadingSpinner } from './LoadingSpinner'
import { MeasurementOverlay } from './MeasurementOverlay'
import { UploadIcon } from 'lucide-react'

/**
 * Main 3D viewer canvas - displays rendered molecular structure
 */
export function Canvas() {
  const { renderImage, renderState, renderError, renderProgress, atoms } =
    useAxiomStore()
  const { renderStructure } = useAxiom()
  const renderTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  // Mouse controls with debounced re-render
  const mouseControls = useMouseControls(() => {
    // Debounce rendering during mouse interaction
    if (renderTimeoutRef.current) clearTimeout(renderTimeoutRef.current)
    renderTimeoutRef.current = setTimeout(() => {
      renderStructure()
    }, 100)
  })

  // Drag and drop file loading
  const dragAndDrop = useDragAndDrop(() => {
    // Auto-render after file is loaded
    renderStructure()
  })

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (renderTimeoutRef.current) clearTimeout(renderTimeoutRef.current)
    }
  }, [])

  return (
    <main
      className="flex-1 flex items-center justify-center bg-black relative overflow-hidden"
      onDragEnter={dragAndDrop.handleDragEnter}
      onDragOver={dragAndDrop.handleDragOver}
      onDragLeave={dragAndDrop.handleDragLeave}
      onDrop={dragAndDrop.handleDrop}
      role="main"
      aria-label="Molecular structure viewer"
    >
      {/* Drag and drop overlay */}
      {dragAndDrop.isDragging && (
        <div className="absolute inset-0 bg-blue-500/20 backdrop-blur-sm flex items-center justify-center z-50 border-4 border-blue-500 border-dashed animate-fade-in" role="alert" aria-live="polite">
          <div className="text-white text-center animate-pulse-soft">
            <UploadIcon size={64} className="mx-auto mb-4" />
            <div className="text-2xl font-bold">Drop file to load</div>
            <div className="text-sm mt-2 opacity-80">
              Supports PDB, XYZ, GRO, LAMMPS, CIF
            </div>
          </div>
        </div>
      )}

      {/* Loading state */}
      {renderState === 'loading' && (
        <LoadingSpinner message="Loading structure..." progress={renderProgress} />
      )}

      {/* Rendering state */}
      {renderState === 'rendering' && (
        <LoadingSpinner message="Rendering..." progress={renderProgress} />
      )}

      {/* Error state */}
      {renderState === 'error' && (
        <div className="text-red-500 text-center p-8 max-w-md animate-fade-in" role="alert" aria-live="assertive">
          <div className="text-xl font-bold mb-4">Unable to Render Structure</div>
          <div className="text-sm bg-red-500/10 p-4 rounded-lg border border-red-500/30">
            {renderError || 'An unexpected error occurred while rendering the molecular structure.'}
          </div>
          <div className="text-xs mt-4 text-gray-400">
            Try reloading the file or adjusting rendering settings.
          </div>
        </div>
      )}

      {/* Rendered image */}
      {renderImage && renderState === 'idle' && (
        <img
          src={renderImage}
          alt="Rendered 3D molecular structure visualization"
          className="w-full h-full object-cover cursor-move no-select transition-fast animate-fade-in"
          onMouseDown={mouseControls.handleMouseDown}
          onMouseMove={mouseControls.handleMouseMove}
          onMouseUp={mouseControls.handleMouseUp}
          onWheel={mouseControls.handleWheel}
          onContextMenu={mouseControls.handleContextMenu}
          draggable={false}
          role="img"
          aria-label={atoms ? `Molecular structure with ${atoms.count} atoms` : 'Molecular structure'}
        />
      )}

      {/* Empty state */}
      {!atoms && renderState === 'idle' && (
        <div className="text-gray-400 text-center p-8 animate-fade-in" role="status" aria-label="No structure loaded">
          <UploadIcon size={48} className="mx-auto mb-4 opacity-50 animate-pulse-soft" />
          <div className="text-xl font-semibold mb-2">No Structure Loaded</div>
          <div className="text-sm mb-4">
            Drag and drop a file here, or press <kbd className="px-2 py-1 bg-gray-800 rounded text-xs font-mono">Ctrl+O</kbd> to open
          </div>
          <div className="text-xs opacity-70 space-y-1">
            <div>Supported formats:</div>
            <div>PDB, XYZ, GRO, LAMMPS, CIF</div>
          </div>
        </div>
      )}

      {/* Controls hint overlay */}
      {atoms && (
        <div className="absolute bottom-4 right-4 bg-black/80 backdrop-blur-sm text-white text-xs p-3 rounded-lg shadow-lg animate-slide-in" role="note" aria-label="Mouse controls">
          <div className="font-semibold mb-1">Mouse Controls</div>
          <div className="space-y-1 text-gray-300">
            <div><strong>Left drag:</strong> Orbit</div>
            <div><strong>Right drag:</strong> Pan</div>
            <div><strong>Scroll:</strong> Zoom</div>
          </div>
        </div>
      )}

      {/* Measurement overlays */}
      {atoms && renderImage && renderState === 'idle' && <MeasurementOverlay />}
    </main>
  )
}
