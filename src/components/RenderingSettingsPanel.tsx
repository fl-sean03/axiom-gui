import { useState } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import { useAxiom } from '../hooks/useAxiom'
import { Button } from './ui/Button'
import {
  PaletteIcon,
  SunIcon,
  CircleIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  ZapIcon,
} from 'lucide-react'

/**
 * Phase 3: Rendering Settings Panel
 *
 * Comprehensive rendering style and appearance controls including:
 * - Render style selection (stick, ball-and-stick, spacefill, wireframe)
 * - Quality presets (low/medium/high/ultra) affecting SSAA and AO samples
 * - Atom size and bond width controls
 * - Lighting controls (ambient, diffuse, specular intensity)
 * - Background color picker with preset swatches
 * - Settings persistence to localStorage
 */
export function RenderingSettingsPanel() {
  const { renderSettings, setRenderSettings, renderConfig, setRenderConfig, atoms } = useAxiomStore()
  const { renderStructure } = useAxiom()

  // Collapsible section state
  const [styleExpanded, setStyleExpanded] = useState(true)
  const [qualityExpanded, setQualityExpanded] = useState(false)
  const [lightingExpanded, setLightingExpanded] = useState(false)
  const [backgroundExpanded, setBackgroundExpanded] = useState(false)

  // Custom background color state
  const [customBgColor, setCustomBgColor] = useState('#000000')

  // Render style options
  const renderStyles = [
    { value: 'ball-and-stick', label: 'Ball & Stick', icon: '⚛️' },
    { value: 'spacefill', label: 'Spacefill', icon: '🔵' },
    { value: 'stick', label: 'Stick', icon: '│' },
    { value: 'wireframe', label: 'Wireframe', icon: '◇' },
  ] as const

  // Quality presets
  type QualityPreset = 'low' | 'medium' | 'high' | 'ultra'
  const qualityPresets: Array<{
    value: QualityPreset
    label: string
    ssaa: 0 | 1 | 2 | 4
    ao_samples: 4 | 8 | 16 | 32
    description: string
  }> = [
    { value: 'low', label: 'Low', ssaa: 0, ao_samples: 4, description: 'Fastest' },
    { value: 'medium', label: 'Medium', ssaa: 1, ao_samples: 8, description: 'Balanced' },
    { value: 'high', label: 'High', ssaa: 2, ao_samples: 16, description: 'Sharp' },
    { value: 'ultra', label: 'Ultra', ssaa: 4, ao_samples: 32, description: 'Best quality' },
  ]

  // Determine current quality preset based on settings
  const getCurrentQuality = (): QualityPreset | null => {
    const preset = qualityPresets.find(
      p => p.ssaa === renderConfig.ssaa && p.ao_samples === renderConfig.ao_samples
    )
    return preset ? preset.value : null
  }

  // Background color presets
  const bgPresets = [
    { color: '#000000', label: 'Black' },
    { color: '#FFFFFF', label: 'White' },
    { color: '#1E293B', label: 'Slate' },
    { color: '#0F172A', label: 'Navy' },
    { color: '#374151', label: 'Gray' },
    { color: '#1F2937', label: 'Charcoal' },
  ]

  const handleRenderStyleChange = (style: typeof renderSettings.renderStyle) => {
    setRenderSettings({ renderStyle: style })
    // Adjust atom size defaults based on style
    if (style === 'spacefill') {
      setRenderSettings({ atomScale: 1.0 })
    } else if (style === 'ball-and-stick') {
      setRenderSettings({ atomScale: 0.3, bondRadius: 0.15 })
    } else if (style === 'stick') {
      setRenderSettings({ atomScale: 0.2, bondRadius: 0.15 })
    } else if (style === 'wireframe') {
      setRenderSettings({ atomScale: 0.1, bondRadius: 0.05 })
    }
    renderStructure()
  }

  const handleAtomScaleChange = (scale: number) => {
    setRenderSettings({ atomScale: scale })
    renderStructure()
  }

  const handleBondRadiusChange = (radius: number) => {
    setRenderSettings({ bondRadius: radius })
    renderStructure()
  }

  const handleLightingChange = (
    setting: 'ambient' | 'diffuse' | 'specular',
    value: number,
  ) => {
    setRenderSettings({ [setting]: value })
    renderStructure()
  }

  const handleQualityPresetChange = (preset: QualityPreset) => {
    const selected = qualityPresets.find(p => p.value === preset)
    if (selected) {
      setRenderConfig({
        ssaa: selected.ssaa,
        ao_samples: selected.ao_samples
      })
      renderStructure()
    }
  }

  const handleBackgroundColorChange = (color: string) => {
    setRenderSettings({ backgroundColor: color })
    setCustomBgColor(color)
    renderStructure()
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
      <h3 className="text-sm font-semibold">Rendering Style</h3>

      {/* Render Style Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={CircleIcon}
          title="Render Style"
          expanded={styleExpanded}
          onToggle={() => setStyleExpanded(!styleExpanded)}
        />

        {styleExpanded && (
          <div className="space-y-3 pt-2">
            {/* Style Selection */}
            <div className="grid grid-cols-2 gap-2">
              {renderStyles.map((style) => (
                <Button
                  key={style.value}
                  variant={
                    renderSettings.renderStyle === style.value
                      ? 'default'
                      : 'outline'
                  }
                  size="sm"
                  onClick={() => handleRenderStyleChange(style.value)}
                  disabled={!atoms}
                  className="flex items-center gap-2"
                >
                  <span className="text-base">{style.icon}</span>
                  <span className="text-xs">{style.label}</span>
                </Button>
              ))}
            </div>

            {/* Atom Size Slider */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Atom Size
                </label>
                <span className="text-xs font-mono text-foreground">
                  {renderSettings.atomScale.toFixed(2)}x
                </span>
              </div>
              <input
                type="range"
                min="0.05"
                max="2.0"
                step="0.05"
                value={renderSettings.atomScale}
                onChange={(e) => handleAtomScaleChange(Number(e.target.value))}
                disabled={!atoms}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>Small</span>
                <span>Large</span>
              </div>
            </div>

            {/* Bond Width Slider */}
            {renderSettings.renderStyle !== 'spacefill' && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <label className="text-xs text-muted-foreground">
                    Bond Width
                  </label>
                  <span className="text-xs font-mono text-foreground">
                    {renderSettings.bondRadius.toFixed(2)}Å
                  </span>
                </div>
                <input
                  type="range"
                  min="0.05"
                  max="0.5"
                  step="0.05"
                  value={renderSettings.bondRadius}
                  onChange={(e) =>
                    handleBondRadiusChange(Number(e.target.value))
                  }
                  disabled={!atoms}
                  className="w-full"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Thin</span>
                  <span>Thick</span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Quality Presets Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={ZapIcon}
          title="Quality"
          expanded={qualityExpanded}
          onToggle={() => setQualityExpanded(!qualityExpanded)}
        />

        {qualityExpanded && (
          <div className="space-y-3 pt-2">
            {/* Quality Preset Buttons */}
            <div className="grid grid-cols-2 gap-2">
              {qualityPresets.map((preset) => (
                <Button
                  key={preset.value}
                  variant={
                    getCurrentQuality() === preset.value
                      ? 'default'
                      : 'outline'
                  }
                  size="sm"
                  onClick={() => handleQualityPresetChange(preset.value)}
                  disabled={!atoms}
                  className="flex flex-col items-center gap-0.5 h-auto py-2"
                >
                  <span className="text-xs font-semibold">{preset.label}</span>
                  <span className="text-xs text-muted-foreground">
                    {preset.description}
                  </span>
                </Button>
              ))}
            </div>

            {/* Quality Details */}
            <div className="rounded bg-accent/50 p-2 text-xs text-muted-foreground space-y-1">
              <div className="flex justify-between">
                <span>SSAA:</span>
                <span className="font-mono text-foreground">
                  {renderConfig.ssaa === 0 ? 'Off' : `${renderConfig.ssaa}x`}
                </span>
              </div>
              <div className="flex justify-between">
                <span>AO Samples:</span>
                <span className="font-mono text-foreground">
                  {renderConfig.ao_samples}
                </span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Lighting Settings Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={SunIcon}
          title="Lighting"
          expanded={lightingExpanded}
          onToggle={() => setLightingExpanded(!lightingExpanded)}
        />

        {lightingExpanded && (
          <div className="space-y-3 pt-2">
            {/* Ambient Light */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Ambient
                </label>
                <span className="text-xs font-mono text-foreground">
                  {Math.round(renderSettings.ambient * 100)}%
                </span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={renderSettings.ambient}
                onChange={(e) =>
                  handleLightingChange('ambient', Number(e.target.value))
                }
                disabled={!atoms}
                className="w-full"
              />
            </div>

            {/* Diffuse Light */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Diffuse
                </label>
                <span className="text-xs font-mono text-foreground">
                  {Math.round(renderSettings.diffuse * 100)}%
                </span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={renderSettings.diffuse}
                onChange={(e) =>
                  handleLightingChange('diffuse', Number(e.target.value))
                }
                disabled={!atoms}
                className="w-full"
              />
            </div>

            {/* Specular Light */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-xs text-muted-foreground">
                  Specular
                </label>
                <span className="text-xs font-mono text-foreground">
                  {Math.round(renderSettings.specular * 100)}%
                </span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={renderSettings.specular}
                onChange={(e) =>
                  handleLightingChange('specular', Number(e.target.value))
                }
                disabled={!atoms}
                className="w-full"
              />
            </div>
          </div>
        )}
      </div>

      {/* Background Color Section */}
      <div className="space-y-2 rounded-lg border border-border p-3">
        <SectionHeader
          icon={PaletteIcon}
          title="Background"
          expanded={backgroundExpanded}
          onToggle={() => setBackgroundExpanded(!backgroundExpanded)}
        />

        {backgroundExpanded && (
          <div className="space-y-3 pt-2">
            {/* Color Presets */}
            <div className="grid grid-cols-3 gap-2">
              {bgPresets.map((preset) => (
                <button
                  key={preset.color}
                  onClick={() => handleBackgroundColorChange(preset.color)}
                  disabled={!atoms}
                  className={`flex flex-col items-center gap-1 rounded border-2 p-2 transition-all hover:scale-105 ${
                    renderSettings.backgroundColor === preset.color
                      ? 'border-primary'
                      : 'border-border'
                  }`}
                  title={preset.label}
                >
                  <div
                    className="h-6 w-6 rounded border border-border"
                    style={{ backgroundColor: preset.color }}
                  />
                  <span className="text-xs text-muted-foreground">
                    {preset.label}
                  </span>
                </button>
              ))}
            </div>

            {/* Custom Color Picker */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">
                Custom Color
              </label>
              <div className="flex gap-2">
                <input
                  type="color"
                  value={customBgColor}
                  onChange={(e) => setCustomBgColor(e.target.value)}
                  disabled={!atoms}
                  className="h-10 w-full cursor-pointer rounded border border-border"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleBackgroundColorChange(customBgColor)}
                  disabled={!atoms}
                >
                  Apply
                </Button>
              </div>
              <div className="text-xs text-muted-foreground font-mono">
                Current: {renderSettings.backgroundColor}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
