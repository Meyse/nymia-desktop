<script lang="ts">
// Component: src/lib/components/onboarding/NoBlockchainFoundStep.svelte
// Description: Side-step component shown when no running blockchain daemons are detected.
// Provides options to retry detection or browse for custom configuration folder.
// Changes:
// - New component extracted from BlockchainDetectionStep manual selection logic
// - Clear messaging about two possible scenarios (start daemon vs custom config)
// - Retry and browse functionality with proper event dispatching

    import { createEventDispatcher } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { slide } from 'svelte/transition';
    
    // Import Lucide Icons
    import { 
        FolderOpen,
        RefreshCw,
        Loader,
        AlertCircle,
        Play,
        Settings
    } from 'lucide-svelte';
    
    // Import Components
    import Button from '../Button.svelte';
    
    // Import Types
    import type { ParallelDetectionResult } from '$lib/types';

    // --- State ---
    let isRetrying = false;
    let isBrowsing = false;
    let errorMessage: string | null = null;

    // --- Event Dispatcher ---
    const dispatch = createEventDispatcher<{
        retryDetection: void;
        blockchainsFound: { results: ParallelDetectionResult };
        error: { message: string };
    }>();

    // --- Actions ---
    async function retryDetection() {
        console.log('NoBlockchainFoundStep: Retrying blockchain detection...');
        isRetrying = true;
        errorMessage = null;

        try {
            const result: ParallelDetectionResult = await invoke('detect_all_blockchains');
            
            // Check for any detected blockchains (Available OR Loading states)
            const detectedChains = result.blockchains.filter(b => 
                b.status === 'Available' || b.status === 'Loading'
            );
            
            if (detectedChains.length > 0) {
                console.log(`NoBlockchainFoundStep: Retry successful - found ${detectedChains.length} blockchain(s) (Available: ${result.total_detected}, Loading: ${detectedChains.length - result.total_detected})`);
                dispatch('blockchainsFound', { results: result });
            } else {
                console.log('NoBlockchainFoundStep: Retry completed but still no blockchains found (Available or Loading)');
                // No error message needed - user stays on current screen as expected
            }
        } catch (error) {
            console.error('NoBlockchainFoundStep: Retry detection failed:', error);
            errorMessage = `Detection failed: ${String(error)}`;
            dispatch('error', { message: String(error) });
        } finally {
            isRetrying = false;
        }
    }

    async function browseForFolder() {
        console.log('NoBlockchainFoundStep: Opening folder selection dialog...');
        isBrowsing = true;
        errorMessage = null;

        try {
            const selectedPath: string | null = await invoke('select_folder_dialog');
            
            if (selectedPath) {
                console.log('NoBlockchainFoundStep: User selected folder:', selectedPath);
                
                // Detect blockchains from the custom path
                const result: ParallelDetectionResult = await invoke('detect_blockchain_from_path', {
                    path: selectedPath
                });

                const availableChains = result.blockchains.filter(b => b.status === 'Available');
                
                if (availableChains.length > 0) {
                    console.log(`NoBlockchainFoundStep: Found ${availableChains.length} blockchain(s) in custom folder`);
                    dispatch('blockchainsFound', { results: result });
                } else {
                    console.log('NoBlockchainFoundStep: No valid blockchains found in selected folder');
                    errorMessage = 'No valid blockchain configuration files found in the selected folder. Please ensure your .conf files are in the selected directory.';
                }
            } else {
                console.log('NoBlockchainFoundStep: User cancelled folder selection');
            }
        } catch (error) {
            console.error('NoBlockchainFoundStep: Folder browsing failed:', error);
            errorMessage = `Failed to read configurations from selected folder: ${String(error)}`;
            dispatch('error', { message: String(error) });
        } finally {
            isBrowsing = false;
        }
    }
</script>

<div class="step-content-area">
    <h1 class="text-2xl font-semibold text-dark-text-primary mb-12 select-none cursor-default">No Running Blockchain Found</h1>
   

    <!-- Scenarios with integrated actions -->
    <div class="space-y-12">
        <!-- Scenario 1: Start Blockchain -->
        <div>
            <h3 class="text-base font-medium text-dark-text-primary mb-2 select-none cursor-default">Start Your Blockchain</h3>
            <p class="text-xs text-dark-text-secondary mb-4 leading-relaxed select-none cursor-default">
                Your blockchain daemon might not be running yet. Make sure to start your node before connecting Nymia.
            </p>
                         <Button
                 variant="secondary"
                 iconComponent={RefreshCw}
                 loading={isRetrying}
                 loadingText="Detecting..."
                 disabled={isBrowsing}
                 on:click={retryDetection}
             >
                 Retry to Connect
             </Button>
        </div>

        <!-- Scenario 2: Custom Configuration -->
        <div>
            <h3 class="text-base font-medium text-dark-text-primary mb-2 select-none cursor-default">Custom Configuration</h3>
            <p class="text-xs text-dark-text-secondary mb-4 leading-relaxed select-none cursor-default">
                Your blockchain is running but the configuration files are in a custom location that Nymia couldn't detect automatically.
            </p>
                         <Button
                 variant="secondary"
                 iconComponent={FolderOpen}
                 loading={isBrowsing}
                 loadingText="Checking folder..."
                 disabled={isRetrying}
                 on:click={browseForFolder}
             >
                 Browse for Folder
             </Button>
        </div>
    </div>

    <!-- Error Display -->
    {#if errorMessage}
        <div class="mt-6 bg-red-800/30 border border-red-600/40 rounded-lg p-4" transition:slide={{ duration: 200 }}>
            <div class="flex items-start space-x-3">
                <AlertCircle class="w-5 h-5 text-red-300 flex-shrink-0 mt-0.5" />
                <p class="text-sm text-red-300 flex-1">{errorMessage}</p>
            </div>
        </div>
    {/if}
</div>

<style>
    .step-content-area {
        width: 100%;
    }
</style> 