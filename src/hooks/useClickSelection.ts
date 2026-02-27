import { useCallback, useEffect } from 'react'
import { useAxiomStore } from '../store/axiomStore'
import * as tauri from '../utils/tauri'
import {
  calculateAngle,
  calculateDistance,
  generateMeasurementId,
} from '../utils/measurements'
import type { SelectedAtom } from '../types/axiom'

/**
 * Hook for click-based atom selection and measurements
 * Handles selection logic, multi-select with modifiers, and automatic measurement creation
 */
export function useClickSelection() {
  const {
    atoms,
    selectedAtoms,
    addSelectedAtom,
    removeSelectedAtom,
    clearSelectedAtoms,
    setSelectedAtoms,
    addDistanceMeasurement,
    addAngleMeasurement,
  } = useAxiomStore()

  /**
   * Select an atom by index
   * @param index Atom index
   * @param modifiers Keyboard modifiers (ctrl, shift)
   */
  const selectAtom = useCallback(
    async (index: number, modifiers: { ctrl: boolean; shift: boolean }) => {
      if (!atoms) return

      try {
        // Get atom details from backend
        const atomDetails = await tauri.getAtomDetails(index)

        // Check if atom is already selected
        const alreadySelected = selectedAtoms.some((a) => a.index === index)

        if (modifiers.ctrl) {
          // Ctrl: Toggle selection
          if (alreadySelected) {
            removeSelectedAtom(index)
          } else {
            addSelectedAtom(atomDetails)
            // Auto-create measurements based on selection count
            createMeasurementsIfNeeded([...selectedAtoms, atomDetails])
          }
        } else if (modifiers.shift) {
          // Shift: Add to selection without clearing
          if (!alreadySelected) {
            addSelectedAtom(atomDetails)
            createMeasurementsIfNeeded([...selectedAtoms, atomDetails])
          }
        } else {
          // No modifier: Replace selection
          setSelectedAtoms([atomDetails])
        }
      } catch (error) {
        console.error('Failed to select atom:', error)
      }
    },
    [
      atoms,
      selectedAtoms,
      addSelectedAtom,
      removeSelectedAtom,
      setSelectedAtoms,
    ],
  )

  /**
   * Auto-create measurements based on number of selected atoms
   * - 2 atoms: Create distance measurement
   * - 3 atoms: Create angle measurement (middle atom is vertex)
   */
  const createMeasurementsIfNeeded = useCallback(
    (atoms: SelectedAtom[]) => {
      if (atoms.length === 2) {
        // Create distance measurement
        const [atom1, atom2] = atoms
        const distance = calculateDistance(atom1, atom2)
        addDistanceMeasurement({
          id: generateMeasurementId(),
          atom1,
          atom2,
          distance,
        })
      } else if (atoms.length === 3) {
        // Create angle measurement (atom2 is vertex)
        const [atom1, atom2, atom3] = atoms
        const angle = calculateAngle(atom1, atom2, atom3)
        addAngleMeasurement({
          id: generateMeasurementId(),
          atom1,
          atom2,
          atom3,
          angle,
        })
      }
    },
    [addDistanceMeasurement, addAngleMeasurement],
  )

  /**
   * Clear all selections and measurements
   */
  const clearSelection = useCallback(() => {
    clearSelectedAtoms()
  }, [clearSelectedAtoms])

  /**
   * Handle keyboard shortcuts
   */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape: Clear selection
      if (e.key === 'Escape') {
        clearSelection()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [clearSelection])

  return {
    selectAtom,
    clearSelection,
  }
}
