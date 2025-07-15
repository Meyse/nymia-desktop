<!-- 
  Component: src/lib/components/verusid-registration/IdentityInfoStep.svelte
  Description: First step of VerusID registration - handles name and namespace selection
  Features single-line UI with name input and namespace dropdown in format "name.namespace@"
  Changes:
  - Created compact single-line UI with name input on left, namespace dropdown on right
  - Styled namespace dropdown to match CustomDropdown.svelte design
  - Fetches available namespaces from blockchain using get_available_namespaces API
  - Filters namespaces based on options (33 or 41) and proofprotocol (1)
  - Visual separators with "." and "@" symbols
  - Proper validation and error handling
  - Added USD pricing for Verus blockchain using estimateconversion RPC calls
  - Shows ~$X.XX USD instead of native currency amounts for Verus blockchain only
  - Handles both direct namespace currency fees and reserve currency fees
  - Includes referral discount USD pricing calculations
  - Added root currency option at top of dropdown showing only [root] label
  - Fetches root currency data using getcurrency RPC call
  - Root selection creates name@ format (no parent namespace) instead of name.parent@
  - Compact design with minimal spacing and smaller components
  - Referral input expects complete ID (e.g., john.namespace@) with no preview
  - Removed USD pricing status messages for cleaner UI
  - Added real-time VerusID name availability checking with debounce
  - Added referral code validation with namespace matching and existence check
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';
  import { selectedBlockchain } from '$lib/stores/blockchain';
  import type { NamespaceOption } from '$lib/types';

  // Props
  export let name: string = '';
  export let selectedNamespace: NamespaceOption | null = null;

  // State
  let namespaces: NamespaceOption[] = [];
  let fetchingNamespaces = false;
  let fetchError: string | null = null;
  let showDropdown = false;
  let showingSkeleton = false;
  let inputElement: HTMLInputElement;
  let dropdownElement: HTMLDivElement;
  let referralCode: string = '';
  let showReferralInput = false;
  
  // Validation State
  let nameAvailabilityStatus: 'unchecked' | 'checking' | 'available' | 'taken' | 'error' = 'unchecked';
  let nameAvailabilityMessage: string | null = null;
  let referralStatus: 'unchecked' | 'checking' | 'valid' | 'invalid' | 'error' = 'unchecked';
  let referralMessage: string | null = null;
  let debounceTimeout: number;


  // Root currency state
  let rootCurrency: NamespaceOption | null = null;
  let fetchingRootCurrency = false;
  let rootCurrencyError: string | null = null;
  
  // USD pricing state
  let vrscToUsdRate: number | null = null;
  let fetchingUsdPrice = false;
  let usdPriceError: string | null = null;
  let namespaceUsdPrices = new Map<string, number | null>(); // namespace ID -> USD price

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    dataChange: { name: string; namespace: NamespaceOption | null; isValid: boolean; referralCode: string; preview: string; isNameAvailable: boolean; isReferralValid: boolean; };
  }>();

  // Computed
  $: previewId = name && selectedNamespace ? 
    (isRootNamespace(selectedNamespace) ? `${name}@` : `${name}.${selectedNamespace.name}@`) : '';
  
  $: isNameValid = isValidName(name);
  $: isNameAvailable = nameAvailabilityStatus === 'available';
  $: isReferralValid = referralStatus === 'valid' || (referralStatus === 'unchecked' && referralCode.trim() === '');
  $: isValid = isNameValid && selectedNamespace !== null && isNameAvailable && isReferralValid;
  
  // Auto-dispatch when any relevant data changes
  $: if (typeof window !== 'undefined') {
    // List all dependencies explicitly to ensure reactivity
    const deps = { name, selectedNamespace, isValid, referralCode: referralCode.trim(), previewId, isNameAvailable, isReferralValid };
    dispatchDataChange();
  }

  // Lifecycle
  onMount(() => {
    fetchNamespaces();
    fetchRootCurrency();
    // Close dropdown when clicking outside
    document.addEventListener('click', handleOutsideClick);
    return () => {
      document.removeEventListener('click', handleOutsideClick);
      clearTimeout(debounceTimeout);
    };
  });

  // Functions
  function isValidName(inputName: string): boolean {
    // Name is valid if it has content and doesn't exceed max length
    // Character blocking is handled at input time via handleKeyPress
    return inputName.trim().length > 0 && inputName.length <= 20;
  }

  async function fetchNamespaces() {
    fetchingNamespaces = true;
    fetchError = null;
    showingSkeleton = true;

    // Show skeleton namespaces while loading
    namespaces = Array.from({ length: 5 }, (_, i) => ({
      name: 'Loading...',
      currency_id: `skeleton-${i}`,
      registration_fee: 0,
      fully_qualified_name: 'Loading...',
      fee_currency_name: 'Loading...',
      options: 0,
      id_referral_levels: 0
    }));

    try {
      console.log('Fetching available namespaces...');
      const result = await invoke<NamespaceOption[]>('get_available_namespaces');
      namespaces = result;
      console.log(`Loaded ${namespaces.length} available namespaces`);
    } catch (error: any) {
      console.error('Failed to fetch namespaces:', error);
      fetchError = `Failed to load namespaces: ${error.message || error}`;
      namespaces = []; // Clear skeleton on error
    } finally {
      fetchingNamespaces = false;
      showingSkeleton = false;
    }
  }

  async function fetchRootCurrency() {
    if (!$selectedBlockchain?.id) {
      console.log('No blockchain selected, skipping root currency fetch');
      return;
    }

    fetchingRootCurrency = true;
    rootCurrencyError = null;

    try {
      console.log('Fetching root currency for blockchain:', $selectedBlockchain.id);
      const result = await invoke<NamespaceOption>('get_root_currency', {
        blockchainId: $selectedBlockchain.id
      });
      rootCurrency = result;
      console.log(`Loaded root currency: ${rootCurrency.name} (fee: ${rootCurrency.registration_fee} ${rootCurrency.fee_currency_name})`);
    } catch (error: any) {
      console.error('Failed to fetch root currency:', error);
      rootCurrencyError = `Failed to load root currency: ${error.message || error}`;
      rootCurrency = null; // Hide root currency on error
    } finally {
      fetchingRootCurrency = false;
    }
  }

  function handleNameInput(event: Event) {
    const target = event.target as HTMLInputElement;
    let inputValue = target.value;
    
    // Filter out blocked characters: / : * ? " < > | @ . )
    const blockedChars = /[/:*?"<>|@.)]/g;
    const filteredValue = inputValue.replace(blockedChars, '');
    
    // Update the input field if we removed any characters
    if (filteredValue !== inputValue) {
      target.value = filteredValue;
      console.log('Filtered input:', inputValue, '→', filteredValue);
    }
    
    name = filteredValue;
    // Reset status on new input, then trigger debounced check
    nameAvailabilityStatus = 'unchecked';
    nameAvailabilityMessage = null;
    clearTimeout(debounceTimeout);
    debounceTimeout = window.setTimeout(() => {
      checkNameAvailability();
    }, 500); // 500ms delay

    dispatchDataChange();
  }

  function handleNamespaceSelect(namespace: NamespaceOption) {
    // Don't allow selection of skeleton items
    if (showingSkeleton || namespace.currency_id.startsWith('skeleton-')) {
      return;
    }
    
    selectedNamespace = namespace;
    showDropdown = false;

    // Re-validate name and referral against new namespace
    checkNameAvailability();
    resetReferralStatus();

    dispatchDataChange();
  }

  function handleDropdownToggle() {
    if (namespaces.length > 0) {
      showDropdown = !showDropdown;
    }
  }

  function handleOutsideClick(event: MouseEvent) {
    if (
      dropdownElement && 
      !dropdownElement.contains(event.target as Node) &&
      !inputElement?.contains(event.target as Node)
    ) {
      showDropdown = false;
    }
  }

  function dispatchDataChange() {
    dispatch('dataChange', {
      name: name,
      namespace: selectedNamespace,
      isValid,
      referralCode: referralCode.trim(),
      preview: previewId,
      isNameAvailable,
      isReferralValid
    });
  }

  function formatFee(fee: number): string {
    return fee.toFixed(8).replace(/\.?0+$/, '');
  }

  function calculateReferralDiscount(referralLevels: number): number {
    // referralLevels 0-5 maps to 1/2, 1/3, 1/4, 1/5, 1/6, 1/7
    return 1 / (referralLevels + 2);
  }

  function calculateDiscountedPrice(originalPrice: number, referralLevels: number): number {
    const discount = calculateReferralDiscount(referralLevels);
    return originalPrice * (1 - discount);
  }

  function getDiscountPercentage(referralLevels: number): number {
    const discount = calculateReferralDiscount(referralLevels);
    return Math.round(discount * 100);
  }

  function toggleReferralInput() {
    showReferralInput = !showReferralInput;
    if (!showReferralInput) {
      referralCode = '';
      resetReferralStatus();
    }
  }

  function isRootNamespace(namespace: NamespaceOption): boolean {
    if (!rootCurrency) return false;
    return namespace.currency_id === rootCurrency.currency_id || 
           (namespace.name.toLowerCase() === 'vrsc' || namespace.name.toLowerCase() === 'vrsctest');
  }

  // --- Validation Functions ---

  async function checkNameAvailability() {
    if (!name.trim() || !selectedNamespace) {
      nameAvailabilityStatus = 'unchecked';
      nameAvailabilityMessage = null;
      return;
    }

    nameAvailabilityStatus = 'checking';
    nameAvailabilityMessage = null;

    try {
      const fullId = isRootNamespace(selectedNamespace) 
        ? `${name}@` 
        : `${name}.${selectedNamespace.name}@`;
      
      console.log(`Checking availability of: ${fullId}`);
      
      const exists = await invoke<boolean>('check_identity_exists', { identityName: fullId });

      if (exists) {
        nameAvailabilityStatus = 'taken';
        nameAvailabilityMessage = 'This name is already taken in this namespace.';
      } else {
        nameAvailabilityStatus = 'available';
        nameAvailabilityMessage = 'This name is available.';
      }
    } catch (error: any) {
      console.error('Failed to check name availability:', error);
      nameAvailabilityStatus = 'error';
      nameAvailabilityMessage = `Error checking name: ${error.message || error}`;
    }
  }

  async function validateReferral() {
    if (!referralCode.trim() || !selectedNamespace) {
      referralStatus = 'invalid';
      referralMessage = 'Referral code cannot be empty.';
      return;
    }

    referralStatus = 'checking';
    referralMessage = null;

    // 1. Namespace check
    const referralParts = referralCode.split('.');
    const referralName = referralParts[0];
    const referralNamespaceAndAt = referralParts.length > 1 ? referralParts.slice(1).join('.') : '';
    const referralNamespace = referralNamespaceAndAt.endsWith('@') ? referralNamespaceAndAt.slice(0, -1) : referralNamespaceAndAt;

    const isRootSelected = isRootNamespace(selectedNamespace);

    if ((isRootSelected && referralNamespace !== '') || (!isRootSelected && referralNamespace.toLowerCase() !== selectedNamespace.name.toLowerCase())) {
        referralStatus = 'invalid';
        referralMessage = `Referral must be in the selected ".${selectedNamespace.name}@" namespace.`;
        return;
    }

    // 2. Existence check
    try {
      console.log(`Validating referral ID: ${referralCode}`);
      const exists = await invoke<boolean>('check_identity_exists', { identityName: referralCode });
      
      if (exists) {
        referralStatus = 'valid';
        referralMessage = 'Referral code is valid.';
      } else {
        referralStatus = 'invalid';
        referralMessage = 'This referral ID does not exist.';
      }
    } catch (error: any) {
      console.error('Failed to validate referral:', error);
      referralStatus = 'error';
      referralMessage = `Error validating referral: ${error.message || error}`;
    }
  }

  function resetReferralStatus() {
    referralStatus = 'unchecked';
    referralMessage = null;
    referralCode = '';
  }


  // USD Pricing Functions
  async function getVrscToUsdRate(): Promise<number | null> {
    try {
      console.log('Fetching VRSC to USD rate...');
      const vrscToDai = await invoke<number>('estimate_currency_conversion', {
        currency: 'vrsc',
        convertTo: 'dai.veth',
        via: 'bridge.veth',
        amount: 1.0
      });
      console.log(`VRSC to USD rate: $${vrscToDai.toFixed(4)}`);
      return vrscToDai;
    } catch (error) {
      console.error('Failed to get VRSC to USD rate:', error);
      return null;
    }
  }

  async function convertNamespaceToUsd(namespace: NamespaceOption): Promise<number | null> {
    if (!vrscToUsdRate) {
      console.log('No VRSC rate available for USD conversion');
      return null;
    }

    try {
      let vrscAmount: number;

      // Special case: If fee is already in VRSC (root currency), no conversion needed
      if (namespace.fee_currency_name.toLowerCase() === 'vrsc' || namespace.fee_currency_name.toLowerCase() === 'vrsctest') {
        console.log(`Fee already in VRSC: ${namespace.registration_fee} ${namespace.fee_currency_name}`);
        vrscAmount = namespace.registration_fee;
      }
      // Method A: Fee is in namespace currency itself
      else if (namespace.fee_currency_name === namespace.name) {
        console.log(`Converting ${namespace.name} -> VRSC (Method A: direct conversion)`);
        vrscAmount = await invoke<number>('estimate_currency_conversion', {
          currency: namespace.name.toLowerCase(),
          convertTo: 'vrsc',
          amount: namespace.registration_fee
        });
      } 
      // Method B: Fee is in a reserve currency
      else {
        console.log(`Converting ${namespace.fee_currency_name} -> VRSC (Method B: via ${namespace.name})`);
        vrscAmount = await invoke<number>('estimate_currency_conversion', {
          currency: namespace.fee_currency_name.toLowerCase(),
          convertTo: 'vrsc',
          via: namespace.name.toLowerCase(),
          amount: namespace.registration_fee
        });
      }

      const usdAmount = vrscAmount * vrscToUsdRate;
      console.log(`${namespace.name}: ${namespace.registration_fee} ${namespace.fee_currency_name} = ${vrscAmount.toFixed(4)} VRSC = $${usdAmount.toFixed(2)} USD`);
      return usdAmount;
    } catch (error) {
      console.error(`Failed to convert ${namespace.name} fee to USD:`, error);
      return null;
    }
  }

  async function updateUsdPricing() {
    // Only calculate USD prices for Verus blockchain
    if ($selectedBlockchain?.id !== 'verus') {
      console.log('Not Verus blockchain, skipping USD pricing');
      return;
    }

    fetchingUsdPrice = true;
    usdPriceError = null;
    
    try {
      // Get VRSC to USD rate first
      vrscToUsdRate = await getVrscToUsdRate();
      
      if (!vrscToUsdRate) {
        throw new Error('Failed to get VRSC to USD rate');
      }

      // Convert each namespace fee to USD (including root currency)
      const allNamespaces = [...namespaces, ...(rootCurrency ? [rootCurrency] : [])];
      const usdPricePromises = allNamespaces
        .filter(ns => !showingSkeleton && !ns.currency_id.startsWith('skeleton-'))
        .map(async (namespace) => {
          const usdPrice = await convertNamespaceToUsd(namespace);
          return { namespaceId: namespace.currency_id, usdPrice };
        });

      const results = await Promise.all(usdPricePromises);
      
      // Update the USD price map
      const newPriceMap = new Map<string, number | null>();
      results.forEach(({ namespaceId, usdPrice }) => {
        newPriceMap.set(namespaceId, usdPrice);
      });
      namespaceUsdPrices = newPriceMap;

      console.log('USD pricing updated for', results.length, 'namespaces');
    } catch (error) {
      console.error('Failed to update USD pricing:', error);
      usdPriceError = String(error);
    } finally {
      fetchingUsdPrice = false;
    }
  }

  // Helper function to format USD price
  function formatUsdPrice(usdAmount: number): string {
    return `~$${usdAmount.toFixed(2)} USD`;
  }

  function getNamespaceUsdPrice(namespace: NamespaceOption): number | null {
    return namespaceUsdPrices.get(namespace.currency_id) || null;
  }

  function getDiscountedUsdPrice(namespace: NamespaceOption): number | null {
    const originalUsd = getNamespaceUsdPrice(namespace);
    if (!originalUsd || !hasReferralSystem) return null;
    
    const discountFactor = 1 - calculateReferralDiscount(namespace.id_referral_levels);
    return originalUsd * discountFactor;
  }

  // Computed values
  $: hasReferralSystem = selectedNamespace?.options === 41;
  $: discountedPrice = selectedNamespace && hasReferralSystem ? 
    calculateDiscountedPrice(selectedNamespace.registration_fee, selectedNamespace.id_referral_levels) : 
    null;
  $: discountPercentage = selectedNamespace && hasReferralSystem ? 
    getDiscountPercentage(selectedNamespace.id_referral_levels) : 
    0;
  $: isVerusBlockchain = $selectedBlockchain?.id === 'verus';

  // Reactive statements for USD pricing
  $: if (isVerusBlockchain && namespaces.length > 0 && !showingSkeleton && !fetchingNamespaces && !fetchingRootCurrency) {
    updateUsdPricing();
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h3 class="text-lg font-medium text-dark-text-primary mb-2">
      Choose Your VerusID
    </h3>
    <p class="text-sm text-dark-text-secondary">
      Enter a unique name and select a namespace to create your VerusID.
    </p>
  </div>

  <!-- Combined Input Field -->
  <div class="space-y-4">
    <!-- Single Line: Name + Namespace -->
    <div>
      <label class="block text-sm font-medium text-white mb-2">
        VerusID
      </label>
      <div class="flex gap-2">
        <!-- Name Input (left side) -->
        <div class="flex-1">
          <div class="relative">
            <input
              bind:this={inputElement}
              type="text"
              bind:value={name}
              on:input={handleNameInput}
              placeholder="Enter name"
              class="w-full bg-black/60 border hover:border-brand-green/60 focus:border-brand-green rounded-lg shadow-lg px-3 py-2.5 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-brand-green/30
                {nameAvailabilityStatus === 'available' ? 'border-green-500/50' : ''}
                {nameAvailabilityStatus === 'taken' || nameAvailabilityStatus === 'error' ? 'border-red-500/50' : 'border-white/20'}"
              maxlength="20"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
            />
            <!-- Name availability icon -->
            <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
              {#if nameAvailabilityStatus === 'checking'}
                <svg class="animate-spin h-4 w-4 text-white/60" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              {:else if nameAvailabilityStatus === 'available'}
                <svg class="h-5 w-5 text-green-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                </svg>
              {:else if nameAvailabilityStatus === 'taken' || nameAvailabilityStatus === 'error'}
                <svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
              {/if}
            </div>
          </div>
          <div class="text-xs h-4 mt-1 px-1
            {nameAvailabilityStatus === 'available' ? 'text-green-400/80' : ''}
            {nameAvailabilityStatus === 'taken' || nameAvailabilityStatus === 'error' ? 'text-red-400/80' : 'text-white/50'}">
            {nameAvailabilityMessage || 'Not case sensitive • All character sets supported'}
          </div>
        </div>
        
        <!-- Dot separator -->
        <div class="flex items-center px-1">
          <span class="text-white text-lg font-bold">.</span>
        </div>
        
        <!-- Namespace Dropdown (right side) -->
        <div class="flex-1 relative" bind:this={dropdownElement}>
          {#if fetchingNamespaces}
            <button 
              type="button"
              class="relative w-full bg-black/60 border border-white/20 rounded-lg shadow-lg pl-3 pr-10 py-2.5 text-left cursor-default"
              disabled
            >
              <span class="block truncate font-medium min-h-[1.25rem]">
                <div class="h-5 bg-white/10 rounded animate-pulse w-24"></div>
              </span>
              
              <span class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <svg 
                  class="h-4 w-4 text-white/60"
                  xmlns="http://www.w3.org/2000/svg" 
                  viewBox="0 0 20 20" 
                  fill="currentColor" 
                  aria-hidden="true"
                >
                  <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </span>
            </button>
          {:else if fetchError}
            <div class="w-full bg-red-900/40 border border-red-600/50 rounded-lg px-3 py-2.5 text-red-400 text-sm">
              Error loading
              <button
                on:click={fetchNamespaces}
                class="ml-2 text-xs underline hover:no-underline"
              >
                Retry
              </button>
            </div>
          {:else}
            <button 
              type="button"
              class="relative w-full bg-black/60 border border-white/20 hover:border-brand-green/60 focus:border-brand-green rounded-lg shadow-lg pl-3 pr-10 py-2.5 text-left focus:outline-none focus:ring-2 focus:ring-brand-green/30 {showDropdown ? 'border-brand-green ring-2 ring-brand-green/30 bg-black/80' : ''}"
              on:click={handleDropdownToggle}
              aria-haspopup="listbox"
              aria-expanded={showDropdown}
            >
              <span class="block truncate font-medium {selectedNamespace ? 'text-white' : 'text-white/40'} min-h-[1.25rem]">
                {selectedNamespace ? 
                  (isRootNamespace(selectedNamespace) ? '\u00A0' : selectedNamespace.name) : 
                  'Select namespace'}
              </span>
              
              <span class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <svg 
                  class="h-4 w-4 text-white/60 transition-transform duration-150 {showDropdown ? 'rotate-180' : ''}"
                  xmlns="http://www.w3.org/2000/svg" 
                  viewBox="0 0 20 20" 
                  fill="currentColor" 
                  aria-hidden="true"
                >
                  <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </span>
            </button>

            <!-- Dropdown Menu -->
            {#if showDropdown && namespaces.length > 0}
              <div class="absolute z-50 mt-1 w-full">
                <div class="relative">
                  <ul 
                    class="bg-black/95 border border-white/10 shadow-2xl max-h-52 rounded-lg py-1 text-sm overflow-y-auto focus:outline-none backdrop-blur-sm"
                    tabindex="-1"
                    role="listbox"
                  >
                  <!-- Root Currency Option (always at top) -->
                  {#if rootCurrency}
                    <li 
                      class="relative select-none px-3 py-2.5 mx-1 rounded-md border border-transparent cursor-pointer hover:bg-white/5 hover:border-white/10 text-white {selectedNamespace?.currency_id === rootCurrency.currency_id ? 'bg-brand-green/15 text-brand-green border border-brand-green/30 font-medium' : ''}"
                      role="option"
                      aria-selected={selectedNamespace?.currency_id === rootCurrency.currency_id}
                      on:click={() => rootCurrency && handleNamespaceSelect(rootCurrency)}
                    >
                      <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-2">
                          <span class="block truncate text-xs text-white/50">[root]</span>
                        </div>
                        
                        <div class="flex items-center space-x-2 ml-2 flex-shrink-0">
                          <span class="text-xs text-white/60 font-mono {selectedNamespace?.currency_id === rootCurrency.currency_id ? 'text-brand-green/80' : ''}">
                            {#if isVerusBlockchain}
                              {#if fetchingUsdPrice}
                                <span class="animate-pulse">~$... USD</span>
                              {:else}
                                {@const usdPrice = getNamespaceUsdPrice(rootCurrency)}
                                {#if usdPrice}
                                  {formatUsdPrice(usdPrice)}
                                {:else}
                                  {formatFee(rootCurrency.registration_fee)} {rootCurrency.fee_currency_name}
                                {/if}
                              {/if}
                            {:else}
                              {formatFee(rootCurrency.registration_fee)} {rootCurrency.fee_currency_name}
                            {/if}
                          </span>
                          

                        </div>
                      </div>
                    </li>
                  {/if}
                  
                  <!-- Namespace Options -->
                  {#each namespaces as namespace}
                    <li 
                      class="relative select-none px-3 py-2.5 mx-1 rounded-md border border-transparent {showingSkeleton || namespace.currency_id.startsWith('skeleton-') ? 'cursor-default opacity-60' : 'cursor-pointer hover:bg-white/5 hover:border-white/10'} text-white {selectedNamespace?.currency_id === namespace.currency_id ? 'bg-brand-green/15 text-brand-green border border-brand-green/30 font-medium' : ''}"
                      role="option"
                      aria-selected={selectedNamespace?.currency_id === namespace.currency_id}
                      on:click={() => handleNamespaceSelect(namespace)}
                    >
                      <div class="flex items-center justify-between">
                        <span class="block truncate">
                          {#if showingSkeleton || namespace.currency_id.startsWith('skeleton-')}
                            <span class="animate-pulse">{namespace.name}</span>
                          {:else}
                            {namespace.name}
                          {/if}
                        </span>
                        
                        <div class="flex items-center space-x-2 ml-2 flex-shrink-0">
                          <span class="text-xs text-white/60 font-mono {selectedNamespace?.currency_id === namespace.currency_id ? 'text-brand-green/80' : ''}">
                            {#if showingSkeleton || namespace.currency_id.startsWith('skeleton-')}
                              <span class="animate-pulse">Loading...</span>
                            {:else if isVerusBlockchain}
                              {#if fetchingUsdPrice}
                                <span class="animate-pulse">~$... USD</span>
                              {:else}
                                {@const usdPrice = getNamespaceUsdPrice(namespace)}
                                {#if usdPrice}
                                  {formatUsdPrice(usdPrice)}
                                {:else}
                                  {formatFee(namespace.registration_fee)} {namespace.fee_currency_name}
                                {/if}
                              {/if}
                            {:else}
                              {formatFee(namespace.registration_fee)} {namespace.fee_currency_name}
                            {/if}
                          </span>
                          

                        </div>
                      </div>
                    </li>
                  {/each}
                  </ul>
                  
                  <!-- Scroll indicator gradient -->
                  {#if (namespaces.length + (rootCurrency ? 1 : 0)) > 4}
                    <div class="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-black/90 to-transparent rounded-b-lg pointer-events-none"></div>
                  {/if}
                </div>
              </div>
            {:else if showDropdown && namespaces.length === 0}
              <div class="absolute z-50 mt-1 w-full">
                <div class="bg-black/95 border border-white/10 shadow-2xl rounded-lg">
                  <div class="px-3 py-2 text-sm text-white/60">
                    No namespaces available
                  </div>
                </div>
              </div>
            {/if}
          {/if}
        </div>
        
        <!-- @ symbol -->
        <div class="flex items-center px-1">
          <span class="text-white text-lg font-bold">@</span>
        </div>
      </div>
      
    </div>
  </div>

  <!-- Price Information Section -->
  {#if selectedNamespace && !showingSkeleton && !selectedNamespace.currency_id.startsWith('skeleton-')}
    <div class="mt-4 space-y-2">
      <h4 class="text-sm font-medium text-white">Registration Fee</h4>
      
      <!-- Pricing -->
      <div class="space-y-1">
        <!-- Standard Price -->
        <div class="flex justify-between items-center">
          <span class="text-sm text-white/80">Standard:</span>
          <span class="text-sm font-mono text-white">
            {#if isVerusBlockchain}
              {#if fetchingUsdPrice}
                <span class="animate-pulse">~$... USD</span>
              {:else}
                {@const usdPrice = getNamespaceUsdPrice(selectedNamespace)}
                {#if usdPrice}
                  {formatUsdPrice(usdPrice)}
                {:else}
                  {formatFee(selectedNamespace.registration_fee)} {selectedNamespace.fee_currency_name}
                {/if}
              {/if}
            {:else}
              {formatFee(selectedNamespace.registration_fee)} {selectedNamespace.fee_currency_name}
            {/if}
          </span>
        </div>
        
        <!-- Referral System (only for options: 41) -->
        {#if hasReferralSystem}
          <div class="flex justify-between items-center">
            <span class="text-sm text-green-400">Referral ({discountPercentage}% off):</span>
            <span class="text-sm font-mono text-green-400">
              {#if isVerusBlockchain}
                {#if fetchingUsdPrice}
                  <span class="animate-pulse">~$... USD</span>
                {:else}
                  {@const discountedUsd = getDiscountedUsdPrice(selectedNamespace)}
                  {#if discountedUsd}
                    {formatUsdPrice(discountedUsd)}
                  {:else}
                    {formatFee(discountedPrice || 0)} {selectedNamespace.fee_currency_name}
                  {/if}
                {/if}
              {:else}
                {formatFee(discountedPrice || 0)} {selectedNamespace.fee_currency_name}
              {/if}
            </span>
          </div>
          
          <!-- Referral Code Section -->
          <div class="mt-2 pt-2 border-t border-white/5">
            {#if !showReferralInput}
              <button
                type="button"
                on:click={toggleReferralInput}
                class="text-xs text-blue-400 hover:text-blue-300 underline hover:no-underline transition-colors"
              >
                I have a referral code
              </button>
            {:else}
              <div class="space-y-2">
                <label class="block text-xs font-medium text-white/70">Referral ID (optional)</label>
                <div class="flex gap-2 items-start">
                  <div class="flex-1">
                    <div class="flex gap-2 items-center">
                      <input
                        type="text"
                        bind:value={referralCode}
                        on:input={() => { referralStatus = 'unchecked'; referralMessage = null; }}
                        placeholder="name.namespace@"
                        class="flex-1 bg-black/60 border hover:border-blue-400/60 focus:border-blue-400 rounded px-2 py-1.5 text-sm text-white placeholder-white/40 focus:outline-none focus:ring-1 focus:ring-blue-400/30
                          {referralStatus === 'valid' ? 'border-green-500/50' : ''}
                          {referralStatus === 'invalid' || referralStatus === 'error' ? 'border-red-500/50' : 'border-white/20'}"
                        autocomplete="off"
                        autocorrect="off"
                        autocapitalize="off"
                        spellcheck="false"
                        readonly={referralStatus === 'valid' || referralStatus === 'checking'}
                      />

                      {#if referralStatus === 'valid'}
                        <button
                          type="button"
                          on:click={() => { referralCode = ''; referralStatus = 'unchecked'; referralMessage = null; }}
                          class="bg-gray-700/50 hover:bg-gray-600/50 text-white/80 px-2.5 py-1.5 text-xs font-medium rounded transition-colors"
                        >
                          Clear
                        </button>
                      {:else}
                        <Button
                          variant="secondary"
                          size="small"
                          on:click={validateReferral}
                          loading={referralStatus === 'checking'}
                          disabled={!referralCode.trim() || !selectedNamespace}
                        >
                          Apply
                        </Button>
                      {/if}

                    </div>
                    {#if referralMessage}
                      <p class="text-xs mt-1.5
                        {referralStatus === 'valid' ? 'text-green-400/90' : ''}
                        {referralStatus === 'invalid' || referralStatus === 'error' ? 'text-red-400/90' : 'text-white/60'}">
                        {referralMessage}
                      </p>
                    {/if}
                  </div>

                </div>
              </div>
            {/if}
          </div>
        {/if}
        

      </div>
    </div>
  {/if}

 
</div>

<style>
  /* Custom scrollbar for dropdown */
  .max-h-52::-webkit-scrollbar {
    width: 6px;
  }
  
  .max-h-52::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .max-h-52::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 3px;
  }
  
  .max-h-52::-webkit-scrollbar-thumb:hover {
    background: #4b5563;
  }
</style> 