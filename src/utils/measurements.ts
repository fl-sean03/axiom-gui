import type { SelectedAtom } from '../types/axiom'

/**
 * Calculate Euclidean distance between two atoms in 3D space
 * @param atom1 First atom
 * @param atom2 Second atom
 * @returns Distance in Ångströms
 */
export function calculateDistance(
  atom1: SelectedAtom,
  atom2: SelectedAtom,
): number {
  const [x1, y1, z1] = atom1.position
  const [x2, y2, z2] = atom2.position

  const dx = x2 - x1
  const dy = y2 - y1
  const dz = z2 - z1

  return Math.sqrt(dx * dx + dy * dy + dz * dz)
}

/**
 * Calculate angle between three atoms (atom2 is the vertex)
 * @param atom1 First atom
 * @param atom2 Vertex atom
 * @param atom3 Third atom
 * @returns Angle in degrees
 */
export function calculateAngle(
  atom1: SelectedAtom,
  atom2: SelectedAtom,
  atom3: SelectedAtom,
): number {
  const [x1, y1, z1] = atom1.position
  const [x2, y2, z2] = atom2.position
  const [x3, y3, z3] = atom3.position

  // Vector from atom2 to atom1
  const v1 = [x1 - x2, y1 - y2, z1 - z2]
  // Vector from atom2 to atom3
  const v2 = [x3 - x2, y3 - y2, z3 - z2]

  // Dot product
  const dot = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]

  // Magnitudes
  const mag1 = Math.sqrt(v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2])
  const mag2 = Math.sqrt(v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2])

  // Angle in radians
  const cosTheta = dot / (mag1 * mag2)
  // Clamp to [-1, 1] to handle floating point errors
  const clampedCosTheta = Math.max(-1, Math.min(1, cosTheta))
  const angleRad = Math.acos(clampedCosTheta)

  // Convert to degrees
  return (angleRad * 180) / Math.PI
}

/**
 * Format distance for display
 * @param distance Distance in Ångströms
 * @returns Formatted string
 */
export function formatDistance(distance: number): string {
  return `${distance.toFixed(2)} Å`
}

/**
 * Format angle for display
 * @param angle Angle in degrees
 * @returns Formatted string
 */
export function formatAngle(angle: number): string {
  return `${angle.toFixed(1)}°`
}

/**
 * Generate unique ID for measurements
 */
export function generateMeasurementId(): string {
  return `measure-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}
