// Type definitions for Axiom GUI

export interface AtomsData {
  count: number
  elements: number[]
  positions: number[]
  bounds: BoundingBox
}

export interface BoundingBox {
  min: [number, number, number]
  max: [number, number, number]
  center: [number, number, number]
  radius: number
}

export interface CameraState {
  position: [number, number, number]
  target: [number, number, number]
  up: [number, number, number]
  fov: number
}

export interface RenderConfig {
  width: number
  height: number
  ssaa: 0 | 1 | 2 | 4
  enable_ao: boolean
  ao_samples: 4 | 8 | 16 | 32
  background: 'black' | 'white' | 'transparent'
}

export interface RenderSettings {
  renderStyle: 'ball-and-stick' | 'spacefill' | 'stick' | 'wireframe'
  atomScale: number // Atom size multiplier (0.05-2.0)
  bondRadius: number // Bond radius in Ångströms (0.05-0.5)
  ambient: number // Ambient light intensity (0-1)
  diffuse: number // Diffuse light intensity (0-1)
  specular: number // Specular light intensity (0-1)
  backgroundColor: string // Hex color code
}

export interface StructureStats {
  total_atoms: number
  element_counts: Record<number, number>
  has_bonds: boolean
  bond_count: number
}

export interface FileInfo {
  path: string
  name: string
  format: string
}

export interface RecentFile extends FileInfo {
  timestamp: number // Unix timestamp when file was opened
}

export interface SelectionQuery {
  query: string
  indices: number[]
  count: number
}

// Click-based selection for measurements and visualization
export interface SelectedAtom {
  index: number
  element: number
  position: [number, number, number]
  screenPosition?: [number, number] // 2D canvas coordinates for overlay
}

// Measurement types
export interface DistanceMeasurement {
  id: string
  atom1: SelectedAtom
  atom2: SelectedAtom
  distance: number // in Ångströms
}

export interface AngleMeasurement {
  id: string
  atom1: SelectedAtom
  atom2: SelectedAtom // vertex
  atom3: SelectedAtom
  angle: number // in degrees
}

export type RenderState = 'idle' | 'loading' | 'rendering' | 'error'

// Element symbols (atomic number → symbol)
export const ELEMENT_SYMBOLS: Record<number, string> = {
  1: 'H',
  6: 'C',
  7: 'N',
  8: 'O',
  9: 'F',
  11: 'Na',
  12: 'Mg',
  14: 'Si',
  15: 'P',
  16: 'S',
  17: 'Cl',
  19: 'K',
  20: 'Ca',
  22: 'Ti',
  26: 'Fe',
  27: 'Co', // Cobalt
  29: 'Cu',
  30: 'Zn',
  35: 'Br', // Bromine
  82: 'Pb', // Lead
  // Add more as needed
}

// CPK colors for elements (as hex)
export const ELEMENT_COLORS: Record<number, string> = {
  1: '#FFFFFF', // H - white
  6: '#909090', // C - gray
  7: '#3050F8', // N - blue
  8: '#FF0D0D', // O - red
  9: '#90E050', // F - green
  11: '#AB5CF2', // Na - purple
  12: '#8AFF00', // Mg - green
  14: '#F0C8A0', // Si - tan
  15: '#FF8000', // P - orange
  16: '#FFFF30', // S - yellow
  17: '#1FF01F', // Cl - green
  19: '#8F40D4', // K - purple
  20: '#3DFF00', // Ca - green
  22: '#BFC2C7', // Ti - gray
  26: '#E06633', // Fe - orange
  27: '#F090A0', // Co - pink
  29: '#C88033', // Cu - copper
  30: '#7D80B0', // Zn - blue-gray
  35: '#A62929', // Br - dark red
  82: '#575961', // Pb - dark gray
}
