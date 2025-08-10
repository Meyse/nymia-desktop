<script lang="ts">
// Component: src/lib/components/settings/SettingsView.svelte
// Description: Displays application settings, replacing the ConversationView.
// Changes:
// - Created component with Persistence Toggle and Delete History button.
// - Improved visual design with more compact UI.
// - Enhanced iconography and layout.
// - Made text consistent with PersistencePromptModal.
// - Added better visual hierarchy.
// - Added private address display section with copy functionality at the top.
// - Integrated Button component for consistent copy action styling.

    import { createEventDispatcher, onMount, onDestroy } from 'svelte';
    import { Trash2, Database, RotateCcw, ArrowLeft, HardDriveDownload, AlertCircle, MessageSquare, Save, Copy, Check, Wallet } from 'lucide-svelte';
    import ConfirmDeleteModal from './ConfirmDeleteModal.svelte';
    import Button from '../Button.svelte';

    // --- Props ---
    export let currentPersistenceSetting: boolean | null = null; // The current value from parent
    export let privateAddress: string = ''; // NEW: Private address for display and copying

    // --- State ---
    let showConfirmDelete = false;
    let internalPersistenceEnabled: boolean = false;
    let copySuccess = false; // NEW: Copy success state
    let copyTimeout: ReturnType<typeof setTimeout> | null = null; // NEW: Copy timeout

    // --- Events ---
    const dispatch = createEventDispatcher<{ 
        togglePersistence: { enabled: boolean }; // Inform parent of toggle change
        deleteHistory: void; // Request parent to delete history
        closeSettings: void; // Request parent to close the settings view
    }>();

    // --- Lifecycle & Reactivity ---
    // Sync internal state when prop changes
    $: if (currentPersistenceSetting !== null) {
        internalPersistenceEnabled = currentPersistenceSetting;
    }

    // Handle Escape key press to close settings
    function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
            // Close settings view if no modal is open
            if (!showConfirmDelete) {
			    dispatch('closeSettings');
            }
		}
	}

    onMount(() => {
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
        // Cleanup copy timeout
        if (copyTimeout) {
            clearTimeout(copyTimeout);
        }
	});

    // --- Functions ---
    function handleToggleChange() {
        // Update internal state immediately for visual feedback
        internalPersistenceEnabled = !internalPersistenceEnabled;
        // Dispatch the new value to the parent to handle saving
        dispatch('togglePersistence', { enabled: internalPersistenceEnabled });
    }

    function openConfirmDeleteModal() {
        showConfirmDelete = true;
    }

    function handleConfirmDelete() {
        showConfirmDelete = false;
        dispatch('deleteHistory');
    }

    function handleCancelDelete() {
        showConfirmDelete = false;
    }

    function handleGoBack() {
        dispatch('closeSettings');
    }

    // NEW: Handle copying private address
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

</script>

<div class="flex flex-col h-full">
    <!-- Settings Header -->
    <div class="flex items-center h-[50px] px-3 bg-dark-bg-secondary border-b border-dark-border-primary flex-shrink-0 shadow-sm">
        <button 
            on:click={handleGoBack}
            class="p-1.5 rounded text-dark-text-secondary hover:bg-dark-bg-tertiary hover:text-dark-text-primary focus:outline-none focus:ring-1 focus:ring-dark-border-secondary transition-colors duration-150 mr-2"
            title="Back to Chat"
        >
            <ArrowLeft size={16} />
        </button>
        <span class="font-medium text-base text-dark-text-primary">Settings</span>
    </div>

    <!-- Settings Content -->
    <div class="flex-grow p-3 space-y-4 overflow-y-auto">

        <!-- Private Address Card -->
        <div class="bg-dark-bg-secondary rounded-lg border border-dark-border-primary shadow-sm overflow-hidden">
            <div class="border-b border-dark-border-primary bg-dark-bg-primary px-3 py-2 flex items-center">
                <Wallet size={14} class="text-green-400 mr-2" />
                <h3 class="text-sm font-medium text-dark-text-primary">Your Private Address</h3>
            </div>
            
            <div class="p-3">
                <div class="space-y-2">
                    <p class="text-xs text-dark-text-secondary">
                        Send funds to this address to enable sending chat messages
                    </p>
                    
                    {#if privateAddress}
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
                    {:else}
                        <div class="p-3 bg-dark-bg-primary border border-dark-border-primary rounded-md">
                            <p class="text-xs text-dark-text-disabled italic">
                                Private address not available
                            </p>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <!-- Persistence Setting Card -->
        <div class="bg-dark-bg-secondary rounded-lg border border-dark-border-primary shadow-sm overflow-hidden">
            <div class="border-b border-dark-border-primary bg-dark-bg-primary px-3 py-2 flex items-center">
                <Database size={14} class="text-blue-400 mr-2" />
                <h3 class="text-sm font-medium text-dark-text-primary">Chat History Storage</h3>
            </div>
            
            <div class="p-3">
                <div class="flex items-center justify-between mb-3">
                    <div class="space-y-1 mr-4">
                        <p class="text-sm font-medium text-dark-text-primary">
                            Save chat history locally
                        </p>
                        <p class="text-xs text-dark-text-secondary">
                            {internalPersistenceEnabled 
                                ? 'Your conversations will remain available between sessions' 
                                : 'Your sent messages will be lost when you log out'}
                        </p>
                    </div>
                    
                    {#if currentPersistenceSetting !== null}
                        <button 
                            role="switch"
                            aria-checked={internalPersistenceEnabled}
                            on:click={handleToggleChange}
                            class={`relative inline-flex items-center h-6 rounded-full w-11 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-dark-bg-secondary focus:ring-brand-green ${internalPersistenceEnabled ? 'bg-brand-green' : 'bg-dark-bg-tertiary'}`}
                        >
                            <span class="sr-only">Enable chat history persistence</span>
                            <span 
                                class={`inline-block w-4 h-4 transform bg-white rounded-full transition-transform duration-200 ease-in-out ${internalPersistenceEnabled ? 'translate-x-6' : 'translate-x-1'}`}
                            ></span>
                        </button>
                    {:else}
                        <span class="text-xs text-dark-text-disabled italic">Loading...</span>
                    {/if}
                </div>
                
                <div class="space-y-1.5">
                    <div class="flex items-start">
                        <MessageSquare size={12} class="text-blue-400 mt-0.5 mr-1.5 flex-shrink-0" />
                        <p class="text-xs text-dark-text-secondary">
                            Received messages can always be retrieved from the blockchain.
                        </p>
                    </div>
                    
                    <div class="flex items-start">
                        <AlertCircle size={12} class="text-red-400 mt-0.5 mr-1.5 flex-shrink-0" />
                        <p class="text-xs text-red-400">
                            All saved chat data is stored unencrypted on this computer.
                        </p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Delete History Card -->
        <div class="bg-dark-bg-secondary rounded-lg border border-dark-border-primary shadow-sm overflow-hidden">
            <div class="border-b border-dark-border-primary bg-dark-bg-primary px-3 py-2 flex items-center">
                <Trash2 size={14} class="text-red-400 mr-2" />
                <h3 class="text-sm font-medium text-dark-text-primary">Data Management</h3>
            </div>
            
            <div class="p-3">
                <div class="flex items-start justify-between">
                    <div class="space-y-1 mr-4">
                        <p class="text-sm font-medium text-dark-text-primary">
                            Delete chat history
                        </p>
                        <p class="text-xs text-dark-text-secondary">
                            Remove all locally stored conversations for this VerusID
                        </p>
                        <p class="text-xs text-red-400 font-medium mt-1">
                            This action cannot be undone
                        </p>
                    </div>
                    
                    <button 
                        type="button"
                        on:click={openConfirmDeleteModal}
                        disabled={currentPersistenceSetting === null}  
                        class="py-1.5 px-3 border border-red-500/70 rounded shadow-sm text-xs font-medium text-red-400 bg-dark-bg-tertiary hover:bg-red-900/50 focus:outline-none focus:ring-2 focus:ring-offset-1 focus:ring-offset-dark-bg-secondary focus:ring-red-500 transition-colors ease-in-out disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                    >
                        <Trash2 size={14} class="mr-1.5"/>
                        Delete All
                    </button>
                </div>
            </div>
        </div>
        
    </div>
</div>

<!-- Confirmation Modal -->
<ConfirmDeleteModal 
    bind:showModal={showConfirmDelete}
    on:confirm={handleConfirmDelete}
    on:cancel={handleCancelDelete}
/>

<style>
/* Styles for the toggle switch can be added or adjusted here if needed */
</style> 