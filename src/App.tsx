import { Toolbar } from './components/Toolbar'
import { Sidebar } from './components/Sidebar'
import { Canvas } from './components/Canvas'
import { StatusBar } from './components/StatusBar'
import { AutoLoadTest } from './components/AutoLoadTest'
import { ErrorBoundary } from './components/ErrorBoundary'
import { KeyboardShortcutsHelp } from './components/KeyboardShortcutsHelp'
import { RecentFilesProvider } from './contexts/RecentFilesContext'
import { useRenderSettingsPersistence } from './hooks/useRenderSettingsPersistence'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'

/**
 * Main App component - root layout
 */
function App() {
  // Persist render settings to localStorage
  useRenderSettingsPersistence()

  // Enable keyboard shortcuts
  useKeyboardShortcuts()

  return (
    <ErrorBoundary>
      <RecentFilesProvider>
      <div className="h-screen w-screen flex flex-col overflow-hidden">
        {/* Auto-load test component (for headless testing) */}
        <AutoLoadTest />

        {/* Top toolbar */}
        <Toolbar />

        {/* Main content area */}
        <div className="flex-1 flex overflow-hidden">
          {/* Left sidebar */}
          <Sidebar />

          {/* Center canvas */}
          <Canvas />
        </div>

        {/* Bottom status bar */}
        <StatusBar />
      </div>

      {/* Keyboard shortcuts help dialog */}
      <KeyboardShortcutsHelp />
    </RecentFilesProvider>
    </ErrorBoundary>
  )
}

export default App
