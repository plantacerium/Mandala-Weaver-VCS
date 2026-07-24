import { create } from 'zustand';
import type { Monad, MandalaState } from '../../types/ontology';
import type { ZoomMode } from '../d3/zoom';

interface WorkspaceState {
  mandalaState: MandalaState | null;
  selectedMonad: Monad | null;
  hoveredMonad: Monad | null;
  viewMode: 'orbit' | 'rings' | 'vectors' | 'distill';
  selectedForDistill: Monad[];
  zoomMode: ZoomMode;
  lineageCache: Map<string, { monads: Monad[]; depth: number }>;
  
  setMandalaState: (state: MandalaState) => void;
  selectMonad: (monad: Monad | null) => void;
  hoverMonad: (monad: Monad | null) => void;
  setViewMode: (mode: 'orbit' | 'rings' | 'vectors' | 'distill') => void;
  setSelectedForDistill: (monads: Monad[]) => void;
  setZoomMode: (mode: ZoomMode) => void;
  setLineageCache: (monadId: string, data: { monads: Monad[]; depth: number }) => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  mandalaState: null,
  selectedMonad: null,
  hoveredMonad: null,
  viewMode: 'orbit',
  selectedForDistill: [],
  zoomMode: 'normal',
  lineageCache: new Map(),
  
  setMandalaState: (state) => set({ mandalaState: state }),
  selectMonad: (monad) => set({ selectedMonad: monad }),
  hoverMonad: (monad) => set({ hoveredMonad: monad }),
  setViewMode: (mode) => set({ viewMode: mode }),
  setSelectedForDistill: (monads) => set({ selectedForDistill: monads }),
  setZoomMode: (mode) => set({ zoomMode: mode }),
  setLineageCache: (monadId, data) => set((state) => {
    const newCache = new Map(state.lineageCache);
    newCache.set(monadId, data);
    return { lineageCache: newCache };
  }),
}));