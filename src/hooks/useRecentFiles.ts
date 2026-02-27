import { useState, useEffect } from 'react'
import type { RecentFile, FileInfo } from '../types/axiom'
import {
  loadRecentFiles,
  addRecentFile,
  clearRecentFiles,
  removeRecentFile
} from '../utils/recentFiles'

export interface UseRecentFilesReturn {
  recentFiles: RecentFile[]
  addRecent: (fileInfo: FileInfo) => void
  removeRecent: (path: string) => void
  clearRecents: () => void
}

/**
 * Hook for managing recent files list with localStorage persistence
 */
export function useRecentFiles(): UseRecentFilesReturn {
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([])

  // Load recent files from localStorage on mount
  useEffect(() => {
    const files = loadRecentFiles()
    setRecentFiles(files)
  }, [])

  // Add a file to recent files
  const addRecent = (fileInfo: FileInfo) => {
    const updated = addRecentFile(fileInfo)
    setRecentFiles(updated)
  }

  // Remove a file from recent files
  const removeRecent = (path: string) => {
    const updated = removeRecentFile(path)
    setRecentFiles(updated)
  }

  // Clear all recent files
  const clearRecents = () => {
    clearRecentFiles()
    setRecentFiles([])
  }

  return {
    recentFiles,
    addRecent,
    removeRecent,
    clearRecents
  }
}
