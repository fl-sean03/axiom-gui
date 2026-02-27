import { ReactNode } from 'react'

interface SkeletonProps {
  className?: string
  children?: ReactNode
}

/**
 * Skeleton loading component
 * Provides smooth loading state animations
 */
export function Skeleton({ className = '', children }: SkeletonProps) {
  return (
    <div
      className={`
        animate-pulse bg-gray-200 dark:bg-gray-700 rounded
        ${className}
      `}
      role="status"
      aria-label="Loading"
    >
      {children}
    </div>
  )
}

/**
 * Panel skeleton with common layout
 */
export function PanelSkeleton() {
  return (
    <div className="space-y-3 p-4">
      <Skeleton className="h-6 w-32" />
      <div className="space-y-2">
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-3/4" />
      </div>
    </div>
  )
}

/**
 * List item skeleton
 */
export function ListItemSkeleton() {
  return (
    <div className="flex items-center gap-3 p-3">
      <Skeleton className="h-10 w-10 rounded-full" />
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-3 w-2/3" />
      </div>
    </div>
  )
}
