import { useAxiomStore } from '../store/axiomStore'
import { useAxiom } from '../hooks/useAxiom'
import { Button } from './ui/Button'
import {
  RotateCcwIcon,
  MoveVerticalIcon,
  MoveHorizontalIcon,
  BoxIcon,
  ZoomInIcon,
  ZoomOutIcon,
  MaximizeIcon,
  MoveIcon,
  RotateCwIcon,
  CircleDotIcon,
  EyeIcon,
} from 'lucide-react'
import { orbitCamera, panCamera, zoomCamera, fitCameraToBox } from '../utils/camera'
import { useState, useEffect } from 'react'

/**
 * Camera controls (reset, presets, zoom, rotation, pan)
 * Enhanced with sliders, real-time control, and visual feedback
 */
export function CameraControls() {
  const { camera, atoms, cameraPreset, setCameraPreset, setCamera } = useAxiomStore()
  const { renderStructure } = useAxiom()

  // Local state for slider values (in degrees for rotation)
  const [rotationX, setRotationX] = useState(0)
  const [rotationY, setRotationY] = useState(0)
  const [rotationZ, setRotationZ] = useState(0)
  const [zoomLevel, setZoomLevel] = useState(100)
  const [panX, setPanX] = useState(0)
  const [panY, setPanY] = useState(0)

  // Sync zoom level with camera distance
  useEffect(() => {
    const distance = Math.sqrt(
      camera.position[0] ** 2 + camera.position[1] ** 2 + camera.position[2] ** 2
    )
    setZoomLevel(Math.round((150 / distance) * 100))
  }, [camera.position])

  // Handle rotation slider changes (cumulative rotations)
  const handleRotation = (axis: 'x' | 'y' | 'z', value: number) => {
    let deltaX = 0
    let deltaY = 0

    if (axis === 'x') {
      deltaY = ((value - rotationX) * Math.PI) / 180
      setRotationX(value)
    } else if (axis === 'y') {
      deltaX = ((value - rotationY) * Math.PI) / 180
      setRotationY(value)
    } else if (axis === 'z') {
      // Z rotation would require rolling the camera (rotating the up vector)
      // For simplicity, we'll combine it with Y rotation
      deltaX = ((value - rotationZ) * Math.PI) / 180
      setRotationZ(value)
    }

    const newCamera = orbitCamera(camera, deltaX, deltaY)
    setCamera(newCamera)
    renderStructure()
  }

  // Handle zoom slider
  const handleZoomSlider = (value: number) => {
    setZoomLevel(value)
    const currentDistance = Math.sqrt(
      camera.position[0] ** 2 + camera.position[1] ** 2 + camera.position[2] ** 2
    )
    const targetDistance = 150 / (value / 100)
    const delta = targetDistance - currentDistance
    const newCamera = zoomCamera(camera, delta)
    setCamera(newCamera)
    renderStructure()
  }

  // Handle pan controls
  const handlePan = (axis: 'x' | 'y', value: number) => {
    const oldX = panX
    const oldY = panY

    if (axis === 'x') {
      setPanX(value)
      const deltaX = (value - oldX) * 2
      const newCamera = panCamera(camera, deltaX, 0)
      setCamera(newCamera)
    } else {
      setPanY(value)
      const deltaY = (value - oldY) * 2
      const newCamera = panCamera(camera, 0, deltaY)
      setCamera(newCamera)
    }
    renderStructure()
  }

  // Reset all controls to default
  const handleReset = () => {
    setRotationX(0)
    setRotationY(0)
    setRotationZ(0)
    setPanX(0)
    setPanY(0)
    setCameraPreset('default')
    renderStructure()
  }

  // Fit camera to structure bounds
  const handleFitToView = () => {
    if (!atoms) return
    const newCamera = fitCameraToBox(atoms.bounds, 45)
    setCamera(newCamera)
    setPanX(0)
    setPanY(0)
    renderStructure()
  }

  // Center on structure
  const handleCenter = () => {
    if (!atoms) return
    setCamera({ target: atoms.bounds.center })
    setPanX(0)
    setPanY(0)
    renderStructure()
  }

  const handlePreset = (preset: 'default' | 'top' | 'side' | 'front' | 'isometric') => {
    setCameraPreset(preset)
    setRotationX(0)
    setRotationY(0)
    setRotationZ(0)
    setPanX(0)
    setPanY(0)
    renderStructure()
  }

  const handleZoomButton = (factor: number) => {
    const currentDistance = Math.sqrt(
      camera.position[0] ** 2 + camera.position[1] ** 2 + camera.position[2] ** 2
    )
    const newDistance = currentDistance * factor

    // Normalize direction and scale to new distance
    const direction = camera.position.map(c => c / currentDistance) as [number, number, number]
    const newPosition = direction.map(d => d * newDistance) as [number, number, number]

    setCamera({ position: newPosition })
    renderStructure()
  }

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold flex items-center gap-2">
        <EyeIcon size={16} />
        Camera
      </h3>

      {/* Reset and Fit buttons */}
      <div className="grid grid-cols-2 gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleReset}
          className="gap-2"
        >
          <RotateCcwIcon size={14} />
          Reset
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleFitToView}
          disabled={!atoms}
          className="gap-2"
        >
          <MaximizeIcon size={14} />
          Fit
        </Button>
      </div>

      {/* Rotation Controls */}
      <div className="space-y-2">
        <label className="text-xs text-muted-foreground flex items-center gap-1">
          <RotateCwIcon size={12} />
          Rotation
        </label>
        <div className="space-y-2 pl-2">
          <div className="flex items-center gap-2">
            <span className="text-xs w-4 text-muted-foreground">X</span>
            <input
              type="range"
              min="-180"
              max="180"
              value={rotationX}
              onChange={(e) => handleRotation('x', parseFloat(e.target.value))}
              className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
            />
            <input
              type="number"
              value={rotationX}
              onChange={(e) => handleRotation('x', parseFloat(e.target.value) || 0)}
              className="w-14 px-1 py-0.5 text-xs bg-background border border-border rounded text-right"
            />
            <span className="text-xs text-muted-foreground">°</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs w-4 text-muted-foreground">Y</span>
            <input
              type="range"
              min="-180"
              max="180"
              value={rotationY}
              onChange={(e) => handleRotation('y', parseFloat(e.target.value))}
              className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
            />
            <input
              type="number"
              value={rotationY}
              onChange={(e) => handleRotation('y', parseFloat(e.target.value) || 0)}
              className="w-14 px-1 py-0.5 text-xs bg-background border border-border rounded text-right"
            />
            <span className="text-xs text-muted-foreground">°</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs w-4 text-muted-foreground">Z</span>
            <input
              type="range"
              min="-180"
              max="180"
              value={rotationZ}
              onChange={(e) => handleRotation('z', parseFloat(e.target.value))}
              className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
            />
            <input
              type="number"
              value={rotationZ}
              onChange={(e) => handleRotation('z', parseFloat(e.target.value) || 0)}
              className="w-14 px-1 py-0.5 text-xs bg-background border border-border rounded text-right"
            />
            <span className="text-xs text-muted-foreground">°</span>
          </div>
        </div>
      </div>

      {/* Zoom Controls */}
      <div className="space-y-2">
        <label className="text-xs text-muted-foreground flex items-center gap-1">
          <ZoomInIcon size={12} />
          Zoom
        </label>
        <div className="flex items-center gap-2 pl-2">
          <Button
            variant="outline"
            size="icon"
            onClick={() => handleZoomButton(1.2)}
            className="h-7 w-7"
          >
            <ZoomOutIcon size={14} />
          </Button>
          <input
            type="range"
            min="10"
            max="500"
            value={zoomLevel}
            onChange={(e) => handleZoomSlider(parseFloat(e.target.value))}
            className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
          />
          <Button
            variant="outline"
            size="icon"
            onClick={() => handleZoomButton(0.83)}
            className="h-7 w-7"
          >
            <ZoomInIcon size={14} />
          </Button>
          <span className="text-xs font-mono w-12 text-right">{zoomLevel}%</span>
        </div>
      </div>

      {/* Pan Controls */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="text-xs text-muted-foreground flex items-center gap-1">
            <MoveIcon size={12} />
            Pan
          </label>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleCenter}
            disabled={!atoms}
            className="h-6 px-2 text-xs gap-1"
          >
            <CircleDotIcon size={12} />
            Center
          </Button>
        </div>
        <div className="space-y-2 pl-2">
          <div className="flex items-center gap-2">
            <span className="text-xs w-4 text-muted-foreground">X</span>
            <input
              type="range"
              min="-50"
              max="50"
              value={panX}
              onChange={(e) => handlePan('x', parseFloat(e.target.value))}
              className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
            />
            <span className="text-xs font-mono w-10 text-right">{panX.toFixed(0)}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs w-4 text-muted-foreground">Y</span>
            <input
              type="range"
              min="-50"
              max="50"
              value={panY}
              onChange={(e) => handlePan('y', parseFloat(e.target.value))}
              className="flex-1 h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary"
            />
            <span className="text-xs font-mono w-10 text-right">{panY.toFixed(0)}</span>
          </div>
        </div>
      </div>

      {/* View Presets */}
      <div className="space-y-2">
        <label className="text-xs text-muted-foreground flex items-center gap-1">
          <BoxIcon size={12} />
          View Presets
        </label>
        <div className="grid grid-cols-2 gap-2">
          <Button
            variant={cameraPreset === 'top' ? 'default' : 'outline'}
            size="sm"
            onClick={() => handlePreset('top')}
            className="gap-2"
          >
            <MoveVerticalIcon size={14} />
            Top
          </Button>
          <Button
            variant={cameraPreset === 'side' ? 'default' : 'outline'}
            size="sm"
            onClick={() => handlePreset('side')}
            className="gap-2"
          >
            <MoveHorizontalIcon size={14} />
            Side
          </Button>
          <Button
            variant={cameraPreset === 'front' ? 'default' : 'outline'}
            size="sm"
            onClick={() => handlePreset('front')}
            className="gap-2"
          >
            <BoxIcon size={14} />
            Front
          </Button>
          <Button
            variant={cameraPreset === 'isometric' ? 'default' : 'outline'}
            size="sm"
            onClick={() => handlePreset('isometric')}
            className="gap-2"
          >
            <BoxIcon size={14} />
            Iso
          </Button>
        </div>
      </div>
    </div>
  )
}
