import { createContext, useContext, ReactNode } from 'react'
import { useRecentFiles, UseRecentFilesReturn } from '../hooks/useRecentFiles'

const RecentFilesContext = createContext<UseRecentFilesReturn | null>(null)

export function RecentFilesProvider({ children }: { children: ReactNode }) {
  const recentFiles = useRecentFiles()

  return (
    <RecentFilesContext.Provider value={recentFiles}>
      {children}
    </RecentFilesContext.Provider>
  )
}

export function useRecentFilesContext(): UseRecentFilesReturn {
  const context = useContext(RecentFilesContext)
  if (!context) {
    throw new Error('useRecentFilesContext must be used within RecentFilesProvider')
  }
  return context
}
