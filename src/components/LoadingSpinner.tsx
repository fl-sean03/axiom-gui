/**
 * Loading spinner component for rendering progress
 */
export function LoadingSpinner({
  message,
  progress,
}: {
  message?: string
  progress?: number | null
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-4">
      <div className="relative w-16 h-16">
        {progress !== null && progress !== undefined ? (
          // Progress circle
          <svg className="w-16 h-16 transform -rotate-90">
            <circle
              cx="32"
              cy="32"
              r="28"
              stroke="currentColor"
              strokeWidth="4"
              fill="none"
              className="text-gray-700"
            />
            <circle
              cx="32"
              cy="32"
              r="28"
              stroke="currentColor"
              strokeWidth="4"
              fill="none"
              strokeDasharray={`${2 * Math.PI * 28}`}
              strokeDashoffset={`${2 * Math.PI * 28 * (1 - progress / 100)}`}
              className="text-blue-500 transition-all duration-300"
              strokeLinecap="round"
            />
          </svg>
        ) : (
          // Spinning loader
          <>
            <div className="absolute inset-0 border-4 border-gray-700 rounded-full"></div>
            <div className="absolute inset-0 border-4 border-blue-500 rounded-full border-t-transparent animate-spin"></div>
          </>
        )}
      </div>
      {message && (
        <div className="text-white text-lg font-medium">{message}</div>
      )}
      {progress !== null && progress !== undefined && (
        <div className="text-blue-400 text-sm font-mono">{progress}%</div>
      )}
    </div>
  )
}
