import { useState } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { useAxiom } from '../hooks/useAxiom'
import { Button } from './ui/Button'
import {
  SparklesIcon,
  PaletteIcon,
  MonitorIcon,
  InfoIcon,
  RotateCcwIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  LockIcon,
  UnlockIcon,
} from 'lucide-react'

/**
 * Enhanced rendering controls with quality presets, resolution settings,
 * and comprehensive configuration options
 */
export function RenderControls() {
  const { renderConfig, setRenderConfig, atoms, stats } = useAxiomStore()
  const { renderStructure } = useAxiom()

  // Collapsible section state
  const [qualityExpanded, setQualityExpanded] = useState(true)
  const [backgroundExpanded, setBackgroundExpanded] = useState(true)
  const [resolutionExpanded, setResolutionExpanded] = useState(false)
  const [performanceExpanded, setPerformanceExpanded] = useState(false)

  // Aspect ratio lock state
  const [aspectRatioLocked, setAspectRatioLocked] = useState(true)
  const [aspectRatio, setAspectRatio] = useState(
    renderConfig.width / renderConfig.height,
  )

  const handleSSAAChange = (ssaa: 0 | 1 | 2 | 4) => {
    setRenderConfig({ ssaa })
    renderStructure()
  }

  const handleAOToggle = () => {
    setRenderConfig({ enable_ao: !renderConfig.enable_ao })
    renderStructure()
  }

  const handleAOSamplesChange = (ao_samples: 4 | 8 | 16 | 32) => {
    setRenderConfig({ ao_samples })
    renderStructure()
  }

  const handleBackgroundChange = (
    background: 'black' | 'white' | 'transparent',
  ) => {
    setRenderConfig({ background })
    renderStructure()
  }

  // Quality presets
  const applyQualityPreset = (preset: 'low' | 'medium' | 'high' | 'ultra') => {
    const presets = {
      low: { ssaa: 0 as const, enable_ao: false, ao_samples: 4 as const },
      medium: { ssaa: 1 as const, enable_ao: true, ao_samples: 8 as const },
      high: { ssaa: 2 as const, enable_ao: true, ao_samples: 16 as const },
      ultra: { ssaa: 4 as const, enable_ao: true, ao_samples: 32 as const },
    }
    setRenderConfig(presets[preset])
    renderStructure()
  }

  // Get current quality preset (if matches)
  const getCurrentPreset = ():
    | 'low'
    | 'medium'
    | 'high'
    | 'ultra'
    | 'custom' => {
    const { ssaa, enable_ao, ao_samples } = renderConfig
    if (ssaa === 0 && !enable_ao) return 'low'
    if (ssaa === 1 && enable_ao && ao_samples === 8) return 'medium'
    if (ssaa === 2 && enable_ao && ao_samples === 16) return 'high'
    if (ssaa === 4 && enable_ao && ao_samples === 32) return 'ultra'
    return 'custom'
  }

  // Resolution presets
  const applyResolutionPreset = (preset: '720p' | '1080p' | '4k' | '8k') => {
    const presets = {
      '720p': { width: 1280, height: 720 },
      '1080p': { width: 1920, height: 1080 },
      '4k': { width: 3840, height: 2160 },
      '8k': { width: 7680, height: 4320 },
    }
    const { width, height } = presets[preset]
    setRenderConfig({ width, height })
    setAspectRatio(width / height)
    renderStructure()
  }

  // Handle width change with aspect ratio lock
  const handleWidthChange = (width: number) => {
    if (width < 100 || width > 16384) return
    if (aspectRatioLocked) {
      const height = Math.round(width / aspectRatio)
      setRenderConfig({ width, height })
    } else {
      setRenderConfig({ width })
    }
    renderStructure()
  }

  // Handle height change with aspect ratio lock
  const handleHeightChange = (height: number) => {
    if (height < 100 || height > 16384) return
    if (aspectRatioLocked) {
      const width = Math.round(height * aspectRatio)
      setRenderConfig({ width, height })
    } else {
      setRenderConfig({ height })
    }
    renderStructure()
  }

  // Toggle aspect ratio lock
  const toggleAspectRatioLock = () => {
    if (!aspectRatioLocked) {
      // Locking - save current ratio
      setAspectRatio(renderConfig.width / renderConfig.height)
    }
    setAspectRatioLocked(!aspectRatioLocked)
  }

  // Reset to defaults
  const resetToDefaults = () => {
    setRenderConfig({
      width: 1920,
      height: 1080,
      ssaa: 2,
      enable_ao: true,
      ao_samples: 8,
      background: 'black',
    })
    setAspectRatio(1920 / 1080)
    setAspectRatioLocked(true)
    renderStructure()
  }

  // Calculate estimated memory usage
  const estimateMemoryUsage = () => {
    const { width, height, ssaa } = renderConfig
    const effectiveWidth = width * (ssaa || 1)
    const effectiveHeight = height * (ssaa || 1)
    const pixels = effectiveWidth * effectiveHeight
    const bytesPerPixel = 4 // RGBA
    const mbytes = (pixels * bytesPerPixel) / (1024 * 1024)
    return mbytes.toFixed(1)
  }

  const SectionHeader = ({
    icon: Icon,
    title,
    expanded,
    onToggle,
  }: {
    icon: React.ElementType
    title: string
    expanded: boolean
    onToggle: () => void
  }) => (
    <button
      onClick={onToggle}
      className="flex w-full items-center justify-between rounded-md px-2 py-1 text-sm font-semibold transition-colors hover:bg-accent"
    >
      <div className="flex items-center gap-2">
        <Icon className="h-4 w-4 text-primary" />
        <span>{title}</span>
      </div>
      {expanded ? (
        <ChevronUpIcon className="h-4 w-4 text-muted-foreground" />
      ) : (
        <ChevronDownIcon className="h-4 w-4 text-muted-foreground" />
      )}
    </button>
  )

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold">Rendering</h3>

      {/* Quality Settings Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={SparklesIcon}
          title="Quality Settings"
          expanded={qualityExpanded}
          onToggle={() => setQualityExpanded(!qualityExpanded)}
        />

        {qualityExpanded && (
          <div className="space-y-3 pt-2">
            {/* Quality Presets */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">
                Quality Presets
              </label>
              <div className="grid grid-cols-2 gap-2">
                {(['low', 'medium', 'high', 'ultra'] as const).map((preset) => (
                  <Button
                    key={preset}
                    variant={
                      getCurrentPreset() === preset ? 'default' : 'outline'
                    }
                    size="sm"
                    onClick={() => applyQualityPreset(preset)}
                    disabled={!atoms}
                  >
                    {preset.charAt(0).toUpperCase() + preset.slice(1)}
                  </Button>
                ))}
              </div>
            </div>

            {/* SSAA */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">
                Antialiasing (SSAA)
              </label>
              <div className="flex gap-2">
                {[0, 1, 2, 4].map((value) => (
                  <Button
                    key={value}
                    variant={renderConfig.ssaa === value ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => handleSSAAChange(value as 0 | 1 | 2 | 4)}
                    disabled={!atoms}
                    className="flex-1"
                  >
                    {value === 0 ? 'Off' : `${value}x`}
                  </Button>
                ))}
              </div>
            </div>

            {/* Ambient Occlusion */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Ambient Occlusion
                </label>
                <Button
                  variant={renderConfig.enable_ao ? 'default' : 'outline'}
                  size="sm"
                  onClick={handleAOToggle}
                  disabled={!atoms}
                >
                  {renderConfig.enable_ao ? 'On' : 'Off'}
                </Button>
              </div>

              {/* AO Samples (only if AO enabled) */}
              {renderConfig.enable_ao && (
                <div className="space-y-2 pl-4">
                  <label className="text-xs text-muted-foreground">
                    Samples
                  </label>
                  <div className="flex gap-2">
                    {[4, 8, 16, 32].map((value) => (
                      <Button
                        key={value}
                        variant={
                          renderConfig.ao_samples === value
                            ? 'default'
                            : 'outline'
                        }
                        size="sm"
                        onClick={() =>
                          handleAOSamplesChange(value as 4 | 8 | 16 | 32)
                        }
                        disabled={!atoms}
                        className="flex-1"
                      >
                        {value}
                      </Button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Background Settings Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={PaletteIcon}
          title="Background"
          expanded={backgroundExpanded}
          onToggle={() => setBackgroundExpanded(!backgroundExpanded)}
        />

        {backgroundExpanded && (
          <div className="space-y-2 pt-2">
            <div className="flex gap-2">
              {[
                { value: 'black', label: 'Black' },
                { value: 'white', label: 'White' },
                { value: 'transparent', label: 'Trans' },
              ].map(({ value, label }) => (
                <Button
                  key={value}
                  variant={
                    renderConfig.background === value ? 'default' : 'outline'
                  }
                  size="sm"
                  onClick={() =>
                    handleBackgroundChange(
                      value as 'black' | 'white' | 'transparent',
                    )
                  }
                  disabled={!atoms}
                  className="flex-1"
                >
                  {label}
                </Button>
              ))}
            </div>

            {/* Background preview */}
            <div className="flex items-center gap-2 rounded border border-border p-2">
              <div
                className="h-8 w-8 rounded border-2 border-border"
                style={{
                  backgroundColor:
                    renderConfig.background === 'transparent'
                      ? 'transparent'
                      : renderConfig.background,
                  backgroundImage:
                    renderConfig.background === 'transparent'
                      ? 'linear-gradient(45deg, #ccc 25%, transparent 25%, transparent 75%, #ccc 75%, #ccc), linear-gradient(45deg, #ccc 25%, transparent 25%, transparent 75%, #ccc 75%, #ccc)'
                      : 'none',
                  backgroundSize: '8px 8px',
                  backgroundPosition: '0 0, 4px 4px',
                }}
              />
              <span className="text-xs text-muted-foreground">
                Preview: {renderConfig.background}
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Resolution Settings Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={MonitorIcon}
          title="Resolution"
          expanded={resolutionExpanded}
          onToggle={() => setResolutionExpanded(!resolutionExpanded)}
        />

        {resolutionExpanded && (
          <div className="space-y-3 pt-2">
            {/* Resolution Presets */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">Presets</label>
              <div className="grid grid-cols-2 gap-2">
                {(['720p', '1080p', '4k', '8k'] as const).map((preset) => (
                  <Button
                    key={preset}
                    variant="outline"
                    size="sm"
                    onClick={() => applyResolutionPreset(preset)}
                    disabled={!atoms}
                  >
                    {preset.toUpperCase()}
                  </Button>
                ))}
              </div>
            </div>

            {/* Custom Resolution */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Custom Resolution
                </label>
                <button
                  onClick={toggleAspectRatioLock}
                  className="rounded p-1 transition-colors hover:bg-accent"
                  title={
                    aspectRatioLocked
                      ? 'Aspect ratio locked'
                      : 'Aspect ratio unlocked'
                  }
                >
                  {aspectRatioLocked ? (
                    <LockIcon className="h-3 w-3 text-primary" />
                  ) : (
                    <UnlockIcon className="h-3 w-3 text-muted-foreground" />
                  )}
                </button>
              </div>
              <div className="flex gap-2">
                <div className="flex-1">
                  <input
                    type="number"
                    min="100"
                    max="16384"
                    value={renderConfig.width}
                    onChange={(e) => handleWidthChange(Number(e.target.value))}
                    disabled={!atoms}
                    className="w-full rounded border border-input bg-background px-2 py-1 text-sm font-mono"
                  />
                  <label className="text-xs text-muted-foreground">Width</label>
                </div>
                <div className="flex items-center text-muted-foreground">×</div>
                <div className="flex-1">
                  <input
                    type="number"
                    min="100"
                    max="16384"
                    value={renderConfig.height}
                    onChange={(e) => handleHeightChange(Number(e.target.value))}
                    disabled={!atoms}
                    className="w-full rounded border border-input bg-background px-2 py-1 text-sm font-mono"
                  />
                  <label className="text-xs text-muted-foreground">
                    Height
                  </label>
                </div>
              </div>
              <div className="text-xs text-muted-foreground">
                Aspect ratio: {(renderConfig.width / renderConfig.height).toFixed(2)}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Performance Info Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={InfoIcon}
          title="Performance Info"
          expanded={performanceExpanded}
          onToggle={() => setPerformanceExpanded(!performanceExpanded)}
        />

        {performanceExpanded && (
          <div className="space-y-2 pt-2 text-xs">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Resolution:</span>
              <span className="font-mono">
                {renderConfig.width} × {renderConfig.height}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Effective:</span>
              <span className="font-mono">
                {renderConfig.width * (renderConfig.ssaa || 1)} ×{' '}
                {renderConfig.height * (renderConfig.ssaa || 1)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">SSAA:</span>
              <span className="font-mono">
                {renderConfig.ssaa === 0 ? 'Off' : `${renderConfig.ssaa}x`}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">AO:</span>
              <span className="font-mono">
                {renderConfig.enable_ao
                  ? `${renderConfig.ao_samples} samples`
                  : 'Off'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Est. Memory:</span>
              <span className="font-mono">~{estimateMemoryUsage()} MB</span>
            </div>
            {stats && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Atoms:</span>
                <span className="font-mono">{stats.total_atoms.toLocaleString()}</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Reset Button */}
      <Button
        variant="outline"
        size="sm"
        onClick={resetToDefaults}
        disabled={!atoms}
        className="w-full"
      >
        <RotateCcwIcon className="mr-2 h-4 w-4" />
        Reset to Defaults
      </Button>
    </div>
  )
}
