export type PlanTier = 'free' | 'pro';

const STORAGE_KEY = 'cacalm_plan';

export const PRO_CHANNELS = [
  'ch-birds',
  'ch-cafe',
  'ch-fire',
  'ch-stream',
  'ch-binaural',
  'ch-spatial'
] as const;

function getInitialPlan(): PlanTier {
  try {
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored === 'pro' || stored === 'free') return stored;
    }
  } catch {
    // Storage unavailable
  }
  return 'free';
}

const state = $state<{
  plan: PlanTier;
  showUpgradeModal: boolean;
}>({
  plan: typeof window === 'undefined' ? 'free' : getInitialPlan(),
  showUpgradeModal: false
});

export function getPro() {
  return {
    get plan() {
      return state.plan;
    },
    get isPro() {
      return state.plan === 'pro';
    },
    get showUpgradeModal() {
      return state.showUpgradeModal;
    },
    set showUpgradeModal(val: boolean) {
      state.showUpgradeModal = val;
    }
  };
}

export function setPlan(plan: PlanTier) {
  state.plan = plan;
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, plan);
    }
  } catch {
    // Ignore
  }
}

export function togglePlan() {
  setPlan(state.plan === 'pro' ? 'free' : 'pro');
}

export function upgradeToPro() {
  setPlan('pro');
  state.showUpgradeModal = false;
}

export function openUpgradeModal() {
  state.showUpgradeModal = true;
}

export function closeUpgradeModal() {
  state.showUpgradeModal = false;
}

export function isProChannel(id: string): boolean {
  return (PRO_CHANNELS as readonly string[]).includes(id);
}

export function isChannelLocked(id: string): boolean {
  return state.plan !== 'pro' && isProChannel(id);
}
