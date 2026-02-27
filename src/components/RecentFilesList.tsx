import { ClockIcon, XIcon, FileIcon } from 'lucide-react'
import type { RecentFile } from '../types/axiom'
import { Button } from './ui/Button'

interface RecentFilesListProps {
  recentFiles: RecentFile[]
  onFileSelect: (file: RecentFile) => void
  onFileRemove: (path: string) => void
  onClearAll: () => void
}

/**
 * Recent Files List component - shows last 10 opened files
 */
export function RecentFilesList({
  recentFiles,
  onFileSelect,
  onFileRemove,
  onClearAll
}: RecentFilesListProps) {
  if (recentFiles.length === 0) {
    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold flex items-center gap-2">
            <ClockIcon size={14} />
            Recent Files
          </h3>
        </div>
        <div className="text-xs text-muted-foreground text-center py-4">
          No recent files
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold flex items-center gap-2">
          <ClockIcon size={14} />
          Recent Files
        </h3>
        {recentFiles.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onClearAll}
            title="Clear all recent files"
            className="h-6 px-2 text-xs"
          >
            Clear
          </Button>
        )}
      </div>

      <div className="space-y-1">
        {recentFiles.map((file) => (
          <div
            key={file.path}
            className="group flex items-center gap-2 p-2 rounded hover:bg-accent cursor-pointer transition-colors"
            onClick={() => onFileSelect(file)}
          >
            <FileIcon size={14} className="text-muted-foreground flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="text-xs font-medium truncate" title={file.name}>
                {file.name}
              </div>
              <div className="text-xs text-muted-foreground truncate" title={file.path}>
                {file.format.toUpperCase()} • {formatRelativeTime(file.timestamp)}
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
              onClick={(e) => {
                e.stopPropagation()
                onFileRemove(file.path)
              }}
              title="Remove from recent files"
            >
              <XIcon size={12} />
            </Button>
          </div>
        ))}
      </div>
    </div>
  )
}

/**
 * Format timestamp as relative time (e.g., "2 minutes ago", "1 hour ago")
 */
function formatRelativeTime(timestamp: number): string {
  const now = Date.now()
  const diffMs = now - timestamp
  const diffMinutes = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMinutes < 1) return 'just now'
  if (diffMinutes === 1) return '1 minute ago'
  if (diffMinutes < 60) return `${diffMinutes} minutes ago`
  if (diffHours === 1) return '1 hour ago'
  if (diffHours < 24) return `${diffHours} hours ago`
  if (diffDays === 1) return '1 day ago'
  if (diffDays < 7) return `${diffDays} days ago`

  // For older files, show the date
  const date = new Date(timestamp)
  return date.toLocaleDateString()
}
