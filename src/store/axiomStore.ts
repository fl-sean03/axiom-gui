import { create } from 'zustand'
import type {
  AngleMeasurement,
  AtomsData,
  CameraState,
  DistanceMeasurement,
  FileInfo,
  RenderConfig,
  RenderSettings,
  RenderState,
  SelectedAtom,
  SelectionQuery,
  StructureStats,
} from '../types/axiom'

interface AxiomStore {
  // Structure data
  atoms: AtomsData | null
  fileInfo: FileInfo | null
  stats: StructureStats | null

  // Rendering
  renderConfig: RenderConfig
  renderSettings: RenderSettings
  renderImage: string | null // base64 PNG data URL
  renderState: RenderState
  renderError: string | null
  renderProgress: number | null // 0-100 for progress tracking

  // Camera
  camera: CameraState
  cameraPreset: 'default' | 'top' | 'side' | 'front' | 'isometric'

  // Selection (semantic query-based)
  selection: SelectionQuery | null

  // Click-based selection and measurements
  selectedAtoms: SelectedAtom[]
  distanceMeasurements: DistanceMeasurement[]
  angleMeasurements: AngleMeasurement[]

  // UI state
  sidebarOpen: boolean
  showStats: boolean
  showKeyboardHelp: boolean

  // Actions
  setAtoms: (atoms: AtomsData, fileInfo: FileInfo) => void
  setStats: (stats: StructureStats) => void
  setRenderConfig: (config: Partial<RenderConfig>) => void
  setRenderSettings: (settings: Partial<RenderSettings>) => void
  setRenderImage: (image: string) => void
  setRenderState: (state: RenderState, error?: string) => void
  setRenderProgress: (progress: number | null) => void
  setCamera: (camera: Partial<CameraState>) => void
  setCameraPreset: (preset: AxiomStore['cameraPreset']) => void
  setSelection: (selection: SelectionQuery | null) => void
  addSelectedAtom: (atom: SelectedAtom) => void
  removeSelectedAtom: (index: number) => void
  clearSelectedAtoms: () => void
  setSelectedAtoms: (atoms: SelectedAtom[]) => void
  addDistanceMeasurement: (measurement: DistanceMeasurement) => void
  addAngleMeasurement: (measurement: AngleMeasurement) => void
  clearMeasurements: () => void
  toggleSidebar: () => void
  toggleStats: () => void
  reset: () => void
}

const DEFAULT_CAMERA: CameraState = {
  position: [0, 0, 150], // Zoomed way out
  target: [0, 0, 0],
  up: [0, 1, 0],
  fov: 45,
}

const DEFAULT_RENDER_CONFIG: RenderConfig = {
  width: 1920,
  height: 1080,
  ssaa: 2,
  enable_ao: true,
  ao_samples: 8,
  background: 'black',
}

const DEFAULT_RENDER_SETTINGS: RenderSettings = {
  renderStyle: 'ball-and-stick',
  atomScale: 0.3,
  bondRadius: 0.15,
  ambient: 0.3,
  diffuse: 0.6,
  specular: 0.3,
  backgroundColor: '#000000',
}

export const useAxiomStore = create<AxiomStore>((set) => ({
  // Initial state
  atoms: null,
  fileInfo: null,
  stats: null,
  renderConfig: DEFAULT_RENDER_CONFIG,
  renderSettings: DEFAULT_RENDER_SETTINGS,
  renderImage: null,
  renderState: 'idle',
  renderError: null,
  renderProgress: null,
  camera: DEFAULT_CAMERA,
  cameraPreset: 'default',
  selection: null,
  selectedAtoms: [],
  distanceMeasurements: [],
  angleMeasurements: [],
  sidebarOpen: true,
  showStats: false,
  showKeyboardHelp: false,

  // Actions
  setAtoms: (atoms, fileInfo) =>
    set({
      atoms,
      fileInfo,
      renderState: 'idle',
      selection: null,
    }),

  setStats: (stats) => set({ stats }),

  setRenderConfig: (config) =>
    set((state) => ({
      renderConfig: { ...state.renderConfig, ...config },
    })),

  setRenderSettings: (settings) =>
    set((state) => ({
      renderSettings: { ...state.renderSettings, ...settings },
    })),

  setRenderImage: (image) =>
    set({
      renderImage: image,
      renderState: 'idle',
      renderError: null,
      renderProgress: null,
    }),

  setRenderState: (state, error) =>
    set({
      renderState: state,
      renderError: error || null,
    }),

  setRenderProgress: (progress) =>
    set({
      renderProgress: progress,
    }),

  setCamera: (camera) =>
    set((state) => ({
      camera: { ...state.camera, ...camera },
      cameraPreset: 'default',
    })),

  setCameraPreset: (preset) => {
    let position: [number, number, number]
    const target: [number, number, number] = [0, 0, 0]
    const up: [number, number, number] = [0, 1, 0]

    switch (preset) {
      case 'top':
        position = [0, 150, 0]
        break
      case 'side':
        position = [150, 0, 0]
        break
      case 'front':
        position = [0, 0, 150]
        break
      case 'isometric':
        position = [106.1, 106.1, 106.1] // sqrt(3) * 150 / 3 for each component
        break
      default:
        position = [0, 0, 150]
    }

    set({
      camera: { position, target, up, fov: 45 },
      cameraPreset: preset,
    })
  },

  setSelection: (selection) => set({ selection }),

  addSelectedAtom: (atom) =>
    set((state) => ({
      selectedAtoms: [...state.selectedAtoms, atom],
    })),

  removeSelectedAtom: (index) =>
    set((state) => ({
      selectedAtoms: state.selectedAtoms.filter((a) => a.index !== index),
    })),

  clearSelectedAtoms: () =>
    set({
      selectedAtoms: [],
      distanceMeasurements: [],
      angleMeasurements: [],
    }),

  setSelectedAtoms: (atoms) =>
    set({
      selectedAtoms: atoms,
    }),

  addDistanceMeasurement: (measurement) =>
    set((state) => ({
      distanceMeasurements: [...state.distanceMeasurements, measurement],
    })),

  addAngleMeasurement: (measurement) =>
    set((state) => ({
      angleMeasurements: [...state.angleMeasurements, measurement],
    })),

  clearMeasurements: () =>
    set({
      distanceMeasurements: [],
      angleMeasurements: [],
    }),

  toggleSidebar: () =>
    set((state) => ({ sidebarOpen: !state.sidebarOpen })),

  toggleStats: () =>
    set((state) => ({ showStats: !state.showStats })),

  reset: () =>
    set({
      atoms: null,
      fileInfo: null,
      stats: null,
      renderImage: null,
      renderState: 'idle',
      renderError: null,
      renderProgress: null,
      camera: DEFAULT_CAMERA,
      cameraPreset: 'default',
      selection: null,
      selectedAtoms: [],
      distanceMeasurements: [],
      angleMeasurements: [],
    }),
}))
