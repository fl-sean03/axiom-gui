import { useEffect } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { openStructure, exportStructure, exportMeasurements } from '../utils/tauri'

/**
 * Global keyboard shortcuts hook
 * Implements WCAG 2.1 AA keyboard accessibility
 */
export function useKeyboardShortcuts() {
  const atoms = useAxiomStore(state => state.atoms)
  const selectedAtoms = useAxiomStore(state => state.selectedAtoms)
  const clearSelectedAtoms = useAxiomStore(state => state.clearSelectedAtoms)
  const renderSettings = useAxiomStore(state => state.renderSettings)
  const setRenderSettings = useAxiomStore(state => state.setRenderSettings)
  const setCamera = useAxiomStore(state => state.setCamera)
  const camera = useAxiomStore(state => state.camera)
  const distanceMeasurements = useAxiomStore(state => state.distanceMeasurements)
  const angleMeasurements = useAxiomStore(state => state.angleMeasurements)

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if typing in an input field
      if (e.target instanceof HTMLInputElement ||
          e.target instanceof HTMLTextAreaElement) {
        return
      }

      // Ctrl/Cmd shortcuts
      if (e.ctrlKey || e.metaKey) {
        switch (e.key.toLowerCase()) {
          case 'o':
            e.preventDefault()
            openStructure()
            break

          case 's':
            e.preventDefault()
            if (atoms) {
              exportStructure('pdb')
            }
            break

          case 'e':
            e.preventDefault()
            if (atoms) {
              // Export screenshot - trigger render at high resolution
              // This will be handled by the export panel
              console.log('Screenshot export triggered via keyboard shortcut')
            }
            break

          case 'm':
            e.preventDefault()
            if (distanceMeasurements.length > 0 || angleMeasurements.length > 0) {
              exportMeasurements(distanceMeasurements, angleMeasurements)
            }
            break

          case 'a':
            e.preventDefault()
            // Select all atoms (limit to first 1000 for performance)
            if (atoms && atoms.count > 0) {
              const maxSelect = Math.min(atoms.count, 1000)
              const selectedAtoms = Array.from({ length: maxSelect }, (_, i) => ({
                index: i,
                element: atoms.elements[i],
                position: [
                  atoms.positions[i * 3],
                  atoms.positions[i * 3 + 1],
                  atoms.positions[i * 3 + 2]
                ] as [number, number, number]
              }))
              useAxiomStore.setState({ selectedAtoms })
            }
            break

          case 'z':
            e.preventDefault()
            // Reset camera
            useAxiomStore.getState().setCameraPreset('default')
            break
        }
      }

      // Non-modifier shortcuts
      switch (e.key) {
        case 'Escape':
          e.preventDefault()
          clearSelectedAtoms()
          break

        case '1':
        case '2':
        case '3':
        case '4':
          if (!e.ctrlKey && !e.metaKey) {
            e.preventDefault()
            const renderStyles = ['ball-and-stick', 'spacefill', 'stick', 'wireframe'] as const
            const style = renderStyles[parseInt(e.key) - 1]
            if (style) {
              setRenderSettings({ renderStyle: style })
            }
          }
          break

        case 'f':
          e.preventDefault()
          // Fit to view - reset camera to default
          useAxiomStore.getState().setCameraPreset('default')
          break

        case 'r':
          e.preventDefault()
          // Re-render - trigger render by updating render state
          useAxiomStore.setState({ renderState: 'idle' })
          break

        case '?':
          e.preventDefault()
          // Show keyboard shortcuts help
          useAxiomStore.setState({ showKeyboardHelp: true })
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [atoms, selectedAtoms, clearSelectedAtoms, renderSettings, setRenderSettings, setCamera, camera, distanceMeasurements, angleMeasurements])
}
