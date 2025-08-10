<script lang="ts">
// Component: src/lib/components/chat/UserInfoSection.svelte
// Description: Discord-style user info section for the bottom of the sidebar
// Displays user avatar, name, balance, block height, and action buttons (settings, logout)
// Changes:
// - Created new component for sidebar user info section
// - Discord-style layout with avatar, name, and action buttons
// - Balance and block height on separate lines
// - Fixed positioning at bottom of sidebar
// - Added dynamic currency symbol support based on selected blockchain
// - Updated background color to specific hex color #121214
// - Improved private balance display with better layout and monospace font
// - Added comma formatting for block height numbers and removed # symbol
// - Added Fast Messages row with UTXO count display and hoverable tooltip
// - Added UtxoInfo props and loading states for Fast Messages feature
// - Added Pending Balance (0 confirmations) shown only when different from Private Balance
// - Reordered rows: Pending Balance, Private Balance, Fast Messages, Profile, Block Height
// - Removed isTransactionPending prop and transaction spinner logic
// - Added dynamic balance labeling: "Total"/"Available" when discrepancy, "Private Balance" when same
// - Added visual hierarchy with larger "Total" text and spinner indicating waiting for confirmations
// - Fixed Fast Messages tooltip positioning and flashing issues with proper relative positioning and pointer-events-none
// - FIXED: Always show Private Balance and Fast Messages rows with skeleton loading states instead of hiding them
// - Added "Fund" button when private balance is exactly 0 AND no pending balance to open funding modal

  import { createEventDispatcher } from 'svelte';
  import { LogOut, Settings, Layers, Loader, FastForward, HelpCircle, Plus } from 'lucide-svelte';
  import Avatar from '../Avatar.svelte';
  import FundingModal from './FundingModal.svelte';
  import type { PrivateBalance, PendingBalance, UtxoInfo } from '$lib/types';

  // --- Props ---
  export let verusIdName: string = 'Unknown User';
  export let privateBalance: PrivateBalance = null;
  export let pendingBalance: PendingBalance = null; // NEW: Balance with 0 confirmations
  export let blockHeight: number | null = null;
  export let currencySymbol: string = 'VRSC'; // Dynamic currency symbol
  export let utxoInfo: UtxoInfo | null = null; // NEW: UTXO information for Fast Messages
  // Removed unused isUtxoLoading prop
  export let privateAddress: string = ''; // NEW: Private address for funding

  // --- Events ---
  const dispatch = createEventDispatcher<{
    logout: void;
    settings: void;
    funding: { success: boolean }; // NEW: For funding completion
  }>();

  // --- State ---
  let showTooltip = false;
  let showFundingModal = false; // NEW: Funding modal state

  function handleLogout() {
    dispatch('logout');
  }
  
  function handleSettings() {
    dispatch('settings');
  }

  function handleFunding() {
    showFundingModal = true;
  }

  function handleCloseFundingModal(event: CustomEvent<{ success: boolean }>) {
    showFundingModal = false;
    
    // If funding was successful, dispatch event to parent
    if (event.detail.success) {
      dispatch('funding', { success: true });
    }
  }

  function showTooltipOnHover() {
    showTooltip = true;
  }

  function hideTooltipOnLeave() {
    showTooltip = false;
  }

  // Format balances for display
  $: formattedPrivateBalance = privateBalance !== null ? `${privateBalance.toFixed(4)} ${currencySymbol}` : null;
  $: formattedPendingBalance = pendingBalance !== null ? `${pendingBalance.toFixed(4)} ${currencySymbol}` : null;
  
  // Show pending balance only when it differs from private balance
  $: showPendingBalance = pendingBalance !== null && privateBalance !== null && pendingBalance !== privateBalance;

  // Dynamic labels based on whether there's a discrepancy
  $: balanceLabel = showPendingBalance ? 'Available' : 'Private Balance';

  // Check if balance is exactly 0 AND no pending balance to show Fund button
  $: showFundButton = privateBalance === 0 && (pendingBalance === null || pendingBalance === 0);

  // Format block height with commas
  function formatBlockHeight(height: number): string {
    return height.toLocaleString();
  }

</script>



<div class="border-t border-dark-border-primary p-3 flex-shrink-0" style="background-color: #121214">


  <!-- Row 2: Pending Balance (NEW - shown only when different from private balance) -->
  {#if showPendingBalance}
    <div class="flex items-center justify-between text-xs mb-2">
      <span class="text-dark-text-primary font-medium cursor-default select-none">Total</span>
      <div class="flex items-center">
        <Loader size={16} class="mr-2 text-white animate-spin" />
        <span class={`font-mono font-bold cursor-default select-none ${pendingBalance !== null ? 'text-dark-text-primary' : 'text-dark-text-disabled'}`}>
          {formattedPendingBalance}
        </span>
      </div>
    </div>
  {/if}

  <!-- Row 3: Private Balance (Always shown) -->
  <div class="flex items-center justify-between text-xs mb-3">
    <span class={`cursor-default select-none ${showPendingBalance ? 'text-white/60' : 'text-white/45'}`}>{balanceLabel}</span>
    {#if formattedPrivateBalance !== null}
      {#if showFundButton}
        <!-- Fund button when balance is exactly 0 -->
        <button
          on:click={handleFunding}
          class="inline-flex items-center px-2 py-1 text-xs font-medium text-white bg-brand-green hover:bg-[#32905D] border border-transparent rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-brand-green transition-colors duration-150"
        >
          <Plus size={12} class="mr-1" />
          Fund
        </button>
      {:else}
        <span class={`font-mono font-bold cursor-default select-none ${showPendingBalance ? 'text-green-400' : 'text-green-300'}`}>
          {formattedPrivateBalance}
        </span>
      {/if}
    {:else}
      <!-- Skeleton loader for private balance -->
      <div class="skeleton-loader w-20 h-4 rounded"></div>
    {/if}
  </div>

  <!-- Row 4: Fast Messages (Always shown) -->
  <div class="flex items-center justify-between text-xs mb-3">
    <div class="flex items-center">
      <span class="text-white/45 cursor-default select-none">Fast Messages</span>
      <div class="ml-1 relative"
           role="button"
           tabindex="0"
           on:mouseenter={showTooltipOnHover}
           on:mouseleave={hideTooltipOnLeave}>
        <HelpCircle size={14} class="text-white/50 hover:text-white/80" />
        
        <!-- Tooltip positioned relative to icon -->
        {#if showTooltip}
          <div class="absolute bottom-6  bg-black border border-dark-border-primary rounded-lg p-3 shadow-lg w-64 z-50 cursor-default select-none pointer-events-none">
            <div class="text-sm text-dark-text-primary mb-2 font-semibold flex items-center">
              <FastForward size={16} class="mr-1 text-white/50" />
              Fast Messages
            </div>
            <div class="text-xs text-dark-text-secondary leading-relaxed">
              Fast Messages are UTXOs (≥0.0001) that can send immediate transactions (messages) without waiting for a confirmation.
              {#if utxoInfo}
                <div class="mt-2 pt-2 border-t border-dark-border-primary">
                  <div class="text-dark-text-primary">Current breakdown:</div>
                  <div class="mt-1">
                    • {utxoInfo.usable_utxos} usable UTXOs<br>
                    {#if utxoInfo.usable_utxos > 1}
                      • Largest: {utxoInfo.largest_utxo.toFixed(4)} {currencySymbol}<br>
                      • Smallest: {utxoInfo.smallest_utxo.toFixed(4)} {currencySymbol}
                    {:else if utxoInfo.usable_utxos === 1}
                      • Largest: {utxoInfo.largest_utxo.toFixed(4)} {currencySymbol}
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>
    <div class="flex items-center">
      {#if utxoInfo !== null}
        <span class={`font-mono font-bold cursor-default select-none ${utxoInfo.usable_utxos > 0 ? 'text-dark-text-primary' : 'text-white/30'}`}>
          {utxoInfo.usable_utxos}
        </span>
      {:else}
        <!-- Skeleton loader for UTXO count -->
        <div class="skeleton-loader w-6 h-4 rounded"></div>
      {/if}
      <FastForward size={14} class="ml-1 text-white/40" />
    </div>
  </div>

  <!-- Separator before profile -->
  <div class="h-px bg-white/10 my-3"></div>

  <!-- Row 5: Profile (Avatar + Name + Action Buttons) - Moved here -->
  <div class="flex items-center justify-between">
    <!-- Left: Avatar + Name -->
    <div class="flex items-center min-w-0 flex-1">
      <Avatar userId={verusIdName} size="small" showHover={false} />
      <div class="ml-2 min-w-0 flex-1">
        <div class="font-semibold text-dark-text-primary text-sm truncate">
          {verusIdName}
        </div>
      </div>
    </div>

    <!-- Right: Action Buttons -->
    <div class="flex items-center space-x-1 ml-2">
      <button 
        on:click={handleSettings} 
        title="Settings"
        class="p-1.5 rounded-md text-white/50 hover:text-dark-text-primary focus:outline-none focus:ring-1 focus:ring-brand-green"
      >
        <Settings size={16} />
      </button>
      
      <button 
        on:click={handleLogout} 
        title="Log Out"
        class="p-1.5 rounded-md text-white/50 hover:text-dark-text-primary focus:outline-none focus:ring-1 focus:ring-brand-green"
      >
        <LogOut size={16} />
      </button>
    </div>
  </div>

  <!-- Separator before block height -->
  <div class="h-px bg-white/10 my-3"></div>

  <!-- Row 6: Block Height (Subtle) -->
  <div class="flex items-center text-xs text-white/45 cursor-default select-none">
    <Layers size={10} class="mr-1" />
    {#if blockHeight !== null}
      <span>Block {formatBlockHeight(blockHeight)}</span>
    {:else}
      <span class="mr-1">Block</span>
      <div class="skeleton-loader w-12 h-3 rounded"></div>
    {/if}
  </div>
</div>

<!-- Funding Modal -->
<FundingModal 
  bind:show={showFundingModal}
  privateAddress={privateAddress}
  currencySymbol={currencySymbol}
  on:close={handleCloseFundingModal}
/>

<style>
  /* Skeleton loader animation */
  .skeleton-loader {
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.1) 25%, rgba(255, 255, 255, 0.2) 50%, rgba(255, 255, 255, 0.1) 75%);
    background-size: 200% 100%;
    animation: skeleton-loading 1.5s infinite ease-in-out;
  }

  @keyframes skeleton-loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style> 