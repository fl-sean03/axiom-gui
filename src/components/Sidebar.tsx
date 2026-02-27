import { useAxiomStore } from '../store/axiomStore'
import { RenderControls } from './RenderControls'
import { RenderingSettingsPanel } from './RenderingSettingsPanel'
import { SelectionPanel } from './SelectionPanel'
import { AtomSelectionPanel } from './AtomSelectionPanel'
import { AtomPicker } from './AtomPicker'
import { CameraControls } from './CameraControls'
import { StatsPanel } from './StatsPanel'
import { RecentFilesList } from './RecentFilesList'
import { FileInfoPanel } from './FileInfoPanel'
import { ExportPanel } from './ExportPanel'
import { PerformanceMetricsPanel } from './PerformanceMetricsPanel'
import { ChevronLeftIcon, ChevronRightIcon } from 'lucide-react'
import { Button } from './ui/Button'
import { Tooltip } from './ui/Tooltip'
import { useRecentFilesContext } from '../contexts/RecentFilesContext'
import { loadStructureFromPath, getStatistics } from '../utils/tauri'
import { fitCameraToBox } from '../utils/camera'
import { useState } from 'react'
import type { RecentFile } from '../types/axiom'

/**
 * Left sidebar with controls (File Info, Rendering, Selection, Camera)
 */
export function Sidebar() {
  const {
    sidebarOpen,
    toggleSidebar,
    fileInfo,
    atoms,
    setAtoms,
    setStats,
    setCamera,
    setRenderState,
    setRenderProgress
  } = useAxiomStore()
  const recentFilesHook = useRecentFilesContext()
  const [loadingRecentFile, setLoadingRecentFile] = useState(false)

  // Handle loading a file from recent files list
  const handleRecentFileSelect = async (file: RecentFile) => {
    if (loadingRecentFile) return

    try {
      setLoadingRecentFile(true)
      setRenderState('loading')
      setRenderProgress(0)

      // Load structure from path
      const result = await loadStructureFromPath(file.path)
      setRenderProgress(30)

      const { atoms, path, name, format } = result

      // Set atoms and file info
      setAtoms(atoms, { path, name, format })
      setRenderProgress(50)

      // Fit camera to structure
      const newCamera = fitCameraToBox(atoms.bounds, 45)
      setCamera(newCamera)
      setRenderProgress(70)

      // Get statistics
      const stats = await getStatistics()
      setStats(stats)
      setRenderProgress(100)

      setRenderState('idle')
      setRenderProgress(null)

      // Note: Parent component should trigger render after this
    } catch (error) {
      console.error('Failed to load recent file:', error)
      setRenderState('error', String(error))
      setRenderProgress(null)

      // If file no longer exists, remove it from recent files
      if (String(error).includes('No such file') || String(error).includes('not found')) {
        recentFilesHook.removeRecent(file.path)
      }
    } finally {
      setLoadingRecentFile(false)
    }
  }

  if (!sidebarOpen) {
    return (
      <div className="w-12 bg-secondary border-r border-border flex items-start justify-center pt-4" role="complementary" aria-label="Sidebar">
        <Tooltip content="Show sidebar" side="right">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleSidebar}
            aria-label="Show sidebar"
          >
            <ChevronRightIcon size={20} />
          </Button>
        </Tooltip>
      </div>
    )
  }

  return (
    <aside className="w-80 min-w-[20rem] max-w-[24rem] bg-secondary border-r border-border flex flex-col resize-x overflow-auto" role="complementary" aria-label="Control panel">
      {/* Header */}
      <div className="h-12 border-b border-border px-4 flex items-center justify-between flex-shrink-0">
        <h2 className="font-semibold text-lg">Controls</h2>
        <Tooltip content="Hide sidebar" side="left">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleSidebar}
            aria-label="Hide sidebar"
          >
            <ChevronLeftIcon size={20} />
          </Button>
        </Tooltip>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6" role="region" aria-label="Control panels">
        {/* Recent Files */}
        <RecentFilesList
          recentFiles={recentFilesHook.recentFiles}
          onFileSelect={handleRecentFileSelect}
          onFileRemove={recentFilesHook.removeRecent}
          onClearAll={recentFilesHook.clearRecents}
        />

        {/* Divider */}
        {recentFilesHook.recentFiles.length > 0 && (
          <div className="border-t border-border" />
        )}

        {/* File Info */}
        <FileInfoPanel />

        {/* Divider */}
        {fileInfo && <div className="border-t border-border" />}

        {/* Rendering Controls */}
        <RenderControls />

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Rendering Settings (Phase 3) */}
        {atoms && <RenderingSettingsPanel />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Camera Controls */}
        {atoms && <CameraControls />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Atom Selection & Measurements (Phase 4) */}
        {atoms && <AtomSelectionPanel />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Atom Picker (Phase 4) */}
        {atoms && <AtomPicker />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Selection Panel (semantic query-based) */}
        {atoms && <SelectionPanel />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Export Panel (Phase 5) */}
        {atoms && <ExportPanel />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Performance Metrics Panel (Phase 6) */}
        {atoms && <PerformanceMetricsPanel />}

        {/* Divider */}
        {atoms && <div className="border-t border-border" />}

        {/* Stats Panel */}
        {atoms && <StatsPanel />}
      </div>
    </aside>
  )
}
