<!-- 
  Component: src/lib/components/verusid-registration/VerusIdRegistrationModal.svelte
  Description: Large modal for VerusID registration flow with sidebar navigation
  Features side-by-side layout with left sidebar for steps and right content area
  Changes:
  - Created as a large modal with left sidebar and right content layout
  - Left sidebar shows vertical step navigation with numbers and labels
  - Responsive design: sidebar on desktop, compact progress on mobile
  - Clean divider between sidebar and content area
  - Step indicators show current/complete states with visual feedback
  - Updated to use reusable Button component for consistent styling
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import Button from '../Button.svelte';
  import IdentityInfoStep from './IdentityInfoStep.svelte';
  import type { NamespaceOption } from '$lib/types';

  // Props
  export let show: boolean = false;
  export let currentStep: number = 1;
  export let totalSteps: number = 5; // Default to 5 steps, can be adjusted

  // Registration state
  let registrationData = {
    name: '',
    namespace: null as NamespaceOption | null,
    isStep1Valid: false,
    referralCode: '',
    preview: ''
  };

  // Event dispatcher
  const dispatch = createEventDispatcher<{ close: void }>();

  // Step labels - can be customized later
  const stepLabels = [
    'Name',
    'Payment Details', 
    'Verification',
    'Payment',
    'Confirmation'
  ];

  function handleClose() {
    dispatch('close');
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  function handleNext() {
    if (currentStep < totalSteps) {
      currentStep += 1;
    }
  }

  function handlePrevious() {
    if (currentStep > 1) {
      currentStep -= 1;
    }
  }

  function handleStep1DataChange(event: CustomEvent<{ name: string; namespace: NamespaceOption | null; isValid: boolean; referralCode: string; preview: string }>) {
    registrationData.name = event.detail.name;
    registrationData.namespace = event.detail.namespace;
    registrationData.isStep1Valid = event.detail.isValid;
    registrationData.referralCode = event.detail.referralCode;
    registrationData.preview = event.detail.preview;
  }

  // Computed
  $: canProceed = currentStep === 1 ? registrationData.isStep1Valid : true; // Add more step validations later

  // Add window keydown listener
  $: if (show) {
    window.addEventListener('keydown', handleKeydown);
    document.body.style.overflow = 'hidden';
  } else {
    window.removeEventListener('keydown', handleKeydown);
    document.body.style.overflow = '';
  }
</script>

{#if show}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-70 backdrop-blur-sm p-4"
    transition:fade={{ duration: 200 }}
    on:click={handleBackdropClick}
  >
    <!-- Large Modal Content -->
    <div 
      class="bg-[#121214] rounded-lg shadow-2xl w-full max-w-6xl h-[90vh] transform transition-all duration-300 border border-dark-border-primary overflow-hidden flex flex-col"
      transition:scale={{ duration: 250, easing: quintOut }}
      on:click|stopPropagation
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="verusid-registration-title"
    >
      <!-- Header with Close Button -->
      <div class="flex items-center justify-between p-4 border-b border-dark-border-primary bg-[#0f0f11]">
        <h2 id="verusid-registration-title" class="text-xl font-semibold text-dark-text-primary">
          Create New VerusID
        </h2>
        <button
          on:click={handleClose}
          class="text-dark-text-secondary hover:text-dark-text-primary transition-colors duration-150 p-1"
          aria-label="Close modal"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Main Layout: Sidebar + Content -->
      <div class="flex flex-1 overflow-hidden">
        <!-- Left Sidebar - Steps Navigation -->
        <div class="w-56 bg-[#121214] border-r border-dark-border-primary flex-shrink-0 hidden md:flex flex-col">          
                     <div class="flex-1 p-6 space-y-4">
             {#each Array.from({ length: totalSteps }, (_, i) => i + 1) as step, index}
               <div class="flex items-center">
                 <!-- Step circle -->
                 <div class="flex-shrink-0">
                   <div 
                     class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium transition-colors duration-200
                       {step === currentStep 
                         ? 'bg-brand-green text-white' 
                         : step < currentStep 
                         ? 'bg-brand-green text-white'
                         : 'bg-gray-600 text-gray-400'
                       }"
                   >
                     {#if step < currentStep}
                       <!-- Checkmark for completed steps -->
                       <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                         <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                       </svg>
                     {:else}
                       {step}
                     {/if}
                   </div>
                 </div>
                 
                 <!-- Step label -->
                 <div class="ml-3">
                   <p class="text-sm font-medium {step === currentStep ? 'text-brand-green' : step < currentStep ? 'text-white' : 'text-gray-400'}">
                     {stepLabels[index] || `Step ${step}`}
                   </p>
                 </div>
               </div>
             {/each}
           </div>
        </div>

        <!-- Mobile Step Indicator (shown on small screens) -->
        <div class="md:hidden bg-[#0f0f11] border-b border-dark-border-primary px-4 py-3">
          <div class="flex items-center justify-between">
            <div class="text-sm text-dark-text-secondary">
              Step {currentStep} of {totalSteps}
            </div>
            <div class="text-sm font-medium text-dark-text-primary">
              {stepLabels[currentStep - 1] || `Step ${currentStep}`}
            </div>
          </div>
          <div class="mt-2 w-full bg-dark-border-primary rounded-full h-1.5">
            <div 
              class="bg-brand-green h-1.5 rounded-full transition-all duration-300 ease-out"
              style="width: {(currentStep / totalSteps) * 100}%"
            ></div>
          </div>
        </div>

        <!-- Right Content Area -->
        <div class="flex-1 flex flex-col overflow-hidden">
          <div class="flex-1 overflow-y-auto p-6">
        <!-- Step Content -->
        {#if currentStep === 1}
          <IdentityInfoStep
            bind:name={registrationData.name}
            bind:selectedNamespace={registrationData.namespace}
            on:dataChange={handleStep1DataChange}
          />
        {:else if currentStep === 2}
          <!-- Placeholder for step 2 -->
          <div class="flex items-center justify-center h-full">
            <div class="text-center">
              <h3 class="text-lg font-medium text-dark-text-primary mb-2">
                Payment Details
              </h3>
              <p class="text-dark-text-secondary">
                Step 2 content will be implemented here.
              </p>
            </div>
          </div>
        {:else if currentStep === 3}
          <!-- Placeholder for step 3 -->
          <div class="flex items-center justify-center h-full">
            <div class="text-center">
              <h3 class="text-lg font-medium text-dark-text-primary mb-2">
                Verification
              </h3>
              <p class="text-dark-text-secondary">
                Step 3 content will be implemented here.
              </p>
            </div>
          </div>
        {:else if currentStep === 4}
          <!-- Placeholder for step 4 -->
          <div class="flex items-center justify-center h-full">
            <div class="text-center">
              <h3 class="text-lg font-medium text-dark-text-primary mb-2">
                Payment
              </h3>
              <p class="text-dark-text-secondary">
                Step 4 content will be implemented here.
              </p>
            </div>
          </div>
        {:else if currentStep === 5}
          <!-- Placeholder for step 5 -->
          <div class="flex items-center justify-center h-full">
            <div class="text-center">
              <h3 class="text-lg font-medium text-dark-text-primary mb-2">
                Confirmation
              </h3>
              <p class="text-dark-text-secondary">
                Step 5 content will be implemented here.
              </p>
            </div>
          </div>
        {/if}
        
        <!-- Content slot for external step components -->
        <slot {currentStep} {totalSteps} />
          </div>
        </div>
      </div>

      <!-- Footer with navigation buttons -->
      <div class="px-6 py-4 border-t border-dark-border-primary bg-[#0f0f11] flex items-center justify-between">
        <!-- Left side: Cancel button -->
        <Button
          variant="secondary"
          on:click={handleClose}
        >
          Cancel
        </Button>
        
        <!-- Center: Empty spacer -->
        <div class="flex-1"></div>
        
        <!-- Right side: Preview + Navigation buttons -->
        <div class="flex items-center space-x-4">
          <!-- Name Preview -->
          {#if registrationData.preview}
            <span class="text-xs text-brand-green/80 font-medium">Creating:</span>
            <span class="bg-black/60 border border-brand-green/30 rounded-lg px-[12px] py-[4px] text-sm text-white font-mono font-medium">{registrationData.preview}</span>
          {/if}
          
          <!-- Navigation buttons -->
          <div class="flex space-x-3">
            {#if currentStep > 1}
              <Button
                variant="secondary"
                on:click={handlePrevious}
              >
                Previous
              </Button>
            {/if}
            
            <Button
              variant="primary"
              disabled={!canProceed}
              on:click={handleNext}
            >
              {currentStep === totalSteps ? 'Complete' : 'Next'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Ensure modal content scrolls properly */
  :global(body:has(.fixed.inset-0)) {
    overflow: hidden;
  }
</style> 