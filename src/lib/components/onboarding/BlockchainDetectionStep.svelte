<script lang="ts">
// Component: src/lib/components/onboarding/BlockchainDetectionStep.svelte
// Description: Automatically detects available blockchains and allows user to select one.
// Handles parallel detection of blockchain configurations and daemon connections.
// Changes:
// - Added automatic credential saving immediately after successful blockchain selection
// - This ensures credentials are available in the store when the VerusID step tries to load them
// - Prevents "Credentials not found in store" errors during onboarding
// - Refactored manual folder selection into separate NoBlockchainFoundStep component
// - Added state management for switching between detection and no-blockchain-found views

    import { createEventDispatcher, onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { slide, fade } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    
    // Import Lucide Icons
    import { 
        CheckCircle, 
        Loader, 
        XCircle, 
        Clock, 
        AlertTriangle, 
        Search,
        ChevronRight
    } from 'lucide-svelte';
    
    // Import Components
    import NoBlockchainFoundStep from './NoBlockchainFoundStep.svelte';
    
    // Import Store
    import { setSelectedBlockchain } from '$lib/stores/blockchain';
    
    // Import Types
    import type { 
        ParallelDetectionResult, 
        BlockchainDetectionResult, 
        BlockchainStatus,
        Credentials 
    } from '$lib/types';

    // --- Props ---
    export let selectedBlockchainId: string | null = null;

    // --- State ---
    let detectionState: 'loading' | 'completed' | 'error' = 'loading';
    let detectionResults: BlockchainDetectionResult[] = [];
    let detectionError: string | null = null;
    let detectionDuration: number = 0;
    let showNoBlockchainFoundStep = false;
    let refreshInterval: number | null = null;

    // --- Event Dispatcher ---
    const dispatch = createEventDispatcher<{
        blockchainSelected: { 
            blockchainId: string; 
            credentials: Credentials; 
            blockHeight: number;
        };
        detectionCompleted: { availableCount: number };
    }>();

    // --- Lifecycle ---
    onMount(() => {
        startDetection();
        // Set up auto-refresh every minute
        refreshInterval = setInterval(() => {
            if (detectionState === 'completed') {
                console.log('BlockchainDetectionStep: Auto-refreshing detection...');
                startDetection();
            }
        }, 60000); // 60 seconds
    });

    onDestroy(() => {
        if (refreshInterval) {
            clearInterval(refreshInterval);
        }
    });

    // --- Reactive Statements ---
    // Filter chains to only show Available, Loading, and some Error types
    $: filteredResults = detectionResults.filter(chain => {
        // Always show Available and Loading chains
        if (chain.status === 'Available' || chain.status === 'Loading') {
            return true;
        }
        
        // Hide NotFound chains
        if (chain.status === 'NotFound') {
            return false;
        }
        
        // Hide HTTP request failed errors
        if (chain.status === 'Error' && chain.error_message?.includes('HTTP request failed')) {
            return false;
        }
        
        // Show other error types
        return true;
    });

    // Show no blockchain found step if detection is complete but no detected blockchains (Available OR Loading)
    $: if (detectionState === 'completed') {
        const detectedCount = detectionResults.filter(r => r.status === 'Available' || r.status === 'Loading').length;
        showNoBlockchainFoundStep = detectedCount === 0;
    }

    // --- Detection Logic ---
    async function startDetection() {
        console.log('BlockchainDetectionStep: Starting automatic detection...');
        detectionState = 'loading';
        detectionError = null;
        showNoBlockchainFoundStep = false;

        try {
            const result: ParallelDetectionResult = await invoke('detect_all_blockchains');
            
            detectionResults = result.blockchains;
            detectionDuration = result.detection_duration_ms;
            detectionState = 'completed';
            
            console.log(`BlockchainDetectionStep: Detection completed. ${result.total_detected} available blockchains found in ${result.detection_duration_ms}ms`);

            // Dispatch completion event
            dispatch('detectionCompleted', { availableCount: result.total_detected });

        } catch (error) {
            console.error('BlockchainDetectionStep: Detection failed:', error);
            detectionState = 'error';
            detectionError = String(error);
        }
    }

    // --- Event Handlers for NoBlockchainFoundStep ---
    function handleBlockchainsFound(event: CustomEvent<{ results: ParallelDetectionResult }>) {
        console.log('BlockchainDetectionStep: Received blockchains from NoBlockchainFoundStep');
        const result = event.detail.results;
        
        // Merge with existing results, replacing any with same ID
        const newAvailableChains = result.blockchains.filter(b => b.status === 'Available');
        
        for (const newChain of newAvailableChains) {
            const existingIndex = detectionResults.findIndex(r => r.blockchain_id === newChain.blockchain_id);
            if (existingIndex >= 0) {
                detectionResults[existingIndex] = newChain;
            } else {
                detectionResults.push(newChain);
            }
        }
        
        // Sort results by the intended order
        const order = ["verus", "chips", "vdex", "varrr", "verus-testnet"];
        detectionResults.sort((a, b) => {
            const aIndex = order.indexOf(a.blockchain_id);
            const bIndex = order.indexOf(b.blockchain_id);
            return (aIndex === -1 ? 999 : aIndex) - (bIndex === -1 ? 999 : bIndex);
        });
        
        detectionState = 'completed';
        showNoBlockchainFoundStep = false;
        
        const totalAvailable = detectionResults.filter(r => r.status === 'Available').length;
        dispatch('detectionCompleted', { availableCount: totalAvailable });
    }

    function handleNoBlockchainError(event: CustomEvent<{ message: string }>) {
        console.error('BlockchainDetectionStep: Error from NoBlockchainFoundStep:', event.detail.message);
        detectionError = event.detail.message;
    }

    async function selectBlockchain(blockchain: BlockchainDetectionResult) {
        // Only allow selection of Available blockchains with complete data
        if (blockchain.status !== 'Available') {
            console.error(`BlockchainDetectionStep: Cannot select blockchain - status is ${blockchain.status}, not Available`);
            return;
        }
        
        if (!blockchain.credentials || blockchain.block_height === null) {
            console.error('BlockchainDetectionStep: Cannot select blockchain - missing credentials or block height');
            return;
        }

        console.log(`BlockchainDetectionStep: Blockchain selected: ${blockchain.blockchain_name}`);
        selectedBlockchainId = blockchain.blockchain_id;

        try {
            // Save credentials immediately after successful selection
            await invoke('save_credentials', {
                rpcUser: blockchain.credentials.rpc_user,
                rpcPass: blockchain.credentials.rpc_pass,
                rpcPort: blockchain.credentials.rpc_port
            });
            console.log('BlockchainDetectionStep: Credentials saved successfully after blockchain selection.');
        } catch (saveError) {
            console.error('BlockchainDetectionStep: Failed to save credentials after blockchain selection:', saveError);
            // Don't block the flow, but log the error - credentials will still be passed via event
        }

        // Update the blockchain store
        setSelectedBlockchain({
            id: blockchain.blockchain_id,
            name: blockchain.blockchain_name,
            credentials: blockchain.credentials,
            blockHeight: blockchain.block_height
        });

        // Dispatch selection event with credentials (keep for compatibility)
        dispatch('blockchainSelected', {
            blockchainId: blockchain.blockchain_id,
            credentials: blockchain.credentials,
            blockHeight: blockchain.block_height
        });
    }

    function getStatusColor(status: BlockchainStatus, isSelected: boolean = false): string {
        switch (status) {
            case 'Available': 
                return isSelected 
                    ? 'border-brand-green bg-brand-green/10 hover:bg-brand-green/20' 
                    : 'border-dark-border-primary bg-dark-bg-primary hover:bg-dark-bg-secondary hover:border-brand-green/40';
            case 'Loading': return 'border-blue-500/40 bg-blue-800/20';
            case 'Error': return 'border-red-600/40 bg-red-800/20';
            case 'Timeout': return 'border-yellow-600/40 bg-yellow-800/20';
            case 'ParseError': return 'border-orange-600/40 bg-orange-800/20';
            case 'NotFound': return 'border-dark-border-secondary bg-dark-bg-tertiary/50';
            default: return 'border-dark-border-secondary bg-dark-bg-tertiary';
        }
    }

    function getStatusText(status: BlockchainStatus): string {
        switch (status) {
            case 'Available': return 'Ready to connect';
            case 'Loading': return 'Starting up - please wait...';
            case 'Error': return 'Connection failed';
            case 'Timeout': return 'Daemon not responding';
            case 'ParseError': return 'Configuration error';
            case 'NotFound': return 'Not found';
            default: return 'Unknown status';
        }
    }
</script>

<!-- Blockchain Detection Step Content -->
<div class="step-content-area">
    {#if showNoBlockchainFoundStep}
        <!-- No Blockchain Found Step -->
        <NoBlockchainFoundStep 
            on:blockchainsFound={handleBlockchainsFound}
            on:error={handleNoBlockchainError}
        />
    {:else}
        <!-- Main Detection Content -->
        <h1 class="text-2xl font-semibold text-dark-text-primary mb-2 select-none cursor-default">Connect to Blockchain</h1>
        <p class="text-dark-text-secondary text-normal mb-6 select-none cursor-default">
            {#if detectionState === 'loading'}
                Detecting available blockchain daemons...
            {:else if detectionState === 'completed'}
                {#if filteredResults.length > 0}
                    Select a blockchain to connect to:
                {:else}
                    No running blockchain daemons detected.
                {/if}
            {:else}
                Detection failed. Please try again.
            {/if}
        </p>
        
        <!-- Detection Results / Loading States -->
        <div class="mb-6">
            {#if detectionState === 'loading'}
                <!-- Skeleton Loading States -->
                <div class="space-y-3">
                    {#each Array(4) as _, i}
                        <div class="blockchain-item skeleton" transition:fade={{ delay: i * 100, duration: 200 }}>
                            <div class="flex items-center space-x-4">
                                <div class="w-8 h-8 bg-dark-bg-tertiary rounded-full animate-pulse flex-shrink-0"></div>
                                <div class="flex-1 space-y-2">
                                    <div class="w-32 h-4 bg-dark-bg-tertiary rounded animate-pulse"></div>
                                    <div class="w-24 h-3 bg-dark-bg-tertiary rounded animate-pulse"></div>
                                </div>
                                <div class="w-6 h-6 bg-dark-bg-tertiary rounded animate-pulse flex-shrink-0"></div>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <!-- Detection Results -->
                <div class="space-y-2">
                    {#each filteredResults as blockchain, i (blockchain.blockchain_id)}
                        <button
                            type="button"
                            on:click={() => selectBlockchain(blockchain)}
                            disabled={blockchain.status !== 'Available'}
                            class="blockchain-item {getStatusColor(blockchain.status, blockchain.blockchain_id === selectedBlockchainId)} {blockchain.blockchain_id === selectedBlockchainId ? 'ring-2 ring-brand-green' : ''} {blockchain.status === 'Available' ? 'cursor-pointer' : 'cursor-not-allowed opacity-75'}"
                            transition:slide={{ delay: i * 50, duration: 250, easing: quintOut }}
                        >
                            <div class="flex items-center space-x-4">
                                <!-- Status Icon -->
                                <div class="flex-shrink-0 w-8 h-8 flex items-center justify-center">
                                    {#if blockchain.status === 'Available'}
                                        <CheckCircle class="w-6 h-6 text-brand-green" />
                                    {:else if blockchain.status === 'Loading'}
                                        <Loader class="w-6 h-6 text-blue-400 animate-spin" />
                                    {:else if blockchain.status === 'Error'}
                                        <XCircle class="w-6 h-6 text-red-400" />
                                    {:else if blockchain.status === 'Timeout'}
                                        <Clock class="w-6 h-6 text-yellow-400" />
                                    {:else if blockchain.status === 'ParseError'}
                                        <AlertTriangle class="w-6 h-6 text-orange-400" />
                                    {:else if blockchain.status === 'NotFound'}
                                        <Search class="w-6 h-6 text-dark-text-tertiary" />
                                    {:else}
                                        <AlertTriangle class="w-6 h-6 text-dark-text-tertiary" />
                                    {/if}
                                </div>
                                
                                <!-- Blockchain Info -->
                                <div class="flex-1 text-left min-w-0">
                                    <div class="font-semibold text-dark-text-primary text-base truncate">
                                        {blockchain.blockchain_name}
                                    </div>
                                    <div class="text-sm text-dark-text-secondary flex items-center space-x-2">
                                        <span>
                                            {#if blockchain.status === 'Loading' && blockchain.error_message}
                                                {blockchain.error_message}
                                            {:else}
                                                {getStatusText(blockchain.status)}
                                            {/if}
                                        </span>
                                        {#if blockchain.block_height !== null}
                                            <span class="text-dark-text-tertiary">•</span>
                                            <span class="font-mono text-xs">
                                                Block {blockchain.block_height.toLocaleString()}
                                            </span>
                                        {/if}
                                    </div>
                                    {#if blockchain.error_message && blockchain.status !== 'Loading' && blockchain.status !== 'NotFound'}
                                        <div class="text-xs text-red-300 mt-1 truncate">
                                            {blockchain.error_message}
                                        </div>
                                    {/if}
                                </div>
                                
                                <!-- Selection Indicator (only for non-Available statuses) -->
                                <div class="flex-shrink-0">
                                    {#if blockchain.status !== 'Available'}
                                        <!-- Could add indicators for other statuses if needed -->
                                    {/if}
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Error Display -->
        {#if detectionError}
            <div class="mt-4 bg-red-800/30 border border-red-600/40 rounded-lg p-4" transition:slide={{ duration: 200 }}>
                <div class="flex items-start space-x-3">
                    <XCircle class="w-5 h-5 text-red-300 flex-shrink-0 mt-0.5" />
                    <p class="text-sm text-red-300 flex-1">{detectionError}</p>
                </div>
            </div>
        {/if}
    {/if}
</div>

<style>
    .step-content-area {
        width: 100%;
    }

    .blockchain-item {
        @apply w-full p-4 border-2 rounded-lg;
        min-height: 72px;
    }

    .blockchain-item.skeleton {
        @apply border-dark-border-secondary bg-dark-bg-tertiary/30;
    }



    .blockchain-item:not(.skeleton):focus {
        outline: none;
        ring-width: 2px;
        ring-color: rgb(34 197 94 / 0.5);
    }
</style> 