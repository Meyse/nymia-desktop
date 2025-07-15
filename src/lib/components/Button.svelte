<!-- 
  Component: src/lib/components/Button.svelte
  Description: Reusable button component with support for icons and various styles
  Supports both primary (green with darker hover) and secondary (transparent with light border) button styles
  with disabled states, click handlers, loading states, and icon support
  Changes:
  - Added icon support with optional icon slot or iconComponent prop
  - Added loading state support with spinner
  - Enhanced styling and accessibility
  - Improved hover states for better visual feedback
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Loader2 } from 'lucide-svelte';

  // Props
  export let variant: 'primary' | 'secondary' | 'white' = 'primary';
  export let size: 'normal' | 'small' = 'normal';
  export let disabled: boolean = false;
  export let loading: boolean = false;
  export let loadingText: string = 'Loading...';
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let iconComponent: any = null; // Svelte component for icon
  export let iconSize: number = 14;

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    click: void;
  }>();

  // Dynamic classes based on variant
  $: buttonClasses = `
    rounded-md shadow-sm text-xs font-medium select-none 
    focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-brand-green 
    flex items-center justify-center
    
    ${size === 'normal' ? 'py-2 px-3' : 'py-1.5 px-2.5'}

    ${disabled || loading ? 'disabled:opacity-50 disabled:cursor-not-allowed' : ''}
    ${variant === 'primary' 
      ? 'btn-primary border border-transparent text-white bg-brand-green hover:bg-[#32905D]' 
      : variant === 'secondary'
      ? 'btn-secondary border border-white/20 text-dark-text-primary bg-transparent hover:border-white/40'
      : 'btn-white border border-transparent text-black bg-white hover:bg-gray-100'
    }
  `.trim();

  function handleClick() {
    if (!disabled && !loading) {
      dispatch('click');
    }
  }
</script>

<button 
  {type}
  disabled={disabled || loading}
  class={buttonClasses}
  on:click={handleClick}
>
  {#if loading}
    <Loader2 size={iconSize} class="animate-spin mr-1.5" />
    <span>{loadingText}</span>
  {:else}
    {#if $$slots.icon}
      <span class="mr-1.5">
        <slot name="icon" />
      </span>
    {:else if iconComponent}
      <svelte:component this={iconComponent} size={iconSize} class="mr-1.5" />
    {/if}
    <slot />
  {/if}
</button>

<style>
  .btn-primary:disabled {
    background-color: #9fcfb8;
  }
</style> 