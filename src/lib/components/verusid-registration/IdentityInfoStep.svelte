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
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';
  import type { NamespaceOption } from '$lib/types';

  // Props
  export let name: string = '';
  export let selectedNamespace: NamespaceOption | null = null;

  // State
  let namespaces: NamespaceOption[] = [];
  let fetchingNamespaces = false;
  let fetchError: string | null = null;
  let showDropdown = false;
  let inputElement: HTMLInputElement;
  let dropdownElement: HTMLDivElement;

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    dataChange: { name: string; namespace: NamespaceOption | null; isValid: boolean };
  }>();

  // Computed
  $: previewId = name && selectedNamespace ? `${name}.${selectedNamespace.name}@` : '';
  
  $: isValid = name.trim().length > 0 && selectedNamespace !== null && isValidName(name);

  // Lifecycle
  onMount(() => {
    fetchNamespaces();
    // Close dropdown when clicking outside
    document.addEventListener('click', handleOutsideClick);
    return () => {
      document.removeEventListener('click', handleOutsideClick);
    };
  });

  // Functions
  function isValidName(inputName: string): boolean {
    // Basic validation - alphanumeric and some special chars, no spaces
    const nameRegex = /^[a-zA-Z0-9_-]+$/;
    return nameRegex.test(inputName) && inputName.length >= 3 && inputName.length <= 20;
  }

  async function fetchNamespaces() {
    fetchingNamespaces = true;
    fetchError = null;

    try {
      console.log('Fetching available namespaces...');
      const result = await invoke<NamespaceOption[]>('get_available_namespaces');
      namespaces = result;
      console.log(`Loaded ${namespaces.length} available namespaces`);
    } catch (error: any) {
      console.error('Failed to fetch namespaces:', error);
      fetchError = `Failed to load namespaces: ${error.message || error}`;
    } finally {
      fetchingNamespaces = false;
    }
  }

  function handleNameInput(event: Event) {
    const target = event.target as HTMLInputElement;
    name = target.value;
    dispatchDataChange();
  }

  function handleNamespaceSelect(namespace: NamespaceOption) {
    selectedNamespace = namespace;
    showDropdown = false;
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
      name: name.trim(),
      namespace: selectedNamespace,
      isValid
    });
  }

  function formatFee(fee: number): string {
    return fee.toFixed(8).replace(/\.?0+$/, '');
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
          <input
            bind:this={inputElement}
            type="text"
            bind:value={name}
            on:input={handleNameInput}
            placeholder="Enter name"
            class="w-full bg-black/60 border border-white/20 hover:border-brand-green/60 focus:border-brand-green rounded-lg shadow-lg px-3 py-2.5 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-brand-green/30"
            maxlength="20"
          />
        </div>
        
        <!-- Dot separator -->
        <div class="flex items-center px-1">
          <span class="text-white text-lg font-bold">.</span>
        </div>
        
        <!-- Namespace Dropdown (right side) -->
        <div class="flex-1 relative" bind:this={dropdownElement}>
          {#if fetchingNamespaces}
            <div class="w-full bg-black/60 border border-white/20 rounded-lg px-3 py-2.5 text-white/60">
              Loading...
            </div>
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
              <span class="block truncate font-medium {selectedNamespace ? 'text-white' : 'text-white/40'}">
                {selectedNamespace ? selectedNamespace.name : 'Select namespace'}
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
                <ul 
                  class="bg-black/95 border border-white/10 shadow-2xl max-h-60 rounded-lg py-1 text-sm overflow-auto focus:outline-none backdrop-blur-sm"
                  tabindex="-1"
                  role="listbox"
                >
                  {#each namespaces as namespace}
                    <li 
                      class="relative cursor-pointer select-none px-3 py-2.5 mx-1 rounded-md hover:bg-white/5 text-white border border-transparent hover:border-white/10 {selectedNamespace?.currency_id === namespace.currency_id ? 'bg-brand-green/15 text-brand-green border border-brand-green/30 font-medium' : ''}"
                      role="option"
                      aria-selected={selectedNamespace?.currency_id === namespace.currency_id}
                      on:click={() => handleNamespaceSelect(namespace)}
                    >
                      <div class="flex items-center justify-between">
                        <span class="block truncate">
                          {namespace.name}
                        </span>
                        
                        <div class="flex items-center space-x-2 ml-2 flex-shrink-0">
                          <span class="text-xs text-white/60 font-mono {selectedNamespace?.currency_id === namespace.currency_id ? 'text-brand-green/80' : ''}">
                            {formatFee(namespace.registration_fee)} {namespace.fee_currency_name}
                          </span>
                          
                          {#if selectedNamespace?.currency_id === namespace.currency_id}
                            <span class="text-brand-green">
                              <svg class="h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                              </svg>
                            </span>
                          {/if}
                        </div>
                      </div>
                    </li>
                  {/each}
                </ul>
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
      
      <!-- Validation message -->
      {#if name && !isValidName(name)}
        <p class="mt-1 text-xs text-red-400">
          Name must be 3-20 characters, alphanumeric, underscores, and hyphens only.
        </p>
      {/if}
    </div>
  </div>

  <!-- Compact Preview -->
  {#if previewId}
    <div class="bg-brand-green/10 border border-brand-green/30 rounded-lg p-3">
      <div class="flex items-center justify-between">
        <div class="text-lg font-mono text-white">
          {previewId}
        </div>
        {#if selectedNamespace}
          <div class="text-xs text-brand-green/80">
            Fee: {formatFee(selectedNamespace.registration_fee)} {selectedNamespace.fee_currency_name}
          </div>
        {/if}
      </div>
    </div>
  {/if}

 
</div>

<style>
  /* Custom scrollbar for dropdown */
  .max-h-60::-webkit-scrollbar {
    width: 6px;
  }
  
  .max-h-60::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .max-h-60::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 3px;
  }
  
  .max-h-60::-webkit-scrollbar-thumb:hover {
    background: #4b5563;
  }
</style> 