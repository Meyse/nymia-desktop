<script lang="ts">
// Component: src/lib/components/OnboardingFlow.svelte
// Description: Orchestrates the multi-step user onboarding process.
// Manages current step, shared state (credentials), renders step components,
// controls navigation buttons, and communicates with the parent page.
// Changes:
// - Added new welcome step as the first step in the flow
// - Updated step navigation to include the welcome step
// - Modified button logic to handle welcome step
// - Added WelcomeStep component import and rendering
// - Added video to the right panel filling the complete panel  
// - Video plays automatically on loop and is muted for autoplay compatibility
// - Removed hardcoded port fallback - ports must come from credentials or be undefined
// - MAJOR: Replaced manual blockchain + credentials steps with automatic BlockchainDetectionStep
// - Simplified onboarding to Welcome → Detection → VerusID (3 steps instead of 4)
// - Credential saving moved to blockchain detection step to ensure credentials are available for VerusID fetching
// - Fixed Continue button logic to only enable when Available blockchain is selected (not just Loading)
// - Added "Follow on X for updates" social link on the left side of the bottom button bar
// - Added PrivacyInfoModal component and handling for privacy info display from WelcomeStep
// - Extracted inline button elements into reusable Button component for better maintainability
// - Replaced Back button and Primary Action Button with Button component variants
// - LATEST: Added ResponsibilityStep between Welcome and Blockchain Detection steps
// - Updated onboarding flow to Welcome → Responsibility → Detection → VerusID (4 steps total)
// - Added responsibility disclosure with "I Understand" button
// - Added currency symbol support - passes blockchain-specific ticker to VerusID step for balance display
// - Fixed dual scrollbar issue: only blockchain step allows outer scrolling, other steps use fixed layout
// - Removed obsolete 'credentials' prop from VerusIdStep, as credentials are now handled by the backend.

  import { createEventDispatcher } from 'svelte';
  import { slide } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import { ExternalLink } from 'lucide-svelte';

  // Import Step Components
  import WelcomeStep from './onboarding/WelcomeStep.svelte';
  import ResponsibilityStep from './onboarding/ResponsibilityStep.svelte';
  import BlockchainDetectionStep from './onboarding/BlockchainDetectionStep.svelte';
  import VerusIdStep from './onboarding/VerusIdStep.svelte';
  import PrivacyInfoModal from './onboarding/PrivacyInfoModal.svelte';
  
  // Import Shared Components
  import Button from './Button.svelte';

  // Import Shared Types
  import type { 
      Credentials, 
      FormattedIdentity, 
      LoginPayload, 
      OnboardingStep 
  } from '$lib/types';
  
  // Import Currency Symbol Utility
  import { getCurrencySymbol } from '$lib/utils/currencySymbol';

  // --- Types ---
  type Step = 'welcome' | 'responsibility' | 'blockchain' | 'verusid';

  // --- Props ---
  export let initialStep: OnboardingStep = 'welcome';
  export let initialCredentials: Credentials | null = null;

  // --- State ---
  let currentStep: OnboardingStep = initialStep;
  
  // Shared state between steps
  let selectedIdentity: FormattedIdentity | null = null; // Store the full selected identity
  let currentCredentials: Credentials | null = initialCredentials; // Track latest credentials from detection
  let connectionBlockHeight: number | null = null; // Updated by BlockchainDetectionStep
  let selectedBlockchainId: string | null = null; // Track selected blockchain
  let detectionCompleted = false; // Track if detection step is completed
  let availableBlockchainsCount = 0; // Track how many blockchains are available
  let blockchainSelected = false; // Track if an Available blockchain has been selected
  let showPrivacyModal = false; // Control privacy info modal visibility

  // Responsibility message hover state
  let responsibilityHovered = false;
  let responsibilityHoverTimeout: ReturnType<typeof setTimeout> | null = null;

  // --- Event Dispatcher (to parent: +page.svelte) ---
  const dispatch = createEventDispatcher<{
    'login-success': LoginPayload;
    'authentication-cleared': void;
  }>();

  // --- Lifecycle & State Management ---

  // Update internal state if initialCredentials prop changes (e.g., on logout/restart)
  $: if (initialCredentials && initialCredentials !== currentCredentials) {
      console.log("OnboardingFlow: initialCredentials prop changed, updating internal state. Port:", initialCredentials.rpc_port);
      currentCredentials = initialCredentials;
      // Also reset identity selection if creds change during onboarding
      selectedIdentity = null; 
  }
  
  // Reset relevant state when moving away from detection step
  $: if (currentStep !== 'blockchain') {
      // Keep the detection state for navigation logic but reset selection
      blockchainSelected = false;
  }

  // Calculate currency symbol based on selected blockchain
  $: currencySymbol = getCurrencySymbol(selectedBlockchainId);

  // --- Step Navigation ---
  function goToStep(step: OnboardingStep) {
    currentStep = step;
    // Reset relevant state when changing steps
    if (step !== 'verusid') {
      selectedIdentity = null;
    }
  }

  function nextStep() {
    if (currentStep === 'welcome') {
      goToStep('responsibility');
    } else if (currentStep === 'responsibility') {
      goToStep('blockchain');
    } else if (currentStep === 'blockchain' && detectionCompleted && currentCredentials) {
      goToStep('verusid');
    }
  }

  function prevStep() {
    if (currentStep === 'responsibility') {
      goToStep('welcome');
    } else if (currentStep === 'blockchain') {
      goToStep('responsibility');
    } else if (currentStep === 'verusid') {
      // Reset VerusID selection when going back
      selectedIdentity = null;
      blockchainSelected = false; // Reset blockchain selection too
      selectedBlockchainId = null; // Reset blockchain visual selection
      goToStep('blockchain');
    }
  }

  // --- Event Handlers from Step Components ---
  function handleGetStarted() {
    goToStep('responsibility');
  }

  function handleShowPrivacyInfo() {
    showPrivacyModal = true;
  }

  function handleClosePrivacyModal() {
    showPrivacyModal = false;
  }

  function handleUnderstood() {
    goToStep('blockchain');
  }

  // Responsibility message hover handlers
  function handleResponsibilityMouseEnter() {
    // Clear any existing timeout
    if (responsibilityHoverTimeout) {
      clearTimeout(responsibilityHoverTimeout);
      responsibilityHoverTimeout = null;
    }
    responsibilityHovered = true;
  }

  function handleResponsibilityMouseLeave() {
    // Set timeout to remove hover effect after 2 seconds
    responsibilityHoverTimeout = setTimeout(() => {
      responsibilityHovered = false;
      responsibilityHoverTimeout = null;
    }, 2000);
  }

  function handleBlockchainSelected(event: CustomEvent<{ 
    blockchainId: string; 
    credentials: Credentials; 
    blockHeight: number;
  }>) {
    console.log("OnboardingFlow: Blockchain selected event received");
    const { blockchainId, credentials, blockHeight } = event.detail;
    
    // Update shared state
    selectedBlockchainId = blockchainId;
    currentCredentials = credentials;
    connectionBlockHeight = blockHeight;
    detectionCompleted = true;
    blockchainSelected = true; // Mark that an Available blockchain has been selected
    
    console.log(`OnboardingFlow: Selected ${blockchainId} with block height ${blockHeight}`);
  }

  function handleDetectionCompleted(event: CustomEvent<{ availableCount: number }>) {
    console.log("OnboardingFlow: Detection completed event received");
    availableBlockchainsCount = event.detail.availableCount;
    detectionCompleted = true;
  }

  function handleIdSelected(event: CustomEvent<{ identity: FormattedIdentity | null }>) {
      console.log("OnboardingFlow: Received idSelected event with identity:", event.detail.identity);
      selectedIdentity = event.detail.identity; 
      // Parent component (this one) controls enabling the login button via `isPrimaryButtonDisabled`
  }

  async function handleLogin() {
      if (!selectedIdentity || !currentCredentials) {
          console.error("OnboardingFlow: Cannot login - missing selected ID or credentials.");
          return;
      }
      
      // Credentials are now saved earlier during blockchain selection, so we can proceed directly to login
      console.log(`OnboardingFlow: Login initiated for ${selectedIdentity.formatted_name} (${selectedIdentity.i_address})`);
      dispatch('login-success', {
         identity: selectedIdentity,
         blockHeight: connectionBlockHeight,
         blockchainId: selectedBlockchainId
      });
  }

  // --- Dynamic Button Logic ---
  $: primaryButtonLabel = 
      currentStep === 'welcome' ? 'Get Started' :
      currentStep === 'responsibility' ? 'I Understand' :
      currentStep === 'blockchain' ? 'Continue' : 'Login';

  $: isPrimaryButtonDisabled = 
      currentStep === 'welcome' ? false :
      currentStep === 'responsibility' ? false :
      (currentStep === 'blockchain' && (!detectionCompleted || !blockchainSelected)) ||
      (currentStep === 'verusid' && !selectedIdentity); // Check the full object

  $: primaryButtonAction = 
      currentStep === 'welcome' ? handleGetStarted :
      currentStep === 'responsibility' ? handleUnderstood :
      currentStep === 'blockchain' ? nextStep : handleLogin;

</script>

<!-- Main container: Two-panel layout -->
<div class="flex h-screen font-sans">

  <!-- Left Panel: Onboarding Steps -->
  <div class="w-1/2 flex flex-col bg-dark-bg-primary">
  
      <!-- Top padding -->
      <div class="pt-12 px-10"></div>

      <!-- Main Content Area (conditional scrolling: only blockchain step can scroll) -->
      <div class="flex-grow px-10 pt-8 {currentStep === 'blockchain' ? 'overflow-y-auto' : 'overflow-y-hidden'}">
          <div class="step-container mx-auto h-full flex flex-col" style="max-width: 450px;">
              {#if currentStep === 'welcome'}
                 <div transition:slide|local={{ duration: 300, easing: quintOut }}>
                     <WelcomeStep 
                        on:getStarted={handleGetStarted}
                        on:showPrivacyInfo={handleShowPrivacyInfo}
                     />
                 </div>
              {:else if currentStep === 'responsibility'}
                 <div transition:slide|local={{ duration: 300, easing: quintOut }}>
                     <ResponsibilityStep 
                        on:understood={handleUnderstood}
                     />
                 </div>
              {:else if currentStep === 'blockchain'}
                 <div transition:slide|local={{ duration: 300, easing: quintOut }}>
                     <BlockchainDetectionStep 
                        bind:selectedBlockchainId={selectedBlockchainId}
                        on:blockchainSelected={handleBlockchainSelected}
                        on:detectionCompleted={handleDetectionCompleted}
                     />
                 </div>
              {:else if currentStep === 'verusid'}
                 <div transition:slide|local={{ duration: 300, easing: quintOut }}>
                     <VerusIdStep 
                        currencySymbol={currencySymbol}
                        on:idSelected={handleIdSelected}
                     />
                 </div>
              {/if}
          </div>
      </div>

      <!-- Bottom Button Bar -->
      <div class="pr-10 pl-4 py-4 border-t border-dark-border-primary bg-dark-bg-primary mt-auto">
          
          <!-- Responsibility Message (only shown on responsibility step) -->
          {#if currentStep === 'responsibility'}
            <div class="mb-8 pl-8 pr-2 pt-2">
              <p 
                class="text-lg font-semibold tracking-tight select-none cursor-default text-left {responsibilityHovered ? 'text-white' : 'text-white/70'}"
                on:mouseenter={handleResponsibilityMouseEnter}
                on:mouseleave={handleResponsibilityMouseLeave}
              >
                With real privacy comes real responsibility.
              </p>
            </div>
          {/if}

          <div class="flex justify-between items-center cursor-default select-none">
              <!-- Left Side: Social Links -->
              <div class="flex items-center space-x-1">
                  <!-- X Icon -->
                  <a 
                      href="https://x.com/NymiaApp" 
                      target="_blank" 
                      rel="noopener noreferrer"
                      class="flex items-center py-2 px-2 text-xs text-white/45 hover:text-white/70 group"
                  >
                      <svg class="w-5 h-5 group-hover:text-dark-text-primary" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
                      </svg>
                  </a>

                  <!-- GitHub Icon -->
                  <a 
                      href="https://github.com/Meyse/nymia-desktop" 
                      target="_blank" 
                      rel="noopener noreferrer"
                      class="flex items-center py-2 px-2 text-xs text-white/45 hover:text-white/70 group"
                  >
                      <svg class="w-5 h-5 group-hover:text-dark-text-primary" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                      </svg>
                  </a>
              </div>

              <!-- Right Side: Navigation Buttons -->
              <div class="flex space-x-3">
                   <!-- Back Button (Conditional) -->
                  {#if currentStep !== 'welcome'}
                    <Button variant="secondary" on:click={prevStep}>
                      Back
                    </Button>
                  {/if}

                 <!-- Primary Action Button -->
                  <Button 
                    variant="primary" 
                    disabled={isPrimaryButtonDisabled} 
                    on:click={primaryButtonAction}
                  >
                    {primaryButtonLabel}
                  </Button>
              </div>
          </div>
      </div>
  </div>

   <!-- Right Panel: Decorative Background -->
   <div class="w-1/2 relative overflow-hidden bg-black">
       <!-- Background elements... -->
       
     
        <!-- Onboarding video filling the panel -->
        <div class="absolute inset-0">
            <video 
                src="/onboarding-2-apple.mp4" 
                autoplay 
                muted 
                loop 
                playsinline
                class="onboarding-video"
            >
                Your browser does not support the video tag.
            </video>
        </div>
         
        
   </div>
</div>

<!-- Privacy Info Modal -->
{#if showPrivacyModal}
  <PrivacyInfoModal on:close={handleClosePrivacyModal} />
{/if}

<style>
 
  

  /* Onboarding video styling to fill the complete panel */
  .onboarding-video {
    width: 100%;
    height: 100%;
    object-fit: cover; /* Fill container, crop if needed to maintain aspect ratio */
    object-position: center center; /* Center the video */
  }



  @keyframes gradient-shimmer {
    0%, 100% {
      background-position: 0% 50%;
    }
    50% {
      background-position: 100% 50%;
    }
  }

  /* Fallback for browsers that don't support background-clip: text */
  @supports not (background-clip: text) {
    .responsibility-text {
      color: #ffffff;
      background: none;
      -webkit-text-fill-color: initial;
    }
  }

  /* Other styles */
  .step-container {
      width: 100%;
      /* max-width and margin removed, handled by parent div */
  }

</style>