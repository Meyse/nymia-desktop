<script lang="ts">
// Component: src/lib/components/onboarding/VerusIdStep.svelte
// Description: Handles fetching and selecting a VerusID with progressive balance loading and improved UX.
// Changes:
// - Implemented progressive loading: skeleton → names → balances
// - Uses get_login_identities_fast for immediate name loading
// - Sorts identities alphabetically IMMEDIATELY after fetching names (before balance loading)
// - Fetches individual balances using get_identity_balance in parallel
// - KEEPS all valid VerusIDs even if balance fetch fails (shows "-" instead of hiding)
// - Shows skeleton loading during name fetch and balance loading states
// - Enhanced UX with smooth loading transitions and real-time updates
// - Added currency symbol support - balances now display with proper ticker (e.g., "12.5 VRSC")
// - Added "Get VerusID" button underneath dropdown to open registration modal
// - Integrated VerusIdRegistrationModal for new VerusID creation flow
// - Made "Get VerusID" button always visible immediately, regardless of fetch status
// - Removed all credential checking logic since credentials are guaranteed by BlockchainDetectionStep
// - Simplified error handling to focus on VerusID-specific issues only
// - Auto-select newly created VerusID after registration completion
// - Fixed: No longer filters out identities due to temporary balance fetch failures

    import { createEventDispatcher, onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import CustomDropdown from '../CustomDropdown.svelte';
    import Button from '../Button.svelte';
    import VerusIdRegistrationModal from '../verusid-registration/VerusIdRegistrationModal.svelte';
    import VerusIdInfoModal from './VerusIdInfoModal.svelte';

    // Import Shared Types
    import type { FormattedIdentity, DropdownOption } from '$lib/types';

    // --- Types --- (Removed local definitions)
    type FetchStatus = 'idle' | 'fetching' | 'success' | 'error';
    type BalanceLoadingStatus = 'loading' | 'loaded' | 'error';

    // --- Props ---
    export let currencySymbol: string = 'VRSC'; // Currency symbol for balance display

    // --- State ---
    let loginIdentities: FormattedIdentity[] = [];
    let loginIdentityOptions: DropdownOption[] = []; 
    let selectedIdentityIAddress: string | null = null; // Keep this for dropdown binding
    let fetchStatus: FetchStatus = 'idle';
    let fetchError: string | null = null;
    let balanceLoadingStatus: Map<string, BalanceLoadingStatus> = new Map(); // Track balance loading per identity
    let showingSkeleton = false;
    let showRegistrationModal = false;
    let showVerusIdInfoModal = false;

    // --- Event Dispatcher ---
    const dispatch = createEventDispatcher<{
        idSelected: { identity: FormattedIdentity | null }; // Changed event payload
    }>();

    // --- Lifecycle ---
    onMount(() => {
        // Fetch identities immediately when component mounts (credentials guaranteed by BlockchainDetectionStep)
        fetchIdentities();
    });

    // --- Logic ---
    function formatBalance(balance: number | null, iAddress: string): string | null {
        const loadingStatus = balanceLoadingStatus.get(iAddress);
        
        if (loadingStatus === 'loading') {
            return 'skeleton'; // Special value for skeleton display
        }
        
        if (balance === null || loadingStatus === 'error') {
            return null; // Will show "-" in UI
        }
        
        const formattedAmount = balance.toFixed(5).replace(/\.?0+$/, ''); // Remove trailing zeros after decimal
        return `${formattedAmount} ${currencySymbol}`;
    }

    async function fetchIdentities(): Promise<void> {
        // Step 1: Show skeleton loading initially
        fetchStatus = 'fetching';
        fetchError = null;
        showingSkeleton = true;
        loginIdentities = [];
        selectedIdentityIAddress = null;
        balanceLoadingStatus = new Map();
        dispatch('idSelected', { identity: null });
        
        // Show skeleton dropdown options initially
        loginIdentityOptions = Array.from({ length: 3 }, (_, i) => ({
            id: `skeleton-${i}`,
            name: 'Loading...', 
            enabled: false,
            balance: 'skeleton'
        })); 

        try {
            console.log("VerusIdStep: Fetching login identities (fast mode)...");
            
            // Step 2: Get identities without balances (fast)
            const ids = await invoke<FormattedIdentity[]>('get_login_identities_fast'); 
            
            if (ids.length === 0) {
                fetchError = "No eligible VerusIDs found. Identities must have private addresses and spending/signing permissions.";
                fetchStatus = 'error';
                showingSkeleton = false;
                dispatch('idSelected', { identity: null });
                return;
            }

            // Step 3: Sort identities alphabetically immediately
            ids.sort((a, b) => a.formatted_name.localeCompare(b.formatted_name));
            loginIdentities = ids;

            // Step 4: Show all identities in alphabetical order with skeleton balances
            ids.forEach(id => {
                balanceLoadingStatus.set(id.i_address, 'loading');
            });
            
            loginIdentityOptions = ids.map(id => ({ 
                id: id.i_address, 
                name: id.formatted_name, 
                enabled: true,
                balance: formatBalance(id.balance, id.i_address) // Will return 'skeleton'
            }));
            
            fetchStatus = 'success';
            showingSkeleton = false;
            
            console.log("VerusIdStep: Names loaded, now fetching balances...");
            
            // Step 4: Fetch balances progressively
            await fetchBalancesProgressively(ids);
            
        } catch (error: any) {
            console.error("VerusIdStep: Failed to fetch login identities:", error);
            fetchStatus = 'error';
            showingSkeleton = false;
            
            // Better error message handling with user-friendly messages
            let errorMessage = 'Unknown error occurred';
            if (error && typeof error === 'object') {
                if (error.message) {
                    errorMessage = error.message;
                } else if (error.error) {
                    errorMessage = error.error;
                } else {
                    errorMessage = JSON.stringify(error);
                }
            } else {
                errorMessage = String(error);
            }
            
            // Handle VerusID-specific errors
            if (errorMessage.includes('No eligible VerusIDs found') || errorMessage.includes('No VerusIDs with private addresses found')) {
                fetchError = 'No eligible VerusIDs found. Identities must have private addresses and spending/signing permissions.';
            } else {
                fetchError = `Unable to load identities: ${errorMessage}`;
            }
            
            dispatch('idSelected', { identity: null }); // Ensure parent knows none are selected on error
        }
    }

    async function fetchBalancesProgressively(identities: FormattedIdentity[]) {
        console.log(`VerusIdStep: Fetching balances for ${identities.length} identities...`);
        
        // Fetch all balances in parallel
        const balancePromises = identities.map(async (identity) => {
            try {
                console.log(`Fetching balance for ${identity.formatted_name}...`);
                const balance = await invoke<number>('get_identity_balance', { 
                    privateAddress: identity.private_address 
                });
                
                // Update balance loading status
                balanceLoadingStatus.set(identity.i_address, 'loaded');
                
                return { identity: { ...identity, balance }, success: true };
            } catch (error) {
                console.error(`Failed to fetch balance for ${identity.formatted_name}:`, error);
                console.warn(`Balance fetch failed for ${identity.formatted_name} - will show as unavailable`);
                
                // Update balance loading status to error (will show "-" in UI)
                balanceLoadingStatus.set(identity.i_address, 'error');
                
                return { identity: { ...identity, balance: null }, success: false };
            }
        });

        // Wait for all balances to complete
        const results = await Promise.allSettled(balancePromises);
        
        // Update all identities with their balance results (keep ALL identities)
        const updatedIdentities: FormattedIdentity[] = [];
        for (const result of results) {
            if (result.status === 'fulfilled') {
                updatedIdentities.push(result.value.identity);
            } else {
                // If promise was rejected, keep the original identity with null balance
                console.error('Balance fetch promise rejected:', result.reason);
            }
        }
        
        // Update loginIdentities with balance information (keep in same alphabetical order)
        loginIdentities = updatedIdentities;
        
        // Update dropdown options with new balance information (keep same order)
        loginIdentityOptions = loginIdentities.map(id => ({ 
            id: id.i_address, 
            name: id.formatted_name, 
            enabled: true,
            balance: formatBalance(id.balance, id.i_address)
        }));
        
        console.log(`VerusIdStep: Balance loading complete for ${loginIdentities.length} identities`);
    }

    function handleIdSelection(event: CustomEvent<string | number | null>) {
        const newIAddress = typeof event.detail === 'string' ? event.detail : null;
        selectedIdentityIAddress = newIAddress; // Keep local state for dropdown sync
        
        // Find the full identity object based on the selected iAddress
        const selectedFullIdentity = newIAddress
            ? loginIdentities.find(id => id.i_address === newIAddress) || null
            : null;

        console.log("VerusIdStep: Dispatching idSelected with:", selectedFullIdentity);
        // Dispatch the full identity object (or null if none selected)
        dispatch('idSelected', { identity: selectedFullIdentity }); 
    }

    function handleGetVerusId() {
        showRegistrationModal = true;
    }

    function handleCloseRegistrationModal(event: CustomEvent<{ refresh?: boolean; newIdentityName?: string | null }>) {
        showRegistrationModal = false;
        // Refresh identities if a new one was created
        if (event?.detail?.refresh) {
            console.log('VerusIdStep: Refreshing identities after registration completion');
            const newIdentityName = event?.detail?.newIdentityName;
            
            // Refresh identities and then auto-select the new one
            fetchIdentities().then(() => {
                if (newIdentityName) {
                    autoSelectNewIdentity(newIdentityName);
                }
            });
        }
    }

    function autoSelectNewIdentity(identityName: string) {
        console.log('VerusIdStep: Auto-selecting new identity:', identityName);
        
        // Find the identity by formatted name
        const newIdentity = loginIdentities.find(id => id.formatted_name === identityName);
        
        if (newIdentity) {
            selectedIdentityIAddress = newIdentity.i_address;
            dispatch('idSelected', { identity: newIdentity });
            console.log('VerusIdStep: Auto-selected new identity:', newIdentity.formatted_name);
        } else {
            console.warn('VerusIdStep: Could not find new identity to auto-select:', identityName);
        }
    }

    function handleShowVerusIdInfo() {
        showVerusIdInfoModal = true;
    }

    function handleCloseVerusIdInfoModal() {
        showVerusIdInfoModal = false;
    }

</script>

<div class="step-content-area">
    <div class="flex flex-col h-full">
        <!-- Main Content Area (will grow to fill space) -->
        <div class="flex-grow">
            <h1 class="text-2xl font-semibold text-dark-text-primary mb-2 select-none cursor-default">Select VerusID</h1>
            <p class="text-dark-text-secondary text-normal mb-1 select-none cursor-default">Choose the VerusID you want to log in with.</p>
            
            <div class="flex items-center bg-blue-900/30 border-blue-700/50 rounded-md px-3 py-2 mb-6 border-l-2">
                <svg class="w-4 h-4 text-blue-400 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path>
                </svg>
                <span class="text-xs text-blue-300 select-none cursor-default">Only identities with private addresses work with Nymia</span>
            </div>

            {#if fetchStatus === 'error' && fetchError}
                <div class="mt-4 p-3 bg-red-900/40 border border-red-700/50 rounded-md text-center">
                    <p class="text-sm font-medium text-red-300 select-none cursor-default">Error Loading Identities</p>
                    <p class="text-xs text-red-400 select-none cursor-default">{fetchError}</p>
                    <button
                        type="button"
                        on:click={() => {
                            // Retry fetching identities
                            fetchIdentities();
                        }}
                        class="mt-3 inline-flex items-center px-3 py-1 border border-red-600/50 rounded text-xs font-medium text-red-200 bg-red-800/30 hover:bg-red-700/40 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors duration-150"
                    >
                        <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                        </svg>
                        Retry
                    </button>
                </div>
            {:else if fetchStatus === 'fetching' || (fetchStatus === 'success' && loginIdentities.length > 0)}
                <CustomDropdown
                    label="Select Identity"
                    options={loginIdentityOptions}
                    bind:selectedId={selectedIdentityIAddress}
                    placeholder="-- Please choose an ID --"
                    on:change={handleIdSelection} 
                />
            {:else if fetchStatus === 'success' && loginIdentities.length === 0}
                <div class="mt-4 p-3 bg-yellow-900/40 border border-yellow-700/50 rounded-md text-center">
                    <p class="text-sm font-medium text-yellow-300 select-none cursor-default">No Eligible IDs Found</p>
                    <p class="text-xs text-yellow-400 select-none cursor-default">No VerusIDs with private addresses and required permissions were found in your wallet.</p>
                </div>
            {/if}
        </div>

        <!-- Footer Area (pushed to bottom) -->
        <div class="flex-shrink-0">
            <!-- Get VerusID Section - Positioned under dropdown, above footer icons -->
            <div class="mt-8 pt-6 border-t border-white/10">
                <div class="space-y-3">
                    <div class="flex justify-start">
                        <Button
                            variant="white"
                            on:click={handleGetVerusId}
                        >
                            <img slot="icon" src="/verusid-icon.svg" alt="VerusID" class="w-4 h-4" />
                            Get VerusID
                        </Button>
                    </div>
                    <p class="text-sm text-white/55 select-none cursor-default">
                        New to VerusID? 
                        <button 
                            class="text-white/85 hover:text-white/95 cursor-pointer transition-colors duration-150"
                            on:click={handleShowVerusIdInfo}
                        >
                            Learn what it is.
                        </button>
                    </p>
                </div>
            </div>
        </div>
    </div>
</div>

<!-- VerusID Registration Modal -->
<VerusIdRegistrationModal 
    bind:show={showRegistrationModal}
    on:close={handleCloseRegistrationModal}
/>

<!-- VerusID Info Modal -->
{#if showVerusIdInfoModal}
    <VerusIdInfoModal 
        on:close={handleCloseVerusIdInfoModal}
    />
{/if}

<style>
 .step-content-area {
        width: 100%;
        height: 100%;
    }
</style> 