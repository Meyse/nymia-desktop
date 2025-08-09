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
  - Removed close icon from header and disabled backdrop/escape key closing
  - Modal can only be closed via the Cancel button for better user flow control
  - Updated VerusID name preview to look less button-like with subtle styling
  - Added currency conversion sub-step handling within payment flow
  - Shows conversion step when payment option requires conversion (isDirect: false)
  - Fixed Cancel button logic: enabled in steps 1-2 (including after conversion), disabled during Register ID and Keys & Backup
  - Fixed Keys & Backup step: no auto-advance, user must check boxes then click Finish to close modal
  - Removed unnecessary Confirmation step (now 4 steps total)
  - Pass newly created identity name to parent for auto-selection
  - Reset modal state to step 1 when reopened for fresh registration flow
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import Button from '../Button.svelte';
  import IdentityInfoStep from './IdentityInfoStep.svelte';
  import PaymentDetailsStep from './PaymentDetailsStep.svelte';
  import CurrencyConversionStep from './CurrencyConversionStep.svelte';
  import RegisterIdentityStep from './RegisterIdentityStep.svelte';
  import KeyBackupStep from './KeyBackupStep.svelte';
  import type { NamespaceOption } from '$lib/types';

  // Props
  export let show: boolean = false;
  export let currentStep: number = 1;
  export let totalSteps: number = 4; // Updated to 4 steps (removed Confirmation)

  // Registration state
  let registrationData = {
    name: '',
    namespace: null as NamespaceOption | null,
    isStep1Valid: false,
    referralCode: '',
    preview: '',
    isNameAvailable: false,
    isReferralValid: false,
    usdEstimate: null as number | null, // To hold the USD price from step 1
    // Step 2 data
    selectedPaymentOption: null as any,
    isStep2Valid: false,
    // Currency conversion state
    showConversionStep: false,
    conversionCompleted: false,
    conversionTxid: '',
    actualAmountReceived: 0,
    conversionCanProceed: false,
    conversionStartFunction: null as (() => Promise<void>) | null,
    isConverting: false, // Track if conversion is in progress
    controlAddress: '' as string,
    privateAddress: '' as string,
  };

  // Reset modal state when opening
  function resetModalState() {
    currentStep = 1;
    keysBackupReady = false;
    registrationData = {
      name: '',
      namespace: null as NamespaceOption | null,
      isStep1Valid: false,
      referralCode: '',
      preview: '',
      isNameAvailable: false,
      isReferralValid: false,
      usdEstimate: null as number | null,
      selectedPaymentOption: null as any,
      isStep2Valid: false,
      showConversionStep: false,
      conversionCompleted: false,
      conversionTxid: '',
      actualAmountReceived: 0,
      conversionCanProceed: false,
      conversionStartFunction: null as (() => Promise<void>) | null,
      isConverting: false,
      controlAddress: '' as string,
      privateAddress: '' as string,
    };
  }

  // Watch for modal opening and reset state
  $: if (show) {
    resetModalState();
  }

  // Event dispatcher
  const dispatch = createEventDispatcher<{ close: { refresh?: boolean; newIdentityName?: string | null } }>();

  // Step labels - can be customized later
  const stepLabels = [
    'Name',
    'Payment Details', 
    'Register ID',
    'Keys & Backup'
  ];

  function handleClose() {
    // Emit refresh signal when closing after completion, and pass the newly created identity info
    const isCompleted = currentStep === 4 && keysBackupReady;
    const newIdentityName = isCompleted && registrationData.namespace ? 
      (isRootNamespace(registrationData.namespace) ? `${registrationData.name}@` : `${registrationData.name}.${registrationData.namespace.name}@`) : 
      null;
    
    dispatch('close', { 
      refresh: isCompleted,
      newIdentityName: newIdentityName
    });
  }

  function isRootNamespace(namespace: NamespaceOption): boolean {
    const lname = namespace.name.toLowerCase();
    return lname === 'vrsc' || lname === 'vrsctest';
  }

  // Removed backdrop click handler - modal can only be closed via Cancel button

  // Removed escape key handler - modal can only be closed via Cancel button

  function handleNext() {
    // Handle special case for step 2 when conversion is required
    if (currentStep === 2 && registrationData.selectedPaymentOption && !registrationData.selectedPaymentOption.isDirect && !registrationData.conversionCompleted) {
      // Show conversion step instead of going to next step
      registrationData.showConversionStep = true;
      return;
    }

    if (currentStep < totalSteps) {
      currentStep += 1;
      // Reset conversion step state when moving to next step
      registrationData.showConversionStep = false;
    }
  }

  function handlePrevious() {
    // Handle special case when in conversion step
    if (registrationData.showConversionStep) {
      registrationData.showConversionStep = false;
      return;
    }

    if (currentStep > 1) {
      currentStep -= 1;
    }
  }

  function handleStep1DataChange(event: CustomEvent<{ name: string; namespace: NamespaceOption | null; isValid: boolean; referralCode: string; preview: string; isNameAvailable: boolean; isReferralValid: boolean; usdEstimate: number | null; }>) {
    registrationData.name = event.detail.name;
    registrationData.namespace = event.detail.namespace;
    registrationData.isStep1Valid = event.detail.isValid;
    registrationData.referralCode = event.detail.referralCode;
    registrationData.preview = event.detail.preview;
    registrationData.isNameAvailable = event.detail.isNameAvailable;
    registrationData.isReferralValid = event.detail.isReferralValid;
    registrationData.usdEstimate = event.detail.usdEstimate;
  }

  function handleStep2DataChange(event: CustomEvent<{ selectedPaymentOption: any; isValid: boolean; }>) {
    registrationData.selectedPaymentOption = event.detail.selectedPaymentOption;
    registrationData.isStep2Valid = event.detail.isValid;
    
    // Reset conversion state when payment option changes
    registrationData.conversionCompleted = false;
    registrationData.showConversionStep = false;
  }

  function handleConversionCompleted(event: CustomEvent<{ txid: string; actualAmountReceived: number; }>) {
    registrationData.conversionCompleted = true;
    registrationData.conversionTxid = event.detail.txid;
    registrationData.actualAmountReceived = event.detail.actualAmountReceived;
    
    console.log('Conversion completed:', event.detail);
    
    // Don't automatically proceed - wait for user to click "Next"
    // handleNext();
  }

  function handleConversionError(event: CustomEvent<{ message: string; }>) {
    console.error('Conversion error:', event.detail.message);
    // Stay in conversion step to allow retry
  }

  function handleConversionStateChange(event: CustomEvent<{ canProceed: boolean; startFunction: (() => Promise<void>) | null; }>) {
    registrationData.conversionCanProceed = event.detail.canProceed;
    registrationData.conversionStartFunction = event.detail.startFunction;
  }

  function handleConversionStatusChange(event: CustomEvent<{ isConverting: boolean; }>) {
    registrationData.isConverting = event.detail.isConverting;
  }

  // Track when Keys & Backup step is ready (both checkboxes checked)
  let keysBackupReady = false;

  // Computed
  $: canProceed = currentStep === 1 ? registrationData.isStep1Valid : 
                  currentStep === 2 ? (isInConversionFlow ? (registrationData.conversionCompleted || registrationData.conversionCanProceed) : registrationData.isStep2Valid) :
                  currentStep === 3 ? false : // auto-advances on completion
                  currentStep === 4 ? keysBackupReady : // enabled when both checkboxes are checked
                  true; // step 5

  $: canGoBack = (currentStep > 1 || registrationData.showConversionStep) && !registrationData.isConverting && currentStep !== 3 && currentStep !== 4;

  // Cancel button should be enabled in steps 1, 2 (including after conversion), but disabled during Register ID and Keys & Backup
  $: canCancel = !registrationData.isConverting && 
                 (currentStep === 1 || 
                  currentStep === 2 || 
                  (isInConversionFlow && registrationData.conversionCompleted));

  $: isInConversionFlow = registrationData.showConversionStep;

  $: nextButtonText = currentStep === 2 && registrationData.selectedPaymentOption && !registrationData.selectedPaymentOption.isDirect && !registrationData.conversionCompleted 
    ? (registrationData.conversionCompleted ? 'Next' : 'Start Conversion') 
    : currentStep === 3 
    ? 'Next' 
    : currentStep === 4 
    ? 'Finish' 
    : currentStep === totalSteps 
    ? 'Complete' 
    : 'Next';

  // Handle the Finish button click in step 4 to close the modal
  function handleFinish() {
    if (currentStep === 4) {
      handleClose();
    } else {
      handleNext();
    }
  }

  // Compute final price and other data for step 2
  $: finalPrice = registrationData.namespace && registrationData.isReferralValid && registrationData.referralCode ? 
    calculateDiscountedPrice(registrationData.namespace.registration_fee, registrationData.namespace.id_referral_levels) :
    registrationData.namespace?.registration_fee || 0;

  $: isReferralApplied = registrationData.isReferralValid && registrationData.referralCode.trim() !== '';

  $: referralDiscount = registrationData.namespace ? calculateReferralDiscount(registrationData.namespace.id_referral_levels) : 0;
  
  // Debug logging for conversion step data
  $: {
    if (isInConversionFlow && typeof window !== 'undefined') {
      console.log('=== MODAL PASSING TO CONVERSION ===');
      console.log('registrationData.selectedPaymentOption:', registrationData.selectedPaymentOption);
      console.log('registrationData.namespace:', registrationData.namespace);
      console.log('finalPrice:', finalPrice);
      console.log('isInConversionFlow:', isInConversionFlow);
      console.log('=====================================');
    }
  }

  function calculateReferralDiscount(referralLevels: number): number {
    // referralLevels 0-5 maps to 1/2, 1/3, 1/4, 1/5, 1/6, 1/7
    return 1 / (referralLevels + 2);
  }

  function calculateDiscountedPrice(originalPrice: number, referralLevels: number): number {
    const discount = calculateReferralDiscount(referralLevels);
    return originalPrice * (1 - discount);
  }

  // Manage body scroll when modal is shown
  $: if (show) {
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
</script>

{#if show}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-70 backdrop-blur-sm p-4"
    transition:fade={{ duration: 200 }}
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
      <!-- Header without Close Button -->
      <div class="flex items-center p-4 border-b border-dark-border-primary bg-[#0f0f11]">
        <h2 id="verusid-registration-title" class="text-xl font-semibold text-dark-text-primary">
          Create New VerusID
        </h2>
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
                     {#if step < currentStep || (step === currentStep && isInConversionFlow)}
                       <!-- Checkmark for completed steps or current step when in conversion flow -->
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
                   <p class="text-sm font-medium {step === currentStep && !isInConversionFlow ? 'text-brand-green' : step < currentStep || (step === currentStep && isInConversionFlow) ? 'text-white' : 'text-gray-400'}">
                     {stepLabels[index] || `Step ${step}`}
                     {#if step === 2 && isInConversionFlow}
                       <span class="text-xs text-brand-green block">Converting...</span>
                     {/if}
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
              {#if isInConversionFlow}<span class="text-brand-green">(Converting)</span>{/if}
            </div>
            <div class="text-sm font-medium text-dark-text-primary">
              {stepLabels[currentStep - 1] || `Step ${currentStep}`}
              {#if isInConversionFlow}<span class="text-brand-green block text-xs">Currency Conversion</span>{/if}
            </div>
          </div>
          <div class="mt-2 w-full bg-dark-border-primary rounded-full h-1.5">
            <div 
              class="bg-brand-green h-1.5 rounded-full transition-all duration-300 ease-out"
              style="width: {((currentStep + (isInConversionFlow ? 0.5 : 0)) / totalSteps) * 100}%"
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
        {:else if currentStep === 2 && registrationData.namespace}
          {#if isInConversionFlow}
            <!-- Currency Conversion Sub-step -->
            <CurrencyConversionStep
              selectedPaymentOption={registrationData.selectedPaymentOption}
              selectedNamespace={registrationData.namespace}
              finalPrice={finalPrice}
              on:completed={handleConversionCompleted}
              on:error={handleConversionError}
              bind:canProceed={registrationData.conversionCanProceed}
              bind:startFunction={registrationData.conversionStartFunction}
              on:statusChange={handleConversionStatusChange}
            />
          {:else}
            <!-- Regular Payment Details Step -->
            <PaymentDetailsStep
              selectedNamespace={registrationData.namespace}
              finalPrice={finalPrice}
              isReferralApplied={isReferralApplied}
              referralDiscount={referralDiscount}
              previewId={registrationData.preview}
              usdEstimate={registrationData.usdEstimate}
              on:dataChange={handleStep2DataChange}
            />
          {/if}
        {:else if currentStep === 3 && registrationData.namespace}
          <RegisterIdentityStep
            name={registrationData.name}
            selectedNamespace={registrationData.namespace}
            referralCode={registrationData.referralCode}
            on:completed={(e) => {
              // pass addresses to next step
              registrationData.controlAddress = e.detail.controlAddress;
              registrationData.privateAddress = e.detail.privateAddress;
              handleNext();
            }}
            on:error={(e) => {
              console.error('Registration error', e.detail.message);
            }}
          />
        {:else if currentStep === 4}
          <KeyBackupStep
            controlAddress={registrationData.controlAddress}
            privateAddress={registrationData.privateAddress}
            on:ready={() => {
              keysBackupReady = true;
            }}
          />
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
          disabled={!canCancel}
        >
          Cancel
        </Button>
        
        <!-- Center: Empty spacer -->
        <div class="flex-1"></div>
        
        <!-- Right side: Preview + Navigation buttons -->
        <div class="flex items-center space-x-4">
          <!-- Name Preview -->
          {#if registrationData.preview}
            <div class="flex items-center space-x-2 text-xs text-white/70">
              <span class="font-medium">Creating:</span>
              <span class="bg-white/5 border border-white/10 rounded px-2 py-1 text-white font-mono text-sm tracking-wide">
                {registrationData.preview}
              </span>
            </div>
          {/if}
          
          <!-- Navigation buttons -->
          <div class="flex space-x-3">
            {#if canGoBack}
              <Button
                variant="secondary"
                on:click={handlePrevious}
              >
                Previous
              </Button>
            {/if}
            
            {#if isInConversionFlow}
              <Button
                variant="primary"
                disabled={!canProceed}
                on:click={async () => {
                  if (registrationData.conversionCompleted) {
                    // Conversion is complete, proceed to next step
                    handleNext();
                  } else if (registrationData.conversionStartFunction) {
                    // Start the conversion
                    await registrationData.conversionStartFunction();
                  }
                }}
              >
                {registrationData.conversionCompleted ? 'Next' : 'Start Conversion'}
              </Button>
            {:else}
              <Button
                variant="primary"
                disabled={!canProceed}
                on:click={handleFinish}
              >
                {nextButtonText}
              </Button>
            {/if}
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