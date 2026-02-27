import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useAxiomStore } from '../store/axiomStore'
import { fitCameraToBox } from '../utils/camera'
import * as tauri from '../utils/tauri'

/**
 * Auto-load test component - loads 1CRN.pdb on mount for testing
 */
export function AutoLoadTest() {
  const { setAtoms, setStats, setCamera, setRenderImage, setRenderState } = useAxiomStore()

  useEffect(() => {
    const autoLoad = async () => {
      try {
        console.log('Auto-loading 1CRN.pdb for testing...')
        setRenderState('loading')

        // Load crambin protein (327 atoms)
        const path = '/home/agent/projects/axiom/test_data/1crn.pdb'
        const atoms = await invoke<any>('load_structure', { path, format: 'pdb' })

        console.log('Loaded atoms:', atoms)

        // Set atoms and file info
        setAtoms(atoms, {
          path,
          name: '1crn.pdb',
          format: 'pdb'
        })

        // Fit camera to structure
        const newCamera = fitCameraToBox(atoms.bounds, 45)
        setCamera(newCamera)

        // Get statistics
        const stats = await tauri.getStatistics()
        setStats(stats)

        // Render
        setRenderState('rendering')
        const renderConfig = useAxiomStore.getState().renderConfig
        const camera = useAxiomStore.getState().camera

        const imageData = await tauri.renderStructure(renderConfig, camera)
        const dataURL = tauri.arrayBufferToDataURL(new Uint8Array(imageData))

        setRenderImage(dataURL)
        console.log('Auto-load complete!')
      } catch (error) {
        console.error('Auto-load failed:', error)
        setRenderState('error', String(error))
      }
    }

    // Auto-load after 1 second
    const timer = setTimeout(autoLoad, 1000)
    return () => clearTimeout(timer)
  }, []) // Empty deps - only run once

  return null // This component doesn't render anything
}
