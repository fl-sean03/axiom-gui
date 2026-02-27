import { useState, useCallback, DragEvent } from 'react'
import { loadStructureFromPath } from '../utils/tauri'
import { useAxiomStore } from '../store/axiomStore'
import { fitCameraToBox } from '../utils/camera'
import { getStatistics } from '../utils/tauri'
import { useRecentFilesContext } from '../contexts/RecentFilesContext'

/**
 * Hook for handling drag-and-drop file loading
 */
export function useDragAndDrop(onLoad?: () => void) {
  const [isDragging, setIsDragging] = useState(false)
  const {
    setAtoms,
    setCamera,
    setStats,
    setRenderState,
    setRenderProgress,
  } = useAxiomStore()
  const { addRecent } = useRecentFilesContext()

  /**
   * Handle drag enter event
   */
  const handleDragEnter = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])

  /**
   * Handle drag over event
   */
  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  /**
   * Handle drag leave event
   */
  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()

    // Only hide drop zone if leaving the main container
    if (e.currentTarget === e.target) {
      setIsDragging(false)
    }
  }, [])

  /**
   * Handle file drop event
   */
  const handleDrop = useCallback(
    async (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault()
      e.stopPropagation()
      setIsDragging(false)

      try {
        setRenderState('loading')
        setRenderProgress(0)

        // Get dropped file path from Tauri event
        // In Tauri, file paths are available in dataTransfer
        const files = Array.from(e.dataTransfer.files)
        if (files.length === 0) return

        const file = files[0]
        // @ts-ignore - Tauri adds path property to File objects
        const path = file.path

        if (!path) {
          throw new Error('Could not get file path from dropped file')
        }

        setRenderProgress(30)

        // Load structure using the new path-based function
        const { atoms, path: filePath, name, format } = await loadStructureFromPath(path)

        // Yield to UI thread
        await new Promise((resolve) => requestAnimationFrame(resolve))

        // Set atoms and file info
        setAtoms(atoms, { path: filePath, name, format })

        // Add to recent files
        addRecent({ path: filePath, name, format })
        setRenderProgress(50)

        // Fit camera to structure
        const newCamera = fitCameraToBox(atoms.bounds, 45)
        setCamera(newCamera)
        setRenderProgress(70)

        // Get statistics
        const stats = await getStatistics()
        setStats(stats)
        setRenderProgress(80)

        // Call onLoad callback to trigger rendering
        if (onLoad) {
          onLoad()
        }

        setRenderProgress(100)
        setRenderState('idle')
        setRenderProgress(null)
      } catch (error) {
        console.error('Failed to load dropped file:', error)
        setRenderState('error', String(error))
        setRenderProgress(null)
      }
    },
    [setAtoms, setCamera, setStats, setRenderState, setRenderProgress, onLoad],
  )

  return {
    isDragging,
    handleDragEnter,
    handleDragOver,
    handleDragLeave,
    handleDrop,
  }
}
