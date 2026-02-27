import React, { useEffect, useState } from 'react';
import { Activity, Cpu, Layers, Zap, Eye, EyeOff } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface PerformanceMetrics {
  avgFps: number;
  avgRenderMs: number;
  atomsTotal: number;
  atomsRendered: number;
  atomsCulled: number;
  lodHigh: number;
  lodMedium: number;
  lodLow: number;
  lodMinimal: number;
  sampleCount: number;
}

interface OctreeStats {
  totalNodes: number;
  totalAtoms: number;
  maxDepth: number;
}

export const PerformanceMetricsPanel: React.FC = () => {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [octreeStats, setOctreeStats] = useState<OctreeStats | null>(null);
  const [isExpanded, setIsExpanded] = useState(true);

  useEffect(() => {
    // Poll performance metrics every 500ms
    const interval = setInterval(async () => {
      try {
        const perfMetrics = await invoke<PerformanceMetrics>('get_performance_metrics');
        setMetrics(perfMetrics);

        const octree = await invoke<OctreeStats | null>('get_octree_stats');
        setOctreeStats(octree);
      } catch (error) {
        // Silently fail if no metrics available (renderer not initialized)
        setMetrics(null);
        setOctreeStats(null);
      }
    }, 500);

    return () => clearInterval(interval);
  }, []);

  const formatNumber = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const cullingEfficiency = metrics
    ? ((metrics.atomsCulled / metrics.atomsTotal) * 100).toFixed(1)
    : '0.0';

  const renderEfficiency = metrics
    ? ((metrics.atomsRendered / metrics.atomsTotal) * 100).toFixed(1)
    : '0.0';

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
      {/* Header */}
      <div
        className="flex items-center justify-between px-3 py-2 bg-gray-750 cursor-pointer hover:bg-gray-700 transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-blue-400" />
          <span className="font-semibold text-sm">Performance Metrics</span>
        </div>
        <button className="text-gray-400 hover:text-white transition-colors">
          {isExpanded ? (
            <Eye className="w-4 h-4" />
          ) : (
            <EyeOff className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* Content */}
      {isExpanded && (
        <div className="p-3 space-y-3 text-sm">
          {/* FPS and Render Time */}
          <div className="grid grid-cols-2 gap-2">
            <div className="bg-gray-900 rounded p-2">
              <div className="flex items-center gap-1 text-gray-400 mb-1">
                <Zap className="w-3 h-3" />
                <span className="text-xs">FPS</span>
              </div>
              <div className="text-lg font-bold text-green-400">
                {metrics?.avgFps.toFixed(1) ?? '--'}
              </div>
            </div>

            <div className="bg-gray-900 rounded p-2">
              <div className="flex items-center gap-1 text-gray-400 mb-1">
                <Cpu className="w-3 h-3" />
                <span className="text-xs">Render Time</span>
              </div>
              <div className="text-lg font-bold text-blue-400">
                {metrics?.avgRenderMs.toFixed(1) ?? '--'}
                <span className="text-xs text-gray-500 ml-1">ms</span>
              </div>
            </div>
          </div>

          {/* Atom Counts */}
          <div className="space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-gray-400">Total Atoms</span>
              <span className="font-semibold text-white">
                {metrics ? formatNumber(metrics.atomsTotal) : '--'}
              </span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-gray-400">Rendered</span>
              <span className="font-semibold text-green-400">
                {metrics ? formatNumber(metrics.atomsRendered) : '--'}
                <span className="text-gray-500 ml-1">({renderEfficiency}%)</span>
              </span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-gray-400">Culled (Optimized)</span>
              <span className="font-semibold text-yellow-400">
                {metrics ? formatNumber(metrics.atomsCulled) : '--'}
                <span className="text-gray-500 ml-1">({cullingEfficiency}%)</span>
              </span>
            </div>
          </div>

          {/* LOD Distribution */}
          {metrics && (metrics.lodHigh + metrics.lodMedium + metrics.lodLow + metrics.lodMinimal > 0) && (
            <div className="space-y-1">
              <div className="flex items-center gap-1 text-gray-400 mb-1">
                <Layers className="w-3 h-3" />
                <span className="text-xs font-semibold">Level of Detail</span>
              </div>
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">High</span>
                  <span className="text-green-400 font-semibold">
                    {formatNumber(metrics.lodHigh)}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">Medium</span>
                  <span className="text-blue-400 font-semibold">
                    {formatNumber(metrics.lodMedium)}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">Low</span>
                  <span className="text-yellow-400 font-semibold">
                    {formatNumber(metrics.lodLow)}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">Minimal</span>
                  <span className="text-red-400 font-semibold">
                    {formatNumber(metrics.lodMinimal)}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Octree Stats (if available) */}
          {octreeStats && (
            <div className="space-y-1 pt-2 border-t border-gray-700">
              <div className="text-xs font-semibold text-gray-400 mb-1">
                Octree Spatial Index
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Nodes</span>
                <span className="text-white font-semibold">
                  {formatNumber(octreeStats.totalNodes)}
                </span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Max Depth</span>
                <span className="text-white font-semibold">
                  {octreeStats.maxDepth}
                </span>
              </div>
            </div>
          )}

          {/* No Data Message */}
          {!metrics && (
            <div className="text-center text-gray-500 text-xs py-2">
              No performance data available
              <br />
              <span className="text-gray-600 text-xs">Load a structure to see metrics</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
