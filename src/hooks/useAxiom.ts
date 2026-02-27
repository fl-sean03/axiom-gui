import { useCallback } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import * as tauri from '../utils/tauri'
import { fitCameraToBox } from '../utils/camera'
import { useRecentFilesContext } from '../contexts/RecentFilesContext'

/**
 * Hook for Axiom operations (load, render, select, save)
 */
export function useAxiom() {
  const {
    atoms,
    renderConfig,
    camera,
    setAtoms,
    setStats,
    setRenderImage,
    setRenderState,
    setRenderProgress,
    setCamera,
    setSelection,
  } = useAxiomStore()
  const { addRecent } = useRecentFilesContext()

  /**
   * Render current structure with current config
   * Uses async rendering with progress updates to keep UI responsive
   */
  const renderStructure = useCallback(async () => {
    if (!atoms) return

    try {
      setRenderState('rendering')
      setRenderProgress(0)

      // Yield to UI thread before starting heavy computation
      await new Promise((resolve) => requestAnimationFrame(resolve))

      // Simulate progress updates (since backend doesn't report progress yet)
      // In future, backend could send progress events via channels
      let currentProgress = 0
      const progressInterval = setInterval(() => {
        currentProgress = Math.min(currentProgress + 10, 90)
        setRenderProgress(currentProgress)
      }, 50)

      // Perform actual rendering (already async in Tauri)
      const imageData = await tauri.renderStructure(renderConfig, camera)

      clearInterval(progressInterval)
      setRenderProgress(100)

      // Yield to UI thread before updating DOM with large image
      await new Promise((resolve) => requestAnimationFrame(resolve))

      const dataURL = tauri.arrayBufferToDataURL(new Uint8Array(imageData))
      setRenderImage(dataURL)
    } catch (error) {
      console.error('Failed to render structure:', error)
      setRenderState('error', String(error))
      setRenderProgress(null)
    }
  }, [
    atoms,
    renderConfig,
    camera,
    setRenderImage,
    setRenderState,
    setRenderProgress,
  ])

  /**
   * Open and load a molecular structure file
   */
  const loadStructure = useCallback(async () => {
    try {
      setRenderState('loading')
      setRenderProgress(0)

      const result = await tauri.openStructure()
      if (!result) {
        setRenderState('idle')
        setRenderProgress(null)
        return
      }

      setRenderProgress(30)
      const { atoms, path, name, format } = result

      // Yield to UI thread
      await new Promise((resolve) => requestAnimationFrame(resolve))

      // Set atoms and file info
      setAtoms(atoms, { path, name, format })

      // Add to recent files
      addRecent({ path, name, format })
      setRenderProgress(50)

      // Fit camera to structure
      const newCamera = fitCameraToBox(atoms.bounds, 45)
      setCamera(newCamera)
      setRenderProgress(70)

      // Get statistics
      const stats = await tauri.getStatistics()
      setStats(stats)
      setRenderProgress(80)

      // Auto-render - call renderStructure directly without waiting for useCallback update
      // This works because renderStructure is defined above
      await renderStructure()
    } catch (error) {
      console.error('Failed to load structure:', error)
      setRenderState('error', String(error))
      setRenderProgress(null)
    }
  }, [
    setAtoms,
    setCamera,
    setStats,
    setRenderState,
    setRenderProgress,
    renderStructure,
  ])

  /**
   * Apply semantic selection query
   */
  const applySelection = useCallback(
    async (query: string) => {
      if (!atoms) return

      try {
        const selection = await tauri.selectAtoms(query)
        setSelection(selection)

        // Re-render with selection highlighted
        await renderStructure()
      } catch (error) {
        console.error('Selection failed:', error)
        setSelection(null)
        throw error
      }
    },
    [atoms, setSelection, renderStructure],
  )

  /**
   * Clear selection
   */
  const clearSelection = useCallback(() => {
    setSelection(null)
    renderStructure()
  }, [setSelection, renderStructure])

  /**
   * Save current render to file
   */
  const saveImage = useCallback(async () => {
    const { renderImage } = useAxiomStore.getState()
    if (!renderImage) return

    try {
      // Convert data URL back to Uint8Array
      const base64 = renderImage.split(',')[1]
      const binary = atob(base64)
      const array = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) {
        array[i] = binary.charCodeAt(i)
      }

      const success = await tauri.saveImage(array)
      return success
    } catch (error) {
      console.error('Failed to save image:', error)
      throw error
    }
  }, [])

  /**
   * Compute bonds for current structure
   */
  const computeBonds = useCallback(
    async (cutoff: number = 1.8) => {
      if (!atoms) return

      try {
        const bondCount = await tauri.computeBonds(cutoff)

        // Update stats
        const stats = await tauri.getStatistics()
        setStats(stats)

        return bondCount
      } catch (error) {
        console.error('Failed to compute bonds:', error)
        throw error
      }
    },
    [atoms, setStats],
  )

  return {
    loadStructure,
    renderStructure,
    applySelection,
    clearSelection,
    saveImage,
    computeBonds,
  }
}
