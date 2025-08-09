<!-- 
  Component: src/lib/components/verusid-registration/CurrencyConversionStep.svelte
  Description: Currency conversion sub-step for VerusID registration payment flow
  Features address selection, conversion preview, and conversion execution with progress tracking
  Changes:
  - Created currency conversion step with source/destination address selection
  - Integrated with new backend APIs for wallet addresses and currency balances
  - Shows conversion preview with 10% buffer and real-time estimates
  - Implements conversion execution with time-based progress tracking (no timeout)
  - No cancel option during conversion - user must wait for completion
  - Only shows addresses with sufficient balance for conversion
  - UPDATED: Added mono font for address displays using useMonoFont prop in CustomDropdown
  - UPDATED: Redesigned conversion preview with better visual hierarchy and compact layout
  - UPDATED: Moved buffer explanation to tooltip using HelpCircle icon (same style as UserInfoSection)
  - UPDATED: Fixed conversion math - now calls estimateconversion with total amount (base + buffer)
  - UPDATED: Added back conversion fee display (0.025%) for transparency (fee included in estimateconversion)
  - UPDATED: Enhanced debugging for estimateconversion API calls
  - UPDATED: Fixed floating-point precision by rounding all amounts to 8 decimal places (satoshis)
  - UPDATED: Improved breakdown showing: You send → Fee → Buffer → Total → You receive
  - UPDATED: Enhanced dropdown UI - removed checkmarks, smaller text, 5-decimal balances, normal font for "Use all addresses"
  - UPDATED: Replaced misleading block-based progress with time-based progress tracking like RegisterIdentityStep
  - UPDATED: Removed transaction ID display and added educational context about conversion timing
  - UPDATED: Added adaptive messaging based on elapsed time with accurate LP processing descriptions
  - UPDATED: Corrected messaging to reflect Protocol DeFi LP processing (1-10 blocks) rather than "next block"
-->
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { HelpCircle } from 'lucide-svelte';
  import Button from '../Button.svelte';
  import CustomDropdown from '../CustomDropdown.svelte';
  import type { NamespaceOption, DropdownOption } from '$lib/types';

  // Props from PaymentDetailsStep
  export let selectedPaymentOption: any;
  export let selectedNamespace: NamespaceOption;
  export let finalPrice: number; // NOTE: Not used directly, calculations based on selectedPaymentOption
  export let canProceed = false;
  export let startFunction: (() => Promise<void>) | null = null;

  // Interface for address with balances
  interface AddressWithBalance {
    address: string;
    balances: Record<string, number>;
    hasEnoughBalance: boolean;
    availableAmount: number;
  }

  // State
  let addresses: AddressWithBalance[] = [];
  let sourceAddressOptions: DropdownOption[] = [];
  let destinationAddressOptions: DropdownOption[] = [];
  let selectedSourceAddress: string = '*'; // Default to wildcard
  let selectedDestinationAddress: string = '';
  let loading = true;
  let error: string | null = null;
  let conversionRate: number | null = null;
  let fetchingRate = false;
  let totalSourceCurrencyBalance: number = 0;

  // Conversion calculation state
  let sourceAmountToSend: number = 0;
  let amountWithBuffer: number = 0;
  let targetAmountNeeded: number = 0; // For display only

  // Conversion execution state
  let converting = false;
  let conversionStarted = false;
  let conversionTxid: string | null = null;
  let conversionError: string | null = null;
  let pollingInterval: number | null = null;
  let baselineBalance: number = 0; // Baseline balance of target currency in destination address before conversion
  let conversionCompleted = false;
  let actualAmountReceived: number = 0;

  // Timing tracking for dynamic messages
  let conversionStartTime: number = 0;
  let elapsedSeconds: number = 0;
  let timerInterval: number | null = null;

  // Tooltip state
  let showBufferTooltip = false;

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    completed: { txid: string; actualAmountReceived: number };
    error: { message: string };
    statusChange: { isConverting: boolean };
  }>();

  // Computed values
  $: sourceCurrency = selectedPaymentOption?.currencyName || '';
  $: targetCurrency = selectedNamespace?.name || '';
  $: conversionFee = roundToSatoshis(sourceAmountToSend * 0.00025); // 0.025% conversion fee (shown for transparency)
  $: bufferAmount = roundToSatoshis(sourceAmountToSend * 0.1); // 10% buffer
  $: totalAmountToConvert = amountWithBuffer; // Total amount including buffer (already rounded)
  $: internalCanProceed = !converting && !conversionStarted && !!selectedSourceAddress && !!selectedDestinationAddress && !!conversionRate && hasValidSourceAddress && amountWithBuffer > 0;
  $: hasValidSourceAddress = selectedSourceAddress === '*' 
    ? totalSourceCurrencyBalance >= amountWithBuffer
    : addresses.some(addr => addr.address === selectedSourceAddress && addr.hasEnoughBalance);
  $: selectedSourceBalance = selectedSourceAddress === '*' 
    ? `${formatCurrency(totalSourceCurrencyBalance, 5)} ${sourceCurrency}` 
    : sourceAddressOptions.find(opt => opt.id === selectedSourceAddress)?.balance || null;

  // Progress messaging based on elapsed time
  $: progressMessage = getProgressMessage(elapsedSeconds);
  $: progressDescription = getProgressDescription(elapsedSeconds);

  // Lifecycle
  onMount(() => {
    initialize();
  });

  onDestroy(() => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
    if (timerInterval) {
      clearInterval(timerInterval);
    }
  });

  // Reactive statements for state communication
  $: {
    if (typeof window !== 'undefined') {
      canProceed = internalCanProceed;
      startFunction = internalCanProceed ? startConversion : null;
    }
  }

  // Dispatch status changes
  $: {
    if (typeof window !== 'undefined') {
      dispatch('statusChange', { isConverting: converting || conversionStarted });
    }
  }

  // Functions
  async function initialize() {
    loading = true;
    error = null;
    
    // 1. Set amounts based on previous step's calculation
    sourceAmountToSend = roundToSatoshis(selectedPaymentOption?.requiredAmount || 0);
    amountWithBuffer = roundToSatoshis(sourceAmountToSend * 1.1);

    // 2. Estimate what will be received for display purposes and fetch wallet data
    await estimateTargetAmount();
    await fetchAddressesAndBalances();
    
    loading = false;
  }

  async function estimateTargetAmount() {
    if (!sourceAmountToSend || !sourceCurrency || !targetCurrency) {
      return;
    }
    
    try {
      // Use the already calculated and rounded amountWithBuffer
      const conversionParams = {
          currency: sourceCurrency,
          convertTo: targetCurrency,
          amount: amountWithBuffer,  // Already rounded to 8 decimal places
          via: selectedNamespace.name !== targetCurrency ? selectedNamespace.name : undefined,
        };
      
      targetAmountNeeded = await invoke('estimate_currency_conversion', conversionParams);
      
    } catch (err: any) {
      // Don't block the UI, but set to 0 for now
      targetAmountNeeded = 0;
    }
  }

  async function fetchAddressesAndBalances() {
    if (amountWithBuffer === 0) {
      return;
    }

    try {
      const walletAddresses: string[] = await invoke('get_wallet_addresses');

      const addressesWithBalances: AddressWithBalance[] = [];
      const batchSize = 5; // Process 5 addresses at a time

      for (let i = 0; i < walletAddresses.length; i += batchSize) {
        const batch = walletAddresses.slice(i, i + batchSize);

        const balancePromises = batch.map(async (address) => {
          try {
            const balances: Record<string, number> = await invoke('get_address_currency_balances', { address });
            const availableAmount = balances[sourceCurrency] || 0;
            const hasEnoughBalance = availableAmount >= amountWithBuffer;

            return {
              success: true,
              data: {
                address,
                balances,
                hasEnoughBalance,
                availableAmount
              } as AddressWithBalance
            };
          } catch (e) {
            console.warn(`Failed to get balances for address ${address}:`, e);
            return { success: false, data: { address } };
          }
        });

        const batchResults = await Promise.all(balancePromises);

        for (const result of batchResults) {
          if (result.success) {
            const addr = result.data as AddressWithBalance;
            // Only add if it has the source currency
            if (addr.balances[sourceCurrency] && addr.balances[sourceCurrency] > 0) {
              addressesWithBalances.push(addr);
            }
          } else {
            console.log(`Skipping address ${(result.data as {address: string}).address} due to balance fetch failure`);
          }
        }
      }

      // Sort by hasEnoughBalance (true first) then by amount descending
      addressesWithBalances.sort((a, b) => {
        if (a.hasEnoughBalance !== b.hasEnoughBalance) {
          return a.hasEnoughBalance ? -1 : 1;
        }
        return b.availableAmount - a.availableAmount;
      });

      addresses = addressesWithBalances;

      // Calculate total balance for wildcard option
      totalSourceCurrencyBalance = addresses.reduce((sum, addr) => sum + (addr.availableAmount || 0), 0);

      // Setup dropdown options
      setupDropdownOptions();

      // Set default destination address
      if (addresses.length > 0) {
        selectedDestinationAddress = addresses[0].address;
      }

      // Get conversion rate
      await fetchConversionRate();

    } catch (err: any) {
      console.error('Failed to fetch addresses and balances:', err);
      error = `Failed to load wallet data: ${err.message || err}`;
    }
  }

  async function fetchConversionRate() {
    if (!sourceCurrency || !targetCurrency) return;

    fetchingRate = true;
    try {
      const rate: number = await invoke('estimate_currency_conversion', {
        currency: sourceCurrency,
        convertTo: targetCurrency,
        via: selectedNamespace.name !== targetCurrency ? selectedNamespace.name : undefined,
        amount: 1.0
      });

      conversionRate = rate;
    } catch (err: any) {
      // Don't block the UI for rate fetch failures
      conversionRate = null;
    } finally {
      fetchingRate = false;
    }
  }

  async function startConversion() {
    if (!internalCanProceed) return;

    converting = true;
    conversionError = null;

    try {
      // Get baseline balance of the destination address
      baselineBalance = await invoke<number>('get_address_currency_balance', { 
        address: selectedDestinationAddress, 
        currency: targetCurrency 
      });

      // Initiate conversion
      const txid: string = await invoke('send_currency_conversion', {
        fromAddress: selectedSourceAddress,
        toAddress: selectedDestinationAddress,
        fromCurrency: sourceCurrency,
        toCurrency: targetCurrency,
        amount: amountWithBuffer
      });

      conversionTxid = txid;
      conversionStarted = true;

      // Start timing and polling for completion
      startTimer();
      startPolling();

    } catch (err: any) {
      console.error('Failed to start conversion:', err);
      conversionError = `Failed to start conversion: ${err.message || err}`;
      converting = false;
    }
  }

  function startPolling() {
    pollingInterval = window.setInterval(async () => {
      try {
        // Check if conversion is complete by checking if new funds arrived in destination address
        const currentBalance = await invoke<number>('get_address_currency_balance', { 
          address: selectedDestinationAddress, 
          currency: targetCurrency 
        });
        
        // Check if new funds have arrived (balance increased from baseline)
        if (currentBalance > baselineBalance) {
          const newFundsReceived = currentBalance - baselineBalance;
          completeConversion(newFundsReceived);
          return;
        }

        // Continue polling - no timeout, conversions will eventually complete

      } catch (err) {
        // Continue polling unless it's a critical error
      }
    }, 10000); // Poll every 10 seconds
  }

  function completeConversion(newFundsReceived: number) {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
    stopTimer();

    converting = false;
    conversionStarted = false;
    conversionCompleted = true;
    actualAmountReceived = newFundsReceived;

    // Dispatch completed event for the modal to handle
    dispatch('completed', {
      txid: conversionTxid || '',
      actualAmountReceived: newFundsReceived
    });
  }

  function retryConversion() {
    conversionError = null;
    conversionTxid = null;
    conversionStarted = false;
    converting = false;
    conversionCompleted = false;
    actualAmountReceived = 0;
    
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
    stopTimer();

    // Refresh data and try again
    initialize();
  }

  function formatCurrency(amount: number, decimals: number = 8): string {
    return amount.toFixed(decimals).replace(/\.?0+$/, '');
  }

  function roundToSatoshis(amount: number): number {
    // Round to 8 decimal places to avoid floating point precision issues
    return Math.round(amount * 100000000) / 100000000;
  }

  function getAddressDisplayName(address: string): string {
    if (address === '*') return 'Use all addresses';
    return `${address.slice(0, 8)}...${address.slice(-6)}`;
  }

  function setupDropdownOptions() {
    // Source address options (include wildcard + addresses with balances)
    sourceAddressOptions = [
      {
        id: '*',
        name: 'Use all addresses',
        enabled: totalSourceCurrencyBalance >= amountWithBuffer,
        balance: `${formatCurrency(totalSourceCurrencyBalance, 5)} ${sourceCurrency}`
      },
      ...addresses.map(addr => ({
        id: addr.address,
        name: getAddressDisplayName(addr.address),
        enabled: addr.hasEnoughBalance,
        balance: addr.hasEnoughBalance ? `${formatCurrency(addr.availableAmount, 5)} ${sourceCurrency}` : 'Insufficient balance'
      }))
    ];

    // Destination address options (all addresses)
    destinationAddressOptions = addresses.map(addr => ({
      id: addr.address,
      name: getAddressDisplayName(addr.address),
      enabled: true,
      balance: null
    }));
  }

  function handleSourceAddressChange(event: CustomEvent<string | number | null>) {
    const newAddress = typeof event.detail === 'string' ? event.detail : '*';
    selectedSourceAddress = newAddress;
    handleSourceAddressChangeInternal();
  }

  function handleDestinationAddressChange(event: CustomEvent<string | number | null>) {
    const newAddress = typeof event.detail === 'string' ? event.detail : '';
    selectedDestinationAddress = newAddress;
  }

  function handleSourceAddressChangeInternal() {
    // Update destination address to match source if it's not wildcard
    if (selectedSourceAddress !== '*') {
      selectedDestinationAddress = selectedSourceAddress;
    } else if (addresses.length > 0) {
      selectedDestinationAddress = addresses[0].address;
    }
  }

  function showBufferTooltipOnHover() {
    showBufferTooltip = true;
  }

  function hideBufferTooltipOnLeave() {
    showBufferTooltip = false;
  }

  function startTimer() {
    conversionStartTime = Date.now();
    elapsedSeconds = 0;
    
    if (timerInterval) {
      clearInterval(timerInterval);
    }
    
    timerInterval = setInterval(() => {
      elapsedSeconds = Math.floor((Date.now() - conversionStartTime) / 1000);
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function formatElapsedTime(): string {
    const minutes = Math.floor(elapsedSeconds / 60);
    const seconds = elapsedSeconds % 60;
    if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    }
    return `${seconds}s`;
  }

  function getProgressMessage(elapsedSeconds: number) {
    if (elapsedSeconds < 120) { // 0-2 minutes
      return 'Processing conversion (typically 1-5 minutes)';
    } else if (elapsedSeconds < 300) { // 2-5 minutes
      return 'Conversion in progress - blockchain timing varies';
    } else if (elapsedSeconds < 600) { // 5-10 minutes
      return 'Taking longer than average, but this is normal';
    } else { // 10+ minutes
      return 'Current block is taking longer to be found - your conversion will complete';
    }
  }

  function getProgressDescription(elapsedSeconds: number) {
    if (elapsedSeconds < 120) { // 0-2 minutes
      return 'Your conversion is queued in the liquidity pool for processing.';
    } else if (elapsedSeconds < 300) { // 2-5 minutes
      return 'Waiting for the liquidity pool to process your conversion - timing naturally varies.';
    } else if (elapsedSeconds < 600) { // 5-10 minutes
      return 'This is still within normal range - LP processing can take up to 10 blocks.';
    } else { // 10+ minutes
      return 'Still waiting for liquidity pool processing - this will complete within the normal range.';
    }
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h3 class="text-lg font-medium text-dark-text-primary mb-2">
      Currency Conversion Required
    </h3>
    <p class="text-sm text-dark-text-secondary">
      Convert {sourceCurrency} to {targetCurrency} to complete your VerusID registration.
    </p>
  </div>

  {#if loading}
    <!-- Loading State -->
    <div class="space-y-4">
      <div class="bg-black/40 border border-white/10 rounded-lg p-4">
        <div class="space-y-3">
          <div class="h-4 bg-white/10 rounded animate-pulse w-32"></div>
          <div class="h-6 bg-white/10 rounded animate-pulse w-48"></div>
        </div>
      </div>
    </div>

  {:else if error}
    <!-- Error State -->
    <div class="bg-red-900/40 border border-red-600/50 rounded-lg p-4">
      <div class="flex items-start space-x-3">
        <svg class="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div class="flex-1">
          <h4 class="text-sm font-medium text-red-400 mb-1">Error Loading Conversion Data</h4>
          <p class="text-sm text-red-300">{error}</p>
          <Button
            variant="secondary"
            size="small"
            on:click={retryConversion}
          >
            Retry
          </Button>
        </div>
      </div>
    </div>

  {:else if conversionStarted || converting}
    <!-- Conversion Progress State -->
    <div class="space-y-4">
      <!-- Progress Display -->
      <div class="bg-black/20 border border-brand-green/30 rounded-lg p-4">
        <div class="flex items-center space-x-3 mb-4">
          <div class="animate-spin h-5 w-5 border-2 border-brand-green border-t-transparent rounded-full"></div>
          <h4 class="text-sm font-medium text-brand-green">{progressMessage}</h4>
        </div>
        
        <div class="space-y-3">
          <div class="text-xs text-white/60">
            {progressDescription}
          </div>
          
          {#if elapsedSeconds > 0}
            <div class="text-xs text-white/60">
              Elapsed: {formatElapsedTime()}
              {#if elapsedSeconds >= 120}
                • Typically 1-5 minutes total
              {/if}
            </div>
          {/if}
        </div>
        
        <!-- Educational context about timing -->
        <div class="mt-3 text-xs text-white/50">
          <div class="flex items-center space-x-1">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
            <span>Conversions are processed by the DeFi protocol (1-10 blocks, typically 1-10 minutes)</span>
          </div>
        </div>
      </div>

      <!-- Conversion Details (Read-only) -->
      <div class="bg-black/40 border border-white/10 rounded-lg p-4">
        <h4 class="text-sm font-medium text-white mb-3">Conversion Details</h4>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-white/70">From:</span>
            <span class="text-white font-mono">{formatCurrency(amountWithBuffer)} {sourceCurrency}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">To:</span>
            <span class="text-white font-mono">~{formatCurrency(targetAmountNeeded)} {targetCurrency}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">From Address:</span>
            <span class="text-white font-mono text-xs">{getAddressDisplayName(selectedSourceAddress)}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">To Address:</span>
            <span class="text-white font-mono text-xs">{getAddressDisplayName(selectedDestinationAddress)}</span>
          </div>
        </div>
      </div>
    </div>

  {:else if conversionCompleted}
    <!-- Conversion Completed State -->
    <div class="bg-green-900/40 border border-green-600/50 rounded-lg p-4">
      <div class="flex items-start space-x-3">
        <svg class="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <div class="flex-1">
          <h4 class="text-sm font-medium text-green-400 mb-1">Conversion Successful!</h4>
          <p class="text-sm text-green-300">
            Your {sourceCurrency} has been converted to {targetCurrency}.
            You received {formatCurrency(actualAmountReceived)} {targetCurrency}.
          </p>
        </div>
      </div>
    </div>

  {:else}
    <!-- Conversion Setup State -->
    <div class="space-y-6">
      <!-- Address Selection -->
      <div class="space-y-4">
        <div class="flex items-center space-x-2">
          <!-- Source Address -->
          <div class="flex-1">
            <CustomDropdown
              label={`You send (${sourceCurrency})`}
              options={sourceAddressOptions}
              bind:selectedId={selectedSourceAddress}
              placeholder="-- Select source --"
              on:change={handleSourceAddressChange}
              selectedOptionBalance={selectedSourceBalance}
            />
          </div>

          <div class="pt-6">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white/50" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </div>

          <!-- Destination Address -->
          <div class="flex-1">
            <CustomDropdown
              label={`You receive (${targetCurrency})`}
              options={destinationAddressOptions}
              bind:selectedId={selectedDestinationAddress}
              placeholder="-- Select destination --"
              on:change={handleDestinationAddressChange}
            />
          </div>
        </div>
      </div>

      <!-- Conversion Preview -->
      <div class="bg-black/20 border border-white/10 rounded-lg p-3">
        <h4 class="text-sm font-medium text-white mb-2">Conversion Preview</h4>
        <div class="space-y-2">
                    <!-- Primary: You send -->
          <div class="flex justify-between items-center">
            <span class="text-sm text-white/70">You send:</span>
            <span class="font-mono text-white font-medium">{formatCurrency(sourceAmountToSend)} {sourceCurrency}</span>
          </div>
          
          <!-- Secondary: Conversion fee -->
          <div class="flex justify-between items-center">
            <span class="text-xs text-white/50">Conversion fee (0.025%):</span>
            <span class="text-xs font-mono text-white/60">{formatCurrency(conversionFee)} {sourceCurrency}</span>
          </div>
          
          <!-- Secondary: Buffer with tooltip -->
          <div class="flex justify-between items-center">
            <div class="flex items-center">
              <span class="text-xs text-white/50">Buffer (10%):</span>
              <div class="ml-1 relative"
                   role="button"
                   tabindex="0"
                   on:mouseenter={showBufferTooltipOnHover}
                   on:mouseleave={hideBufferTooltipOnLeave}>
                <HelpCircle size={12} class="text-white/40 hover:text-white/60" />
                
                <!-- Tooltip -->
                {#if showBufferTooltip}
                  <div class="absolute bottom-5 left-0 bg-black border border-dark-border-primary rounded-lg p-2 shadow-lg w-48 z-50 cursor-default select-none pointer-events-none">
                    <div class="text-xs text-white/90 leading-relaxed">
                      10% extra is included to account for price fluctuations during conversion.
                    </div>
                  </div>
                {/if}
              </div>
            </div>
            <span class="text-xs font-mono text-brand-green">+{formatCurrency(bufferAmount)} {sourceCurrency}</span>
          </div>
          
          <!-- Separator -->
          <div class="border-t border-white/10 my-2"></div>
          
          <!-- Primary: Total to convert -->
          <div class="flex justify-between items-center">
            <span class="text-sm text-white/70 font-medium">Total to convert:</span>
            <span class="font-mono text-white font-semibold">{formatCurrency(amountWithBuffer)} {sourceCurrency}</span>
          </div>
          
          <!-- Primary: You receive -->
          <div class="flex justify-between items-center">
            <span class="text-sm text-white/70 font-medium">You receive:</span>
            <span class="font-mono text-brand-green font-semibold">~{formatCurrency(targetAmountNeeded)} {targetCurrency}</span>
          </div>
          
          <!-- Tertiary: Exchange rate -->
          {#if fetchingRate}
            <div class="flex justify-between items-center pt-1">
              <span class="text-xs text-white/40">Exchange Rate:</span>
              <span class="text-xs text-white/40 animate-pulse">Loading...</span>
            </div>
          {:else if conversionRate}
            <div class="flex justify-between items-center pt-1">
              <span class="text-xs text-white/40">Exchange Rate:</span>
              <span class="text-xs font-mono text-white/40">1 {sourceCurrency} = {formatCurrency(conversionRate, 6)} {targetCurrency}</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Insufficient Balance Warning -->
      {#if !hasValidSourceAddress && selectedSourceAddress !== '*'}
        <div class="bg-amber-900/40 border border-amber-600/50 rounded-lg p-4">
          <div class="flex items-start space-x-3">
            <svg class="w-5 h-5 text-amber-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <div class="flex-1">
              <h4 class="text-sm font-medium text-amber-400 mb-1">Insufficient Balance</h4>
              <p class="text-sm text-amber-300">
                The selected address does not have enough {sourceCurrency} for this conversion. Please select a different address or use "All addresses".
              </p>
            </div>
          </div>
        </div>
      {/if}

      <!-- Conversion Error -->
      {#if conversionError}
        <div class="bg-red-900/40 border border-red-600/50 rounded-lg p-4">
          <div class="flex items-start space-x-3">
            <svg class="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
            <div class="flex-1">
              <h4 class="text-sm font-medium text-red-400 mb-1">Conversion Failed</h4>
              <p class="text-sm text-red-300 mb-3">{conversionError}</p>
              <Button
                variant="secondary"
                size="small"
                on:click={retryConversion}
              >
                Try Again
              </Button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
</style> 