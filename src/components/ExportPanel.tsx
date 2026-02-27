import { useState } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { Button } from './ui/Button'
import { Download, Image, FileText, Table } from 'lucide-react'
import { renderStructure, saveImage, exportStructure, exportMeasurements } from '../utils/tauri'

type ExportFormat = 'pdb' | 'xyz' | 'cif'
type ResolutionPreset = '1080p' | '4K' | '8K' | 'custom'

interface ResolutionSettings {
  width: number
  height: number
}

const RESOLUTION_PRESETS: Record<ResolutionPreset, ResolutionSettings> = {
  '1080p': { width: 1920, height: 1080 },
  '4K': { width: 3840, height: 2160 },
  '8K': { width: 7680, height: 4320 },
  custom: { width: 1920, height: 1080 },
}

/**
 * Export Panel Component (Phase 5)
 *
 * Provides export functionality for:
 * - PNG screenshots with resolution/quality settings
 * - Structure files (PDB, XYZ, CIF formats)
 * - Measurement data to CSV (distances + angles)
 */
export function ExportPanel() {
  const {
    atoms,
    camera,
    renderConfig,
    distanceMeasurements,
    angleMeasurements,
  } = useAxiomStore()

  // Screenshot export state
  const [resolutionPreset, setResolutionPreset] = useState<ResolutionPreset>('1080p')
  const [customWidth, setCustomWidth] = useState(1920)
  const [customHeight, setCustomHeight] = useState(1080)
  const [ssaa, setSsaa] = useState<0 | 1 | 2 | 4>(2)
  const [screenshotExporting, setScreenshotExporting] = useState(false)

  // Structure export state
  const [structureFormat, setStructureFormat] = useState<ExportFormat>('pdb')
  const [structureExporting, setStructureExporting] = useState(false)

  // Measurement export state
  const [measurementExporting, setMeasurementExporting] = useState(false)

  if (!atoms) {
    return null
  }

  const hasMeasurements = distanceMeasurements.length > 0 || angleMeasurements.length > 0

  /**
   * Export current view as PNG screenshot
   */
  const handleScreenshotExport = async () => {
    if (screenshotExporting) return

    try {
      setScreenshotExporting(true)

      // Determine resolution
      const resolution = resolutionPreset === 'custom'
        ? { width: customWidth, height: customHeight }
        : RESOLUTION_PRESETS[resolutionPreset]

      // Render with specified settings
      const imageData = await renderStructure(
        {
          ...renderConfig,
          width: resolution.width,
          height: resolution.height,
          ssaa,
        },
        camera,
      )

      // Save to file
      const saved = await saveImage(imageData)
      if (!saved) {
        console.log('Screenshot export cancelled by user')
      }
    } catch (error) {
      console.error('Failed to export screenshot:', error)
      alert(`Screenshot export failed: ${error}`)
    } finally {
      setScreenshotExporting(false)
    }
  }

  /**
   * Export structure to file (PDB, XYZ, or CIF)
   */
  const handleStructureExport = async () => {
    if (structureExporting) return

    try {
      setStructureExporting(true)

      const success = await exportStructure(structureFormat)
      if (!success) {
        console.log('Structure export cancelled by user')
      }
    } catch (error) {
      console.error('Failed to export structure:', error)
      alert(`Structure export failed: ${error}`)
    } finally {
      setStructureExporting(false)
    }
  }

  /**
   * Export measurements to CSV
   */
  const handleMeasurementExport = async () => {
    if (measurementExporting) return

    try {
      setMeasurementExporting(true)

      const success = await exportMeasurements(distanceMeasurements, angleMeasurements)
      if (!success) {
        console.log('Measurement export cancelled by user')
      }
    } catch (error) {
      console.error('Failed to export measurements:', error)
      alert(`Measurement export failed: ${error}`)
    } finally {
      setMeasurementExporting(false)
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <Download size={16} className="text-muted-foreground" />
        <h3 className="text-sm font-semibold">Export</h3>
      </div>

      {/* Screenshot Export */}
      <div className="space-y-3 p-3 bg-background rounded-md border border-border">
        <div className="flex items-center gap-2">
          <Image size={14} className="text-muted-foreground" />
          <span className="text-sm font-medium">Screenshot</span>
        </div>

        {/* Resolution Preset */}
        <div className="space-y-1">
          <label className="text-xs text-muted-foreground">Resolution</label>
          <select
            value={resolutionPreset}
            onChange={(e) => setResolutionPreset(e.target.value as ResolutionPreset)}
            className="w-full px-2 py-1.5 text-sm bg-secondary border border-border rounded"
          >
            <option value="1080p">1080p (1920×1080)</option>
            <option value="4K">4K (3840×2160)</option>
            <option value="8K">8K (7680×4320)</option>
            <option value="custom">Custom</option>
          </select>
        </div>

        {/* Custom Resolution */}
        {resolutionPreset === 'custom' && (
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">Width</label>
              <input
                type="number"
                value={customWidth}
                onChange={(e) => setCustomWidth(Number(e.target.value))}
                min={640}
                max={15360}
                className="w-full px-2 py-1.5 text-sm bg-secondary border border-border rounded"
              />
            </div>
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">Height</label>
              <input
                type="number"
                value={customHeight}
                onChange={(e) => setCustomHeight(Number(e.target.value))}
                min={480}
                max={8640}
                className="w-full px-2 py-1.5 text-sm bg-secondary border border-border rounded"
              />
            </div>
          </div>
        )}

        {/* Quality (SSAA) */}
        <div className="space-y-1">
          <label className="text-xs text-muted-foreground">Quality (SSAA)</label>
          <select
            value={ssaa}
            onChange={(e) => setSsaa(Number(e.target.value) as 0 | 1 | 2 | 4)}
            className="w-full px-2 py-1.5 text-sm bg-secondary border border-border rounded"
          >
            <option value={0}>None (Fastest)</option>
            <option value={1}>1× (Good)</option>
            <option value={2}>2× (Better)</option>
            <option value={4}>4× (Best)</option>
          </select>
        </div>

        <Button
          onClick={handleScreenshotExport}
          disabled={screenshotExporting}
          className="w-full"
          size="sm"
        >
          {screenshotExporting ? 'Exporting...' : 'Export PNG'}
        </Button>
      </div>

      {/* Structure File Export */}
      <div className="space-y-3 p-3 bg-background rounded-md border border-border">
        <div className="flex items-center gap-2">
          <FileText size={14} className="text-muted-foreground" />
          <span className="text-sm font-medium">Structure File</span>
        </div>

        <div className="space-y-1">
          <label className="text-xs text-muted-foreground">Format</label>
          <select
            value={structureFormat}
            onChange={(e) => setStructureFormat(e.target.value as ExportFormat)}
            className="w-full px-2 py-1.5 text-sm bg-secondary border border-border rounded"
          >
            <option value="pdb">PDB (Protein Data Bank)</option>
            <option value="xyz">XYZ (Cartesian Coordinates)</option>
            <option value="cif">CIF (Crystallographic)</option>
          </select>
        </div>

        <Button
          onClick={handleStructureExport}
          disabled={structureExporting}
          className="w-full"
          size="sm"
        >
          {structureExporting ? 'Exporting...' : `Export ${structureFormat.toUpperCase()}`}
        </Button>
      </div>

      {/* Measurement Export */}
      <div className="space-y-3 p-3 bg-background rounded-md border border-border">
        <div className="flex items-center gap-2">
          <Table size={14} className="text-muted-foreground" />
          <span className="text-sm font-medium">Measurements</span>
        </div>

        <div className="text-xs text-muted-foreground">
          {hasMeasurements ? (
            <>
              {distanceMeasurements.length} distance{distanceMeasurements.length !== 1 ? 's' : ''},
              {' '}
              {angleMeasurements.length} angle{angleMeasurements.length !== 1 ? 's' : ''}
            </>
          ) : (
            'No measurements to export'
          )}
        </div>

        <Button
          onClick={handleMeasurementExport}
          disabled={measurementExporting || !hasMeasurements}
          className="w-full"
          size="sm"
        >
          {measurementExporting ? 'Exporting...' : 'Export to CSV'}
        </Button>
      </div>
    </div>
  )
}
