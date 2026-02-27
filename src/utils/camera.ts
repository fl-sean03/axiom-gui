import type { CameraState, BoundingBox } from '../types/axiom'

/**
 * Calculate camera position to fit bounding box
 */
export function fitCameraToBox(
  bounds: BoundingBox,
  fov: number = 45,
): CameraState {
  const { center, radius } = bounds
  const distance = radius / Math.tan((fov * Math.PI) / 360)
  const position: [number, number, number] = [
    center[0],
    center[1],
    center[2] + distance * 2,
  ]

  return {
    position,
    target: center,
    up: [0, 1, 0],
    fov,
  }
}

/**
 * Orbit camera around target
 * @param camera Current camera state
 * @param deltaX Horizontal angle delta (radians)
 * @param deltaY Vertical angle delta (radians)
 * @returns New camera state
 */
export function orbitCamera(
  camera: CameraState,
  deltaX: number,
  deltaY: number,
): CameraState {
  const { position, target, up } = camera

  // Vector from target to camera
  const offset = [
    position[0] - target[0],
    position[1] - target[1],
    position[2] - target[2],
  ]

  // Convert to spherical coordinates
  const radius = Math.sqrt(
    offset[0] ** 2 + offset[1] ** 2 + offset[2] ** 2,
  )
  let theta = Math.atan2(offset[0], offset[2]) // azimuthal angle
  let phi = Math.acos(Math.max(-1, Math.min(1, offset[1] / radius))) // polar angle

  // Apply deltas
  theta -= deltaX
  phi = Math.max(0.01, Math.min(Math.PI - 0.01, phi - deltaY))

  // Convert back to Cartesian
  const newOffset: [number, number, number] = [
    radius * Math.sin(phi) * Math.sin(theta),
    radius * Math.cos(phi),
    radius * Math.sin(phi) * Math.cos(theta),
  ]

  return {
    ...camera,
    position: [
      target[0] + newOffset[0],
      target[1] + newOffset[1],
      target[2] + newOffset[2],
    ],
  }
}

/**
 * Pan camera (translate target and position together)
 */
export function panCamera(
  camera: CameraState,
  deltaX: number,
  deltaY: number,
): CameraState {
  const { position, target } = camera

  // Calculate right and up vectors
  const forward = [
    target[0] - position[0],
    target[1] - position[1],
    target[2] - position[2],
  ]
  const forwardNorm = normalize(forward)
  const right = cross(forwardNorm, camera.up)
  const up = cross(right, forwardNorm)

  // Apply pan
  const newPosition: [number, number, number] = [
    position[0] + right[0] * deltaX + up[0] * deltaY,
    position[1] + right[1] * deltaX + up[1] * deltaY,
    position[2] + right[2] * deltaX + up[2] * deltaY,
  ]

  const newTarget: [number, number, number] = [
    target[0] + right[0] * deltaX + up[0] * deltaY,
    target[1] + right[1] * deltaX + up[1] * deltaY,
    target[2] + right[2] * deltaX + up[2] * deltaY,
  ]

  return {
    ...camera,
    position: newPosition,
    target: newTarget,
  }
}

/**
 * Zoom camera (move closer/farther from target)
 */
export function zoomCamera(camera: CameraState, delta: number): CameraState {
  const { position, target } = camera

  const direction = [
    target[0] - position[0],
    target[1] - position[1],
    target[2] - position[2],
  ]
  const distance = Math.sqrt(
    direction[0] ** 2 + direction[1] ** 2 + direction[2] ** 2,
  )

  // Prevent zooming too close or too far
  const newDistance = Math.max(1, Math.min(1000, distance + delta))
  const scale = newDistance / distance

  const newPosition: [number, number, number] = [
    target[0] - direction[0] * scale,
    target[1] - direction[1] * scale,
    target[2] - direction[2] * scale,
  ]

  return {
    ...camera,
    position: newPosition,
  }
}

// Vector math utilities
function normalize(v: number[]): number[] {
  const length = Math.sqrt(v[0] ** 2 + v[1] ** 2 + v[2] ** 2)
  return length > 0 ? [v[0] / length, v[1] / length, v[2] / length] : v
}

function cross(a: number[], b: number[]): number[] {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ]
}
