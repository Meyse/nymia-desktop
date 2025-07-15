<!-- 
  Component: src/lib/components/onboarding/VerusIdInfoModal.svelte
  Description: Modal popup that explains VerusID features and benefits
  Displays detailed information about VerusID's decentralized identity system
  Features clean overlay design with proper accessibility
  Changes:
  - Updated the modal content to be more user-friendly and benefit-oriented.
  - The new text focuses on data ownership, universal identity, and security.
-->
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import Button from '../Button.svelte';

  const dispatch = createEventDispatcher<{ close: void }>();

  function handleClose() {
    dispatch('close');
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleClose();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  // Prevent background scrolling when modal is open
  onMount(() => {
    document.body.style.overflow = 'hidden';
  });

  onDestroy(() => {
    document.body.style.overflow = '';
  });
</script>

<svelte:window on:keydown={handleKeyDown} />

<!-- Modal Backdrop -->
<div 
  class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
  on:click={handleBackdropClick}
  transition:fade={{ duration: 200 }}
>
  <!-- Modal Content -->
  <div 
    class="bg-dark-bg-primary border border-dark-border-primary rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto"
    transition:scale={{ duration: 200, easing: quintOut }}
  >
    <!-- Content -->
    <div class="p-6 space-y-6 text-dark-text-primary select-none cursor-default">
      
      <!-- Your personal database -->
      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          Your personal database = Your complete control
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          Unlike traditional apps that store your data on company servers, VerusID puts your personal database directly in your wallet. There's literally nowhere for companies to access, analyze, or sell your information.
        </p>
      </div>

      <!-- One identity -->
      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          One identity works across all apps
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          Your VerusID is like having a universal profile that works everywhere in the Verus ecosystem. This means:
        </p>
        
        <div class="ml-4 space-y-1">
          <ul class="ml-4 space-y-1 text-xs text-dark-text-secondary">
            <li class="flex items-start">
              <span class="text-brand-green text-xs mr-2 mt-1">•</span>
              <span>No more creating accounts for every new app</span>
            </li>
            <li class="flex items-start">
              <span class="text-brand-green text-xs mr-2 mt-1">•</span>
              <span>Your data travels with you between different services</span>
            </li>
            <li class="flex items-start">
              <span class="text-brand-green text-xs mr-2 mt-1">•</span>
              <span>Apps can access only what you explicitly share</span>
            </li>
             <li class="flex items-start">
              <span class="text-brand-green text-xs mr-2 mt-1">•</span>
              <span>You own your digital identity forever</span>
            </li>
          </ul>
        </div>
      </div>

      <!-- Control your data -->
      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          You control your data
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          Your VerusID lives on the blockchain under your complete control — no company can suspend, ban, or impersonate you. You decide what information to share with each app, and you can revoke access anytime.
        </p>
      </div>

      <!-- Permanent and recoverable -->
      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          Permanent and recoverable
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          Unlike usernames that expire or accounts that can be deleted, your VerusID is permanent with no renewal fees. Even if you lose your keys, you can set up recovery authorities to restore full access to your identity and data.
        </p>
      </div>

      <!-- Got it button -->
      <div class="flex justify-end mt-6">
        <Button
          variant="primary"
          on:click={handleClose}
        >
          Got It
        </Button>
      </div>

    </div>
  </div>
</div> 