<!-- 
  Component: src/lib/components/chat/FundingModal.svelte
  Description: Modal for funding a private address with options for manual and automatic funding
  Features:
  - Shows user's z-address with copy functionality
  - Automatic funding option for 1 Fast Message (1 x 0.0001 UTXO)
  - Manual funding instructions
  - Pre-validates wallet balance before allowing funding
  Changes:
  - Created new component for private address funding functionality
  - Integrates with Modal.svelte for consistent styling
  - Uses Tauri commands for wallet balance checking and funding
  - Provides clear UX for both technical and non-technical users
  - Updated styling to match NewChatModal design and button patterns
  - Uses Button component with primary/secondary variants
  - Fixed header layout and close button alignment
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Copy, Check, Loader, AlertCircle, Zap, X } from 'lucide-svelte';
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
  let walletBalance: number | null = null;
  let isCheckingBalance = false;
  let balanceError: string | null = null;
  let isFunding = false;
  let fundingError: string | null = null;
  let copySuccess = false;
  let copyTimeout: ReturnType<typeof setTimeout> | null = null;

  // Constants
  const UTXO_AMOUNT = 0.0001;
  const UTXO_COUNT = 1;
  const TOTAL_FUNDING_AMOUNT = UTXO_AMOUNT * UTXO_COUNT; // 0.0001

  // Computed - use fixed fee estimate
  $: estimatedFees = 0.0001; // Fixed fee estimate
  $: totalRequired = TOTAL_FUNDING_AMOUNT + estimatedFees; // 0.0001 + 0.0001 = 0.0002
  $: canAffordFunding = walletBalance !== null && walletBalance >= totalRequired;

  // Lifecycle
  onMount(() => {
    if (show) {
      checkWalletBalance();
    }
  });

  // Watch show prop to check balance when modal opens
  $: if (show) {
    checkWalletBalance();
    resetState();
  }

  // Functions
  function resetState() {
    isFunding = false;
    fundingError = null;
    copySuccess = false;
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
  }

  async function checkWalletBalance() {
    isCheckingBalance = true;
    balanceError = null;
    
    try {
      const walletInfo = await invoke<{ balance: number }>('get_wallet_info');
      walletBalance = walletInfo.balance;
    } catch (error) {
      console.error('Failed to check wallet balance:', error);
      balanceError = 'Failed to check wallet balance';
      walletBalance = null;
    } finally {
      isCheckingBalance = false;
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

  async function handleAutoFunding() {
    if (!canAffordFunding || !privateAddress || isFunding) return;

    isFunding = true;
    fundingError = null;

    console.log('🚀 Starting funding process:', {
      privateAddress,
      currencySymbol,
      amount: UTXO_AMOUNT,
      totalRequired,
      walletBalance
    });

    try {
      const txid = await invoke<string>('fund_private_address_for_messages_cmd', {
        zAddress: privateAddress,
        currency: currencySymbol,
      });

      console.log('✅ Funding successful, txid:', txid);
      
      // Close modal and notify parent of success
      dispatch('close', { success: true });
    } catch (error) {
      console.error('❌ Funding failed:', error);
      fundingError = typeof error === 'string' ? error : 'Funding failed. Please try again.';
      isFunding = false;
    }
  }

  function handleClose() {
    dispatch('close', { success: false });
  }

  function formatCurrency(amount: number): string {
    return `${amount.toFixed(4)} ${currencySymbol}`;
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

    <!-- Automatic Funding Section -->
    <div class="space-y-3">
      <div class="flex items-center space-x-2">
        <Zap size={16} class="text-green-400" />
        <h3 class="text-sm font-medium text-dark-text-primary">Fund for 1 Message</h3>
      </div>
      
      {#if isCheckingBalance}
        <div class="flex items-center justify-center py-4">
          <Loader size={16} class="animate-spin text-blue-400 mr-2" />
          <span class="text-sm text-blue-300">Checking balance...</span>
        </div>
      {:else if balanceError}
        <div class="flex items-center justify-between p-3 bg-red-900/30 border border-red-700/50 rounded text-sm text-red-300">
          <div class="flex items-center space-x-2">
            <AlertCircle size={16} />
            <span>{balanceError}</span>
          </div>
          <Button
            variant="secondary"
            on:click={checkWalletBalance}
          >
            Retry
          </Button>
        </div>
      {:else if walletBalance !== null}
        <div class="space-y-3">
          <div class="text-xs text-dark-text-secondary">
            Cost: {formatCurrency(totalRequired)} • Available: {formatCurrency(walletBalance)}
          </div>
          
          {#if canAffordFunding}
            <div class="w-full">
              <Button 
                variant="primary"
                disabled={isFunding || !privateAddress}
                loading={isFunding}
                loadingText="Funding..."
                iconComponent={Zap}
                on:click={handleAutoFunding}
              >
                Fund for 1 Message
              </Button>
            </div>
          {:else}
            <div class="p-3 bg-yellow-900/30 border border-yellow-700/50 rounded text-sm text-yellow-300">
              <div class="flex items-center space-x-2">
                <AlertCircle size={16} />
                <span>Insufficient balance (need {formatCurrency(totalRequired)})</span>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if fundingError}
        <div class="p-3 bg-red-900/30 border border-red-700/50 rounded text-sm text-red-300">
          <div class="flex items-center space-x-2">
            <AlertCircle size={16} />
            <span>{fundingError}</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Manual Funding Section -->
    <div class="pt-3 border-t border-dark-border-primary">
      <p class="text-xs text-dark-text-secondary">
        Or send ≥{formatCurrency(UTXO_AMOUNT)} to the address above from any wallet.
      </p>
    </div>
  </div>
</Modal>
