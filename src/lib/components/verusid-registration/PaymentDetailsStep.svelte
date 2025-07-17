<!-- 
  Component: src/lib/components/verusid-registration/PaymentDetailsStep.svelte
  Description: Step 2 of VerusID registration - shows payment options and wallet balances
  Features detailed cost breakdown and affordable payment methods based on user's wallet
  Changes:
  - Created payment details step with purchase summary and payment options
  - Fetches wallet balances and calculates conversion costs for payment options
  - Shows only affordable payment methods to user
  - Handles referral discounts and USD pricing
  - Includes loading states and error handling
  - Follows PRD specifications for UI layout and user experience
  - Updated compact purchase summary design to save space
  - Enhanced payment method visual hierarchy with better contrast and shadows
  - Improved payment option cards with subtle backgrounds and hover effects
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';
  import { selectedBlockchain } from '$lib/stores/blockchain';
  import type { NamespaceOption } from '$lib/types';

  // Props from Step 1
  export let selectedNamespace: NamespaceOption;
  export let finalPrice: number; // Already includes referral discount if applied (in USD)
  export let isReferralApplied: boolean = false;
  export let referralDiscount: number = 0;
  export let previewId: string = '';
  export let usdEstimate: number | null = null;

  // State for converted price
  let namespaceCurrencyPrice: number | null = null;

  // Payment option interface
  interface PaymentOption {
    currencyName: string;
    displayName: string;
    requiredAmount: number;
    userBalance: number;
    isAffordable: boolean;
    isDirect: boolean; // true if namespace currency, false if conversion
    usdEstimate?: number;
    selected: boolean;
  }

  // State
  let walletInfo: any = null;
  let paymentOptions: PaymentOption[] = [];
  let selectedPaymentOption: PaymentOption | null = null;
  let loading = true;
  let error: string | null = null;
  let calculatingOptions = false;

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    dataChange: { selectedPaymentOption: PaymentOption | null; isValid: boolean };
  }>();

  // Computed
  $: isValid = selectedPaymentOption !== null && selectedPaymentOption.isAffordable;
  $: hasAffordableOptions = paymentOptions.some(option => option.isAffordable);
  $: isVerusBlockchain = $selectedBlockchain?.id === 'verus';

  // Auto-dispatch when data changes (removed to prevent double-click issues)
  // $: if (typeof window !== 'undefined') {
  //   dispatchDataChange();
  // }

  // Lifecycle
  onMount(() => {
    fetchWalletInfoAndCalculateOptions();
  });

  // Functions
  function dispatchDataChange() {
    dispatch('dataChange', {
      selectedPaymentOption,
      isValid
    });
  }

  async function fetchWalletInfoAndCalculateOptions() {
    loading = true;
    error = null;

    try {
      console.log('=== PAYMENT DETAILS DEBUG START ===');
      console.log('Selected namespace:', selectedNamespace);
      console.log('Final price:', finalPrice);
      console.log('Fee currency name:', selectedNamespace.fee_currency_name);
      
      // Step 1: Get wallet balances
      console.log('Step 1: Fetching wallet info...');
      walletInfo = await invoke('get_wallet_info');
      console.log('Wallet info received:', walletInfo);
      console.log('Native balance:', walletInfo.balance);
      console.log('Reserve balances:', walletInfo.reserve_balance);

      // Step 2: Get namespace currency details to find ITS reserves.
      // We must use the namespace's own name/ID, NOT the fee currency's name.
      console.log(`Step 2: Fetching currency details for the namespace itself: "${selectedNamespace.name}"`);
      const currencyDetails = await invoke('get_currency', { 
        currencyname: selectedNamespace.name 
      });
      console.log('Currency details received:', currencyDetails);
      console.log('Currency bestcurrencystate:', (currencyDetails as any).bestcurrencystate);

      // Step 3: Calculate payment options
      await calculatePaymentOptions(currencyDetails);
      console.log('=== PAYMENT DETAILS DEBUG END ===');

    } catch (err: any) {
      console.error('Failed to fetch wallet info or calculate options:', err);
      error = `Failed to load payment options: ${err.message || err}`;
    } finally {
      loading = false;
    }
  }

  async function calculatePaymentOptions(currencyDetails: any) {
    calculatingOptions = true;
    const options: PaymentOption[] = [];

    try {
      console.log('=== CALCULATE PAYMENT OPTIONS DEBUG ===');
      console.log('Blockchain ID:', $selectedBlockchain?.id);
      console.log('Wallet balance (native):', walletInfo.balance);
      console.log('Reserve balances:', walletInfo.reserve_balance);
      
      // Get user's available currencies (native balance + reserve balances)
      const userCurrencies = new Set<string>();
      
      // Add native currency (VRSC/VRSCTEST) - get the currency ID from currencynames
      const nativeCurrencyName = $selectedBlockchain?.id === 'verus' ? 'VRSC' : 'VRSCTEST';
      console.log('Native currency name determined as:', nativeCurrencyName);
      console.log('Native balance check:', walletInfo.balance, '> 0 =', walletInfo.balance > 0);
      
      if (walletInfo.balance > 0) {
        userCurrencies.add(nativeCurrencyName);
        console.log('Added native currency to user holdings:', nativeCurrencyName);
      } else {
        console.log('Native balance is 0, not adding to user holdings');
      }
      
      // Add reserve currencies the user holds
      if (walletInfo.reserve_balance) {
        console.log('Processing reserve balances...');
        Object.keys(walletInfo.reserve_balance).forEach(currency => {
          const balance = walletInfo.reserve_balance[currency];
          console.log(`Reserve currency ${currency}: balance = ${balance}`);
          if (balance > 0) {
            userCurrencies.add(currency);
            console.log(`Added reserve currency to user holdings: ${currency}`);
          }
        });
      } else {
        console.log('No reserve_balance object found');
      }

      console.log('Final user currencies:', Array.from(userCurrencies));

      // --- Intersection Logic ---
      console.log('--- Intersection Logic ---');
      console.log('User has (names):', Array.from(userCurrencies));
      
      const currencyDetailsTyped = currencyDetails as any;
      const idToNameMap = new Map<string, string>();
      if (currencyDetailsTyped.currencynames) {
        Object.entries(currencyDetailsTyped.currencynames).forEach(([id, name]) => {
          idToNameMap.set(id, name as string);
        });
      }
      console.log('ID to Name map:', idToNameMap);

      const reserveCurrencyNames = new Set<string>();
      if (currencyDetailsTyped.bestcurrencystate?.reservecurrencies) {
        console.log(`Found ${currencyDetailsTyped.bestcurrencystate.reservecurrencies.length} reserve currencies in bestcurrencystate. Processing...`);
        currencyDetailsTyped.bestcurrencystate.reservecurrencies.forEach((reserve: any, index: number) => {
          console.log(`  Reserve ${index + 1}:`, reserve);
          console.log(`    -> currencyid: ${reserve.currencyid}`);
          const name = idToNameMap.get(reserve.currencyid);
          console.log(`    -> Mapped name: ${name}`);
          if (name) {
            reserveCurrencyNames.add(name);
            console.log(`    -> Added "${name}" to accepted currencies.`);
          } else {
            reserveCurrencyNames.add(reserve.currencyid);
            console.warn(`    -> Could not find name for ID ${reserve.currencyid}. Added ID as fallback.`);
          }
        });
      } else {
        console.log('No `reservecurrencies` array found in `bestcurrencystate`.');
      }
      // Also add the namespace's own currency name, as you should always be able to pay with the thing you're registering
      reserveCurrencyNames.add(selectedNamespace.name);
      
      // Also add the fee currency itself by name, as it's always a valid payment option
      reserveCurrencyNames.add(selectedNamespace.fee_currency_name);
      console.log('Final list of accepted currency names (reserves + self + fee currency):', reserveCurrencyNames);

      const viableCurrencies = Array.from(userCurrencies).filter(currencyName => 
        reserveCurrencyNames.has(currencyName)
      );
      
      console.log('Viable payment currencies after intersection:', viableCurrencies);
      
      if (viableCurrencies.length === 0) {
        console.log('WARNING: No viable currencies found!');
      }

      // First, calculate the price in the namespace's own currency (for display purposes only)
      console.log('--- Price Calculation ---');
      console.log(`Canonical fee is: ${finalPrice} ${selectedNamespace.fee_currency_name}`);
      console.log(`Converting to namespace's own currency: ${selectedNamespace.name}`);
      
      try {
        // Only convert if the fee currency is different from the namespace's own currency
        if (selectedNamespace.fee_currency_name !== selectedNamespace.name) {
          namespaceCurrencyPrice = await invoke('estimate_currency_conversion', {
            currency: selectedNamespace.fee_currency_name,
            convertTo: selectedNamespace.name,
            amount: finalPrice
          });
          console.log(`Display price in ${selectedNamespace.name}: ${namespaceCurrencyPrice}`);
        } else {
          // If the fee is already in the namespace currency, just use it
          namespaceCurrencyPrice = finalPrice;
          console.log(`Display price is already in ${selectedNamespace.name}: ${namespaceCurrencyPrice}`);
        }
      } catch (e: any) {
        console.error(`Failed to calculate display price in ${selectedNamespace.name}:`, e);
        // Fallback for display - show the original fee
        namespaceCurrencyPrice = finalPrice;
      }


      // --- Payment Option Calculation ---
      // For each viable currency, calculate the cost based on the *original* canonical fee.
      console.log('--- Payment Option Calculation ---');
      for (const currencyName of viableCurrencies) {
        try {
          console.log(`Processing payment option for currency: ${currencyName}`);
          let requiredAmount: number;
          let userBalance: number;
          
          const isDirect = currencyName === selectedNamespace.name;
          console.log(`Is direct payment (paying with namespace's own currency): ${isDirect}`);

          // Determine the required amount for this payment option
          if (isDirect) {
            // This is the namespace's own currency. We already calculated this price. No API call needed.
            requiredAmount = namespaceCurrencyPrice || 0; // Use the pre-calculated display price
            console.log(`Direct payment with namespace currency. Required: ${requiredAmount}`);
          
          } else if (currencyName === selectedNamespace.fee_currency_name) {
            // This is the fee currency. No conversion needed.
            requiredAmount = finalPrice;
            console.log(`Paying with the fee currency itself. Required: ${requiredAmount}`);

          } else {
            // This is a reserve currency that requires conversion.
            const conversionParams: {
              currency: string;
              convertTo: string;
              amount: number;
              via?: string;
            } = {
              currency: selectedNamespace.fee_currency_name,
              convertTo: currencyName,
              amount: finalPrice,
            };

            if (selectedNamespace.fee_currency_name !== selectedNamespace.name) {
              conversionParams.via = selectedNamespace.name;
              console.log(`Converting ${finalPrice} ${selectedNamespace.fee_currency_name} to ${currencyName} via ${conversionParams.via}`);
            } else {
              console.log(`Converting ${finalPrice} ${selectedNamespace.fee_currency_name} to ${currencyName} (direct reserve conversion)`);
            }
            
            requiredAmount = await invoke('estimate_currency_conversion', conversionParams);
            console.log(`Converted required amount: ${requiredAmount} ${currencyName}`);
          }

          // Get user's balance for this currency
          if (currencyName === 'VRSC' || currencyName === 'VRSCTEST') {
            userBalance = walletInfo.balance;
          } else {
            userBalance = walletInfo.reserve_balance[currencyName] || 0;
          }
          console.log(`User balance for ${currencyName}: ${userBalance}`);

          // Account for network fees (small buffer)
          const networkFee = walletInfo.paytxfee || 0.0001;
          const isAffordable = userBalance >= (requiredAmount + networkFee);

          // Calculate USD estimate for the balance if on Verus
          let usdEstimate: number | undefined;
          if (isVerusBlockchain && userBalance > 0) {
            try {
              usdEstimate = await invoke<number>('estimate_currency_conversion', {
                currency: currencyName,
                convertTo: 'dai.veth',
                via: 'bridge.veth',
                amount: userBalance
              });
            } catch (e) {
              console.warn(`Could not estimate USD value for ${currencyName} balance:`, e);
              // Ignore if this estimation fails
            }
          }

          const option: PaymentOption = {
            currencyName: currencyName,
            displayName: currencyName === 'VRSC' ? 'Verus (VRSC)' : 
                        currencyName === 'VRSCTEST' ? 'Verus Testnet (VRSCTEST)' :
                        `${currencyName}`,
            requiredAmount,
            userBalance,
            isAffordable,
            isDirect,
            usdEstimate,
            selected: false
          };

          options.push(option);
          console.log(`Payment option: ${currencyName} - Required: ${requiredAmount}, Balance: ${userBalance}, Affordable: ${isAffordable}, direct=${option.isDirect}`);

        } catch (conversionError: any) {
          console.warn(`Failed to calculate conversion for ${currencyName}:`, conversionError);
          // Continue with other currencies
        }
      }

      // Sort options: affordable first, then direct payments first, then by currency name
      options.sort((a, b) => {
        if (a.isAffordable !== b.isAffordable) {
          return a.isAffordable ? -1 : 1;
        }
        if (a.isDirect !== b.isDirect) {
          return a.isDirect ? -1 : 1;
        }
        return a.currencyName.localeCompare(b.currencyName);
      });

      paymentOptions = options;
      console.log('=== FINAL PAYMENT OPTIONS SUMMARY ===');
      console.log('Total options created:', paymentOptions.length);
      console.log('Affordable options:', paymentOptions.filter(opt => opt.isAffordable).length);
      console.log('Options details:');
      paymentOptions.forEach((option, index) => {
        console.log(`  ${index + 1}. ${option.currencyName}: required=${option.requiredAmount}, balance=${option.userBalance}, affordable=${option.isAffordable}, direct=${option.isDirect}`);
      });

    } catch (err: any) {
      console.error('Error calculating payment options:', err);
      error = `Failed to calculate payment options: ${err.message || err}`;
    } finally {
      calculatingOptions = false;
    }
  }

  function selectPaymentOption(option: PaymentOption) {
    if (!option.isAffordable) return;
    
    // Deselect all options
    paymentOptions.forEach(opt => opt.selected = false);
    
    // Select this option
    option.selected = true;
    selectedPaymentOption = option;
    
    // Trigger reactivity
    paymentOptions = [...paymentOptions];
    
    dispatchDataChange();
  }

  function formatCurrency(amount: number, decimals: number = 8): string {
    return amount.toFixed(decimals).replace(/\.?0+$/, '');
  }

  function formatUsd(amount: number): string {
    return `~$${amount.toFixed(2)} USD`;
  }

  async function refreshBalances() {
    await fetchWalletInfoAndCalculateOptions();
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h3 class="text-lg font-medium text-dark-text-primary mb-2">
      Payment Details
    </h3>
    <p class="text-sm text-dark-text-secondary">
      Review the cost and choose how you'd like to pay for your VerusID.
    </p>
  </div>

  {#if loading}
    <!-- Loading State -->
    <div class="space-y-4">
      <!-- Purchase Summary Skeleton -->
      <div class="bg-black/40 border border-white/10 rounded-lg p-4">
        <div class="space-y-3">
          <div class="h-4 bg-white/10 rounded animate-pulse w-32"></div>
          <div class="h-6 bg-white/10 rounded animate-pulse w-48"></div>
          <div class="h-4 bg-white/10 rounded animate-pulse w-24"></div>
        </div>
      </div>
      
      <!-- Payment Options Skeleton -->
      <div class="space-y-3">
        <div class="h-5 bg-white/10 rounded animate-pulse w-40"></div>
        {#each Array(2) as _}
          <div class="bg-black/40 border border-white/10 rounded-lg p-4">
            <div class="space-y-2">
              <div class="h-4 bg-white/10 rounded animate-pulse w-36"></div>
              <div class="h-4 bg-white/10 rounded animate-pulse w-28"></div>
            </div>
          </div>
        {/each}
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
          <h4 class="text-sm font-medium text-red-400 mb-1">Error Loading Payment Options</h4>
          <p class="text-sm text-red-300">{error}</p>
          <Button
            variant="secondary"
            size="small"
            on:click={refreshBalances}
          >
            Retry
          </Button>
        </div>
      </div>
    </div>

  {:else}
    <!-- Purchase Summary - Compact -->
    <div class="bg-black/20 border border-white/5 rounded-md px-3 py-2">
      <div class="flex items-center justify-between">
        <!-- Left side: VerusID -->
        <div class="flex items-center space-x-2">
          <span class="text-xs text-white/60">Creating:</span>
          <span class="text-xs font-mono text-white font-medium">{previewId}</span>
          {#if isReferralApplied}
            <span class="text-xs text-green-400 bg-green-400/10 px-1.5 py-0.5 rounded">
              {Math.round(referralDiscount * 100)}% off
            </span>
          {/if}
        </div>
        
        <!-- Right side: Price -->
        <div class="text-right">
          <div class="text-xs font-mono text-white font-medium">
            {#if namespaceCurrencyPrice}
              {formatCurrency(namespaceCurrencyPrice)} {selectedNamespace.name}
            {:else}
              <span class="text-white/60">Calculating...</span>
            {/if}
          </div>
          {#if isVerusBlockchain && usdEstimate}
            <div class="text-xs font-mono text-white/50">~${usdEstimate.toFixed(2)} USD</div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Payment Options Section -->
    <div class="space-y-4">
      <div class="flex items-center space-x-2">
        <h4 class="text-base font-medium text-white">Choose Payment Method</h4>
        <div class="h-px bg-white/10 flex-1"></div>
      </div>
      
      {#if calculatingOptions}
        <div class="text-center py-8">
          <div class="animate-spin h-6 w-6 border-2 border-brand-green border-t-transparent rounded-full mx-auto mb-2"></div>
          <p class="text-sm text-white/60">Calculating payment options...</p>
        </div>
      
      {:else if paymentOptions.length > 0}
        <!-- Payment Option Cards -->
        <div class="space-y-3">
          {#each paymentOptions as option (option.currencyName)}
            <div
              class="border-2 rounded-lg p-4 transition-all duration-200 relative
                {option.isAffordable ? 'cursor-pointer' : 'cursor-not-allowed opacity-50'}
                {option.selected && option.isAffordable
                  ? 'bg-brand-green/15 border-dark-border-primary ring-2 ring-brand-green'
                  : 'bg-white/5 border-dark-border-primary'}
                {option.isAffordable ? 'hover:bg-white/10 hover:border-brand-green/40' : ''}"
              on:click={(e) => {
                if (option.isAffordable) {
                  selectPaymentOption(option);
                } else {
                  e.preventDefault();
                  e.currentTarget.blur();
                }
              }}
              role="radio"
              tabindex={option.isAffordable ? 0 : -1}
              aria-checked={option.selected}
              on:keydown={(e) => {
                if (e.key === 'Enter' && option.isAffordable) {
                  selectPaymentOption(option);
                }
              }}
            >
              <div class="flex items-center justify-between">
                <!-- Radio button and currency info -->
                <div class="flex items-center space-x-3">
                  <div class="flex-shrink-0">
                    <div
                      class="w-4 h-4 rounded-full border-2 flex items-center justify-center
                      {option.selected && option.isAffordable
                        ? 'border-brand-green bg-brand-green shadow-md'
                        : 'border-white/50 bg-white/5'}"
                    >
                      {#if option.selected && option.isAffordable}
                        <div class="w-2 h-2 bg-white rounded-full" />
                      {/if}
                    </div>
                  </div>

                  <div>
                    <div class="text-sm font-medium text-white">
                      Pay with {option.displayName}
                    </div>
                    <div class="text-xs text-white/60">
                      {option.isDirect ? 'Direct payment' : 'Requires conversion'}
                    </div>
                  </div>
                </div>

                <!-- Cost and balance info -->
                <div class="text-right">
                  <div class="text-sm font-mono text-white">
                    {formatCurrency(option.requiredAmount)} {option.currencyName}
                  </div>
                  {#if option.isAffordable}
                    <div class="text-xs text-white/60">
                      Balance: {formatCurrency(option.userBalance)} {option.currencyName}
                    </div>
                  {:else}
                    <div class="text-xs text-amber-400">
                      You need {formatCurrency(option.requiredAmount - option.userBalance)} more
                    </div>
                  {/if}
                  {#if option.usdEstimate && option.isAffordable}
                    <div class="text-xs text-white/50">
                      {formatUsd(option.usdEstimate)}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>

        {#if !hasAffordableOptions}
          <!-- Insufficient Funds State -->
          <div class="bg-amber-900/40 border border-amber-600/50 rounded-lg p-4 mt-4">
            <div class="flex items-start space-x-3">
              <svg
                class="w-5 h-5 text-amber-400 mt-0.5 flex-shrink-0"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                  clip-rule="evenodd"
                />
              </svg>
              <div class="flex-1">
                <h4 class="text-sm font-medium text-amber-400 mb-2">Insufficient Funds</h4>
                <p class="text-sm text-amber-300 mb-3">
                  You don't have enough funds to purchase this VerusID with any of your current
                  assets. Please fund your wallet and try again.
                </p>
                <Button variant="secondary" size="small" on:click={refreshBalances}>
                  Refresh Balances
                </Button>
              </div>
            </div>
          </div>
        {/if}
      {:else}
        <!-- No viable payment options at all -->
        <div class="bg-amber-900/40 border border-amber-600/50 rounded-lg p-4">
          <div class="flex items-start space-x-3">
            <svg
              class="w-5 h-5 text-amber-400 mt-0.5 flex-shrink-0"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fill-rule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clip-rule="evenodd"
              />
            </svg>
            <div class="flex-1">
              <h4 class="text-sm font-medium text-amber-400 mb-2">No Payment Options Found</h4>
              <p class="text-sm text-amber-300 mb-3">
                To create <span class="font-mono">{previewId}</span>, you need to hold one of the
                currencies it accepts for payment. We couldn't find any compatible currencies in
                your wallet.
              </p>
              <p class="text-sm text-amber-300 mb-3">
                Please fund your wallet with a supported currency and try again.
              </p>
              <Button variant="secondary" size="small" on:click={refreshBalances}>
                Refresh Balances
              </Button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* Ensure radio button accessibility */
  [role="radio"]:focus {
    outline: 2px solid theme('colors.brand-green.DEFAULT');
    outline-offset: 2px;
  }
</style> 