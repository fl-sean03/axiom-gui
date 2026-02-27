import { useEffect } from 'react'
import { useAxiomStore } from '../store/axiomStore'

const STORAGE_KEY = 'axiom-render-settings'

/**
 * Hook to persist render settings to localStorage
 * Automatically saves on changes and restores on mount
 */
export function useRenderSettingsPersistence() {
  const { renderSettings, setRenderSettings } = useAxiomStore()

  // Load settings from localStorage on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored)
        setRenderSettings(parsed)
      }
    } catch (error) {
      console.error('Failed to load render settings from localStorage:', error)
    }
  }, [setRenderSettings])

  // Save settings to localStorage whenever they change
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(renderSettings))
    } catch (error) {
      console.error('Failed to save render settings to localStorage:', error)
    }
  }, [renderSettings])
}
