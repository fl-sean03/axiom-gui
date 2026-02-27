import { useCallback, useRef } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { orbitCamera, panCamera, zoomCamera } from '../utils/camera'

type MouseButton = 'left' | 'right' | 'middle'

interface MouseState {
  isDragging: boolean
  button: MouseButton | null
  lastX: number
  lastY: number
}

/**
 * Hook for mouse-based camera controls (orbit, pan, zoom)
 */
export function useMouseControls(onCameraChange?: () => void) {
  const { camera, setCamera } = useAxiomStore()
  const mouseState = useRef<MouseState>({
    isDragging: false,
    button: null,
    lastX: 0,
    lastY: 0,
  })

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()

    let button: MouseButton
    if (e.button === 0) button = 'left'
    else if (e.button === 2) button = 'right'
    else if (e.button === 1) button = 'middle'
    else return

    mouseState.current = {
      isDragging: true,
      button,
      lastX: e.clientX,
      lastY: e.clientY,
    }
  }, [])

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      const state = mouseState.current
      if (!state.isDragging || !state.button) return

      const deltaX = e.clientX - state.lastX
      const deltaY = e.clientY - state.lastY

      let newCamera = camera

      if (state.button === 'left') {
        // Orbit
        const sensitivity = 0.005
        newCamera = orbitCamera(
          camera,
          deltaX * sensitivity,
          deltaY * sensitivity,
        )
      } else if (state.button === 'right' || state.button === 'middle') {
        // Pan
        const sensitivity = 0.05
        newCamera = panCamera(
          camera,
          -deltaX * sensitivity,
          deltaY * sensitivity,
        )
      }

      setCamera(newCamera)
      onCameraChange?.()

      mouseState.current.lastX = e.clientX
      mouseState.current.lastY = e.clientY
    },
    [camera, setCamera, onCameraChange],
  )

  const handleMouseUp = useCallback(() => {
    mouseState.current.isDragging = false
    mouseState.current.button = null
  }, [])

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault()

      const delta = e.deltaY * 0.1
      const newCamera = zoomCamera(camera, delta)

      setCamera(newCamera)
      onCameraChange?.()
    },
    [camera, setCamera, onCameraChange],
  )

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault() // Prevent context menu on right-click
  }, [])

  return {
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleWheel,
    handleContextMenu,
  }
}
