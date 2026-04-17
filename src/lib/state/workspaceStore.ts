import { create } from 'zustand';
import type { Monad, MandalaState } from '../../types/ontology';

interface WorkspaceState {
  mandalaState: MandalaState | null;
  selectedMonad: Monad | null;
  hoveredMonad: Monad | null;
  viewMode: 'orbit' | 'rings' | 'vectors' | 'distill';
  selectedForDistill: Monad[];
  
  setMandalaState: (state: MandalaState) => void;
  selectMonad: (monad: Monad | null) => void;
  hoverMonad: (monad: Monad | null) => void;
  setViewMode: (mode: 'orbit' | 'rings' | 'vectors' | 'distill') => void;
  setSelectedForDistill: (monads: Monad[]) => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  mandalaState: null,
  selectedMonad: null,
  hoveredMonad: null,
  viewMode: 'orbit',
  selectedForDistill: [],
  
  setMandalaState: (state) => set({ mandalaState: state }),
  selectMonad: (monad) => set({ selectedMonad: monad }),
  hoverMonad: (monad) => set({ hoveredMonad: monad }),
  setViewMode: (mode) => set({ viewMode: mode }),
  setSelectedForDistill: (monads) => set({ selectedForDistill: monads }),
}));