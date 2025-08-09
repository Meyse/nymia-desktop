<script lang="ts">
// Component: src/lib/components/CustomDropdown.svelte
// Description: A clean and sophisticated custom dropdown component for dark theme.
// Changes:
// - Added support for displaying secondary text (balance) right-aligned in options
// - Updated Option interface to include optional balance field
// - Enhanced option rendering to show Name ← → Balance layout with skeleton loading
// - Added skeleton loading animations for both names and balances during progressive loading
// - Redesigned with cleaner, more modern dark styling (black/transparent backgrounds, white text)
// - Improved visual hierarchy with better contrast and subtle borders
// - Enhanced selected state with brand-green accent styling
// - Added support for displaying balance in the closed state (selectedOptionBalance prop)
// - UPDATED: Added useMonoFont prop to apply monospace font to option names (useful for addresses)
// - UPDATED: Added per-option font control (useMonoFont per option), removed checkmarks, made text smaller

  import { createEventDispatcher, tick } from 'svelte';
  import { fly, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Option {
    id: string | number | null;
    name: string;
    enabled: boolean;
    balance?: string | null; // Optional balance display
    useMonoFont?: boolean; // NEW: Per-option mono font control
  }

  export let options: Option[] = [];
  export let selectedId: string | number | null = null;
  export let label: string = '';
  export let placeholder: string = '-- Select --';
  export let disabled: boolean = false;
  export let selectedOptionBalance: string | null = null; // NEW: To show balance when closed
  export let useMonoFont: boolean = false; // NEW: Use mono font for names (useful for addresses)

  let isOpen = false;
  let selectedOptionName: string = placeholder;
  let dropdownElement: HTMLElement;

  const dispatch = createEventDispatcher<{ change: string | number | null }>();

  // Update selected name when selectedId prop changes externally
  $: {
      const foundOption = options.find(opt => opt.id === selectedId);
      selectedOptionName = foundOption ? foundOption.name : placeholder;
  }

  function toggleDropdown() {
    if (disabled) return;
    isOpen = !isOpen;
  }

  async function selectOption(option: Option) {
    if (!option.enabled || disabled) return;
    selectedId = option.id;
    selectedOptionName = option.name;
    isOpen = false;
    await tick();
    dispatch('change', selectedId);
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownElement && !dropdownElement.contains(event.target as Node)) {
      isOpen = false;
    }
  }
  
  function handleKeydown(event: KeyboardEvent, option: Option) {
      if (event.key === 'Enter') {
          selectOption(option);
      }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="relative w-full" bind:this={dropdownElement}>
  {#if label}
    <label class="block text-sm font-medium text-white mb-2">{label}</label>
  {/if}
  
  <button 
    type="button"
    class="relative w-full bg-black/60 border border-white/20 hover:border-brand-green/60 focus:border-brand-green rounded-lg shadow-lg pl-3 pr-10 py-2.5 text-left focus:outline-none focus:ring-2 focus:ring-brand-green/30 disabled:bg-black/30 disabled:opacity-50 disabled:cursor-not-allowed {isOpen ? 'border-brand-green ring-2 ring-brand-green/30 bg-black/80' : ''}"
    on:click={toggleDropdown}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    {disabled}
  >
    <div class="flex items-center justify-between">
      <span 
        class="block truncate text-sm font-medium {selectedId !== null ? 'text-white' : 'text-white/40'} {useMonoFont ? 'font-mono' : ''}"
      >
        {selectedOptionName}
      </span>

      {#if selectedOptionBalance && selectedId !== null}
        <span class="text-xs text-white/60 font-mono pr-8">
          {selectedOptionBalance}
        </span>
      {/if}
    </div>
    
    <span class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
      <svg 
        class="h-4 w-4 text-white/60 {isOpen ? 'rotate-180' : ''}"
        xmlns="http://www.w3.org/2000/svg" 
        viewBox="0 0 20 20" 
        fill="currentColor" 
        aria-hidden="true"
      >
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
      </svg>
    </span>
  </button>

  {#if isOpen}
    <div 
      class="absolute z-50 mt-1 w-full"
      transition:fly={{ y: -4, duration: 150, easing: cubicOut }}
    >
      <ul 
        class="bg-black/95 border border-white/10 shadow-2xl max-h-60 rounded-lg py-1 text-xs overflow-auto focus:outline-none backdrop-blur-sm"
        tabindex="-1"
        role="listbox"
        aria-labelledby="listbox-label"
        in:scale={{ duration: 100, start: 0.98, easing: cubicOut }}
      >
        {#each options as option (option.id)}
          <li 
            class="relative cursor-pointer select-none px-3 py-2.5 mx-1 rounded-md {option.enabled
              ? 'hover:bg-white/5 text-white border border-transparent hover:border-white/10'
              : 'cursor-not-allowed opacity-60 text-white/40'} {option.id === selectedId
              ? 'bg-brand-green/15 text-brand-green border border-brand-green/30 font-medium'
              : ''}"
            role="option"
            aria-selected={option.id === selectedId}
            on:click={() => selectOption(option)}
            on:keydown={(e) => handleKeydown(e, option)}
            tabindex="0"
          >
            <div class="flex items-center justify-between">
              <span class="block truncate {option.useMonoFont ? 'font-mono' : ''}">
                {#if option.name === 'Loading...'}
                  <div class="w-24 h-4 bg-white/10 rounded animate-pulse"></div>
                {:else}
                  {option.name}
                {/if}
              </span>
              
              <div class="flex items-center ml-2 flex-shrink-0">
                {#if option.balance !== undefined}
                  <span class="text-xs text-white/60 font-mono {option.id === selectedId ? 'text-brand-green/80' : ''}">
                    {#if option.balance === 'skeleton'}
                      <div class="w-16 h-3 bg-white/10 rounded animate-pulse"></div>
                    {:else}
                      {option.balance || '-'}
                    {/if}
                  </span>
                {/if}
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>