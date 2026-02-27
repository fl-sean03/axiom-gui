import { useAxiom } from '../hooks/useAxiom'
import { useAxiomStore } from '../store/axiomStore'
import { Button } from './ui/Button'
import { Tooltip } from './ui/Tooltip'
import {
  FileIcon,
  ImageIcon,
  RotateCcwIcon,
  MaximizeIcon,
  HelpCircleIcon,
  KeyboardIcon,
} from 'lucide-react'

/**
 * Top toolbar with File, View, Help menus
 */
export function Toolbar() {
  const { atoms } = useAxiomStore()
  const { loadStructure, saveImage, renderStructure } = useAxiom()
  const { setCameraPreset, reset } = useAxiomStore()

  const handleResetCamera = () => {
    setCameraPreset('default')
    renderStructure()
  }

  const handleFitToScreen = () => {
    if (atoms) {
      setCameraPreset('default')
      renderStructure()
    }
  }

  return (
    <div className="h-12 bg-background border-b border-border px-4 flex items-center gap-6">
      {/* File Menu */}
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium text-muted-foreground">File</span>
        <Tooltip content="Open structure file (Ctrl+O)">
          <Button
            variant="ghost"
            size="sm"
            onClick={loadStructure}
            className="gap-2"
            aria-label="Open structure file"
          >
            <FileIcon size={16} />
            Open
          </Button>
        </Tooltip>
        <Tooltip content="Save rendered image (Ctrl+E)">
          <Button
            variant="ghost"
            size="sm"
            onClick={saveImage}
            disabled={!atoms}
            className="gap-2"
            aria-label="Save rendered image"
          >
            <ImageIcon size={16} />
            Save Image
          </Button>
        </Tooltip>
      </div>

      {/* View Menu */}
      <div className="flex items-center gap-2 border-l border-border pl-6">
        <span className="text-sm font-medium text-muted-foreground">View</span>
        <Tooltip content="Reset camera to default (Ctrl+Z)">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleResetCamera}
            disabled={!atoms}
            className="gap-2"
            aria-label="Reset camera to default view"
          >
            <RotateCcwIcon size={16} />
            Reset Camera
          </Button>
        </Tooltip>
        <Tooltip content="Fit structure to view (F)">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleFitToScreen}
            disabled={!atoms}
            className="gap-2"
            aria-label="Fit structure to screen"
          >
            <MaximizeIcon size={16} />
            Fit to Screen
          </Button>
        </Tooltip>
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Help */}
      <div className="flex items-center gap-2">
        <Tooltip content="Show keyboard shortcuts (?)">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => useAxiomStore.setState({ showKeyboardHelp: true })}
            className="gap-2"
            aria-label="Show keyboard shortcuts"
          >
            <KeyboardIcon size={16} />
            Shortcuts
          </Button>
        </Tooltip>
        <Tooltip content="About Axiom">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              // TODO: Show about dialog
              alert(
                'Axiom v0.1.0\\n\\nAgent-native molecular visualization\\n\\nBuilt by Heinz Interfaces Laboratory',
              )
            }}
            className="gap-2"
            aria-label="About Axiom"
          >
            <HelpCircleIcon size={16} />
            About
          </Button>
        </Tooltip>
      </div>
    </div>
  )
}
