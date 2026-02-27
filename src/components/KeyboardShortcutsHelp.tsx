import { X } from 'lucide-react'
import { useAxiomStore } from '../store/axiomStore'

/**
 * Keyboard shortcuts help dialog
 * WCAG 2.1 AA compliant with keyboard navigation
 */
export function KeyboardShortcutsHelp() {
  const showKeyboardHelp = useAxiomStore(state => state.showKeyboardHelp)

  if (!showKeyboardHelp) return null

  const shortcuts = [
    { category: 'File Operations', items: [
      { key: 'Ctrl+O', description: 'Open file' },
      { key: 'Ctrl+S', description: 'Save structure' },
      { key: 'Ctrl+E', description: 'Export screenshot' },
      { key: 'Ctrl+M', description: 'Export measurements (CSV)' },
    ]},
    { category: 'View Control', items: [
      { key: 'Ctrl+Z', description: 'Reset camera' },
      { key: 'F', description: 'Fit to view' },
      { key: 'R', description: 'Re-render' },
      { key: '1-4', description: 'Switch render mode (1: Ball-and-stick, 2: Spacefill, 3: Stick, 4: Wireframe)' },
    ]},
    { category: 'Selection', items: [
      { key: 'Ctrl+A', description: 'Select all atoms (max 1000)' },
      { key: 'Esc', description: 'Clear selection' },
    ]},
    { category: 'Help', items: [
      { key: '?', description: 'Show this help' },
    ]},
  ]

  const handleClose = () => {
    useAxiomStore.setState({ showKeyboardHelp: false })
  }

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={handleClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="keyboard-shortcuts-title"
    >
      <div
        className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 id="keyboard-shortcuts-title" className="text-lg font-semibold text-gray-900 dark:text-white">
            Keyboard Shortcuts
          </h2>
          <button
            onClick={handleClose}
            className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            aria-label="Close keyboard shortcuts help"
          >
            <X className="w-5 h-5 text-gray-500 dark:text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {shortcuts.map((section) => (
            <div key={section.category}>
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                {section.category}
              </h3>
              <div className="space-y-2">
                {section.items.map((shortcut) => (
                  <div
                    key={shortcut.key}
                    className="flex items-center justify-between py-2 px-3 bg-gray-50 dark:bg-gray-900 rounded"
                  >
                    <span className="text-sm text-gray-700 dark:text-gray-300">
                      {shortcut.description}
                    </span>
                    <kbd className="px-2 py-1 text-xs font-mono bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded shadow-sm">
                      {shortcut.key}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            Press <kbd className="px-1 py-0.5 text-xs font-mono bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded">Esc</kbd> or click outside to close
          </p>
        </div>
      </div>
    </div>
  )
}
