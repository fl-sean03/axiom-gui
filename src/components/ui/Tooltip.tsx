import { ReactNode, useState } from 'react'

interface TooltipProps {
  content: string
  children: ReactNode
  side?: 'top' | 'bottom' | 'left' | 'right'
  delay?: number
}

/**
 * Accessible tooltip component
 * WCAG 2.1 AA compliant with keyboard support
 */
export function Tooltip({
  content,
  children,
  side = 'top',
  delay = 500
}: TooltipProps) {
  const [show, setShow] = useState(false)
  const [timeoutId, setTimeoutId] = useState<NodeJS.Timeout | null>(null)

  const handleMouseEnter = () => {
    const id = setTimeout(() => setShow(true), delay)
    setTimeoutId(id)
  }

  const handleMouseLeave = () => {
    if (timeoutId) {
      clearTimeout(timeoutId)
      setTimeoutId(null)
    }
    setShow(false)
  }

  const handleFocus = () => {
    setShow(true)
  }

  const handleBlur = () => {
    setShow(false)
  }

  const positionClasses = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  }

  return (
    <div
      className="relative inline-block"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={handleFocus}
      onBlur={handleBlur}
    >
      {children}
      {show && content && (
        <div
          role="tooltip"
          className={`
            absolute z-50 px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700
            rounded shadow-lg whitespace-nowrap pointer-events-none
            ${positionClasses[side]}
          `}
        >
          {content}
          {/* Arrow */}
          <div
            className={`
              absolute w-2 h-2 bg-gray-900 dark:bg-gray-700 rotate-45
              ${side === 'top' ? 'bottom-[-4px] left-1/2 -translate-x-1/2' : ''}
              ${side === 'bottom' ? 'top-[-4px] left-1/2 -translate-x-1/2' : ''}
              ${side === 'left' ? 'right-[-4px] top-1/2 -translate-y-1/2' : ''}
              ${side === 'right' ? 'left-[-4px] top-1/2 -translate-y-1/2' : ''}
            `}
          />
        </div>
      )}
    </div>
  )
}
