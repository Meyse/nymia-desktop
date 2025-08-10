<!-- 
  Component: src/lib/components/chat/FundingModal.svelte
  Description: Modal for displaying a private address that users need to fund to send messages
  Features:
  - Shows user's z-address with copy functionality
  - Provides clear instructions for manual funding from Verus wallets
  - Informs users they can find the address later in Settings
  Changes:
  - Simplified to remove automatic funding functionality
  - Updated styling to match VerusIdInfoModal design patterns
  - Revised messaging to focus on Verus Desktop and CLI funding sources
  - Added reference to Settings for finding the address later
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Copy, Check, X } from 'lucide-svelte';
  import Modal from '../Modal.svelte';
  import Button from '../Button.svelte';

  // Props
  export let show: boolean = false;
  export let privateAddress: string = '';
  export let currencySymbol: string = 'VRSC';

  // Events
  const dispatch = createEventDispatcher<{
    close: { success: boolean };
  }>();

  // State
  let copySuccess = false;
  let copyTimeout: ReturnType<typeof setTimeout> | null = null;

  // Constants  
  const MIN_FUNDING_AMOUNT = 0.0001;

  // Watch show prop to reset state when modal opens
  $: if (show) {
    resetState();
  }

  // Functions
  function resetState() {
    copySuccess = false;
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
  }

  async function handleCopyAddress() {
    try {
      await navigator.clipboard.writeText(privateAddress);
      copySuccess = true;
      
      // Reset copy success after 2 seconds
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  }



  function handleClose() {
    dispatch('close', { success: false });
  }
</script>

<Modal bind:show size="md" on:close={handleClose}>
  <svelte:fragment slot="header" let:modalHeaderId let:handleClose>
    <h2 class="text-base font-medium text-dark-text-primary flex-grow cursor-default select-none" id={modalHeaderId}>
      Fund Private Address
    </h2>
    <button 
      on:click={handleClose}
      class="text-dark-text-secondary hover:text-dark-text-primary p-1 rounded-full hover:bg-dark-bg-tertiary transition-colors focus:outline-none focus:ring-1 focus:ring-dark-border-secondary"
      aria-label="Close modal"
    >
      <X size={16} strokeWidth={2.5} />
    </button>
  </svelte:fragment>

  <div class="p-6 space-y-4">
    <!-- Address Display Section -->
    <div class="space-y-2">
      <h3 class="text-sm font-medium text-dark-text-primary">Your Private Address</h3>
      <div class="flex items-center space-x-2 p-3 bg-dark-bg-primary border border-dark-border-primary rounded-md">
        <div class="flex-1 min-w-0">
          <p class="text-xs font-mono text-dark-text-secondary break-all">
            {privateAddress}
          </p>
        </div>
        <Button
          variant="secondary"
          disabled={!privateAddress}
          iconComponent={copySuccess ? Check : Copy}
          on:click={handleCopyAddress}
        >
          {copySuccess ? 'Copied!' : 'Copy'}
        </Button>
      </div>
    </div>

    <!-- Funding Instructions Section -->
    <div class="space-y-4">
      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          How to Fund Your Address
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          To start sending messages, you need to fund this private address with at least <span class="font-mono text-dark-text-primary">{MIN_FUNDING_AMOUNT.toFixed(4)} {currencySymbol}</span>.
        </p>
      </div>

      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          Sending Funds
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          Send {currencySymbol} to the address above from your Verus Desktop wallet or using the Verus CLI. Once the transaction is confirmed, you'll be able to send private messages.
        </p>
      </div>

      <div class="space-y-2">
        <h3 class="font-semibold text-sm text-dark-text-primary">
          Find This Address Later
        </h3>
        <p class="text-xs text-dark-text-secondary leading-relaxed">
          You can always find this address in your Settings if you need to fund it again or share it with others.
        </p>
      </div>
    </div>
  </div>
</Modal>
