<script context="module" lang="ts">
  export function showToast(message: string, _type: 'success' | 'error' | 'info' = 'info') {
    // Basic browser / console fallback toast
    if (typeof window !== 'undefined') {
      const el = document.createElement('div');
      el.className = 'fixed bottom-5 right-5 z-50 px-4 py-2 rounded-lg bg-teal-500 text-black font-semibold text-xs shadow-lg transition-all duration-300';
      el.innerText = message;
      document.body.appendChild(el);
      setTimeout(() => {
        el.style.opacity = '0';
        setTimeout(() => el.remove(), 300);
      }, 3000);
    }
  }
</script>
