import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import type {
  AtomsData,
  CameraState,
  RenderConfig,
  SelectedAtom,
  SelectionQuery,
  StructureStats,
  DistanceMeasurement,
  AngleMeasurement,
} from '../types/axiom'

// Backend atom details response
interface AtomDetailsResponse {
  index: number
  element: number
  position: [number, number, number]
}

/**
 * Load structure from a file path
 */
export async function loadStructureFromPath(path: string): Promise<{
  atoms: AtomsData
  path: string
  name: string
  format: string
}> {
  const name = path.split('/').pop() || path.split('\\').pop() || 'unknown'
  const format = name.split('.').pop()?.toLowerCase() || 'unknown'

  // Validate file extension
  const validExtensions = ['pdb', 'xyz', 'gro', 'lammpstrj', 'lammps', 'cif']
  if (!validExtensions.includes(format)) {
    throw new Error(
      `Unsupported file format: .${format}. Supported formats: ${validExtensions.join(', ')}`,
    )
  }

  // Load structure via Tauri command
  const atoms = await invoke<AtomsData>('load_structure', {
    path,
    format,
  })

  return { atoms, path, name, format }
}

/**
 * Open file dialog and load structure
 */
export async function openStructure(): Promise<{
  atoms: AtomsData
  path: string
  name: string
  format: string
} | null> {
  // Open file dialog
  const result = await open({
    title: 'Open Molecular Structure',
    multiple: false,
    filters: [
      {
        name: 'Molecular Structures',
        extensions: ['pdb', 'xyz', 'gro', 'lammpstrj', 'lammps', 'cif'],
      },
      {
        name: 'PDB Files',
        extensions: ['pdb'],
      },
      {
        name: 'XYZ Files',
        extensions: ['xyz'],
      },
      {
        name: 'GROMACS Files',
        extensions: ['gro'],
      },
      {
        name: 'LAMMPS Files',
        extensions: ['lammpstrj', 'lammps'],
      },
      {
        name: 'CIF Files',
        extensions: ['cif'],
      },
    ],
  })

  if (!result) return null

  const path = result as string
  return await loadStructureFromPath(path)
}

/**
 * Render structure to PNG
 */
export async function renderStructure(
  config: RenderConfig,
  camera?: CameraState,
): Promise<Uint8Array> {
  return await invoke<Uint8Array>('render_structure', {
    config,
    camera,
  })
}

/**
 * Select atoms using semantic query
 */
export async function selectAtoms(query: string): Promise<SelectionQuery> {
  const indices = await invoke<number[]>('select_atoms', { query })
  return {
    query,
    indices,
    count: indices.length,
  }
}

/**
 * Get structure statistics
 */
export async function getStatistics(): Promise<StructureStats> {
  return await invoke<StructureStats>('get_statistics')
}

/**
 * Compute bonds for loaded structure
 */
export async function computeBonds(cutoff: number = 1.8): Promise<number> {
  return await invoke<number>('compute_bonds', { cutoff })
}

/**
 * Save image to file
 */
export async function saveImage(imageData: Uint8Array): Promise<boolean> {
  const result = await save({
    title: 'Save Image',
    filters: [
      {
        name: 'PNG Image',
        extensions: ['png'],
      },
    ],
    defaultPath: 'axiom-render.png',
  })

  if (!result) return false

  const savePath = result as string
  await invoke('save_image', {
    path: savePath,
    imageData: Array.from(imageData),
  })

  return true
}

/**
 * Get atom details by index
 */
export async function getAtomDetails(index: number): Promise<SelectedAtom> {
  const details = await invoke<AtomDetailsResponse>('get_atom_details', { index })
  return {
    index: details.index,
    element: details.element,
    position: details.position,
  }
}

/**
 * Pick atom at screen coordinates
 * Note: Backend implementation is simplified - returns null for now
 * Using client-side approach instead
 */
export async function pickAtomAtScreen(
  screenX: number,
  screenY: number,
  width: number,
  height: number,
): Promise<SelectedAtom | null> {
  const result = await invoke<AtomDetailsResponse | null>('pick_atom_at_screen', {
    screenX,
    screenY,
    width,
    height,
  })

  if (!result) return null

  return {
    index: result.index,
    element: result.element,
    position: result.position,
  }
}

/**
 * Convert Uint8Array to base64 data URL
 */
export function arrayBufferToDataURL(buffer: Uint8Array): string {
  const base64 = btoa(
    Array.from(buffer)
      .map((b) => String.fromCharCode(b))
      .join(''),
  )
  return `data:image/png;base64,${base64}`
}

/**
 * Export structure to file (PDB, XYZ, or CIF format)
 */
export async function exportStructure(
  format: 'pdb' | 'xyz' | 'cif',
): Promise<boolean> {
  const formatNames = {
    pdb: 'PDB File',
    xyz: 'XYZ File',
    cif: 'CIF File',
  }

  const result = await save({
    title: `Export Structure as ${format.toUpperCase()}`,
    filters: [
      {
        name: formatNames[format],
        extensions: [format],
      },
    ],
    defaultPath: `structure.${format}`,
  })

  if (!result) return false

  const savePath = result as string
  await invoke('export_structure', {
    path: savePath,
    format,
  })

  return true
}

/**
 * Export measurements to CSV file
 */
export async function exportMeasurements(
  distances: DistanceMeasurement[],
  angles: AngleMeasurement[],
): Promise<boolean> {
  const result = await save({
    title: 'Export Measurements to CSV',
    filters: [
      {
        name: 'CSV File',
        extensions: ['csv'],
      },
    ],
    defaultPath: 'measurements.csv',
  })

  if (!result) return false

  const savePath = result as string
  await invoke('export_measurements', {
    path: savePath,
    distances,
    angles,
  })

  return true
}
