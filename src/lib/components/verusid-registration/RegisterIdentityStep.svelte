<!-- 
  Component: src/lib/components/verusid-registration/RegisterIdentityStep.svelte
  Description: Step 3 of VerusID registration - commit and finalize the identity on-chain
  Changes:
  - New step implementing two-phase process: registernamecommitment → wait 1 conf → registeridentity → wait 1 conf
  - Generates fresh control (R-addr) and private (zs-addr) addresses
  - Polls confirmations every 10s with 30-minute timeouts per phase
  - Emits completion event with addresses for next step (Keys & Backup)
  - Compact UI consistent with app styling
  - Added dynamic timing feedback with elapsed time tracking
  - Progressive messaging based on wait duration (0-2min, 2-5min, 5-15min, 15min+)
  - Verus-specific messaging about mining + staking block creation
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';
  import type { NamespaceOption } from '$lib/types';

  export let name: string; // plain name (no trailing @)
  export let selectedNamespace: NamespaceOption; // from step 1
  export let referralCode: string; // may be empty string

  const dispatch = createEventDispatcher<{
    completed: { controlAddress: string; privateAddress: string };
    error: { message: string };
  }>();

  let isRoot = false;
  let controlAddress = '';
  let privateAddress = '';
  let commitTxid: string = '';
  let finalizeTxid: string = '';
  let phase: 'idle' | 'committing' | 'waitingCommit' | 'finalizing' | 'waitingFinalize' | 'done' | 'error' = 'idle';
  let errorMsg: string | null = null;

  // Timing tracking for dynamic messages
  let phaseStartTime: number = 0;
  let elapsedSeconds: number = 0;
  let timerInterval: number | null = null;

  const POLL_INTERVAL_SECS = 10;
  const TIMEOUT_SECS = 30 * 60; // 30 minutes

  $: fullId = isRoot ? `${name}@` : `${name}.${selectedNamespace.name}@`;
  $: identityNameForFinalize = isRoot ? name : `${name}.${selectedNamespace.name}`;

  onMount(() => {
    isRoot = isRootNamespace(selectedNamespace);
    startFlow();
    
    return () => {
      // Cleanup timer on unmount
      if (timerInterval) {
        clearInterval(timerInterval);
      }
    };
  });

  function isRootNamespace(namespace: NamespaceOption): boolean {
    // Same rule as step 1: name '@' when root
    const lname = namespace.name.toLowerCase();
    return lname === 'vrsc' || lname === 'vrsctest';
  }

  async function startFlow() {
    errorMsg = null;
    try {
      phase = 'committing';
      console.log('[RegisterID] acquiring control address…');
      controlAddress = await invoke<string>('get_new_address');
      console.log('[RegisterID] control address =', controlAddress);

      const parentNamespace = isRoot ? '' : selectedNamespace.name;
      const referral = (referralCode || '').trim(); // explicit "" when empty below

      console.log('[RegisterID] register_name_commitment', { name, controlAddress, referral, parentNamespace });
      const commit = await invoke<{ txid: string; namereservation: any }>('register_name_commitment', {
        name,
        controlAddress,
        referralIdentity: referral === '' ? '' : referral,
        parentNamespace: parentNamespace === '' ? '' : parentNamespace,
      });
      commitTxid = commit.txid;
      console.log('[RegisterID] commit txid =', commitTxid);

      phase = 'waitingCommit';
      startTimer();
      console.log('[RegisterID] waiting for next block (commit)…');
      const commitOk = await invoke<boolean>('wait_for_block_increase', {
        blocks: 1,
        intervalSecs: POLL_INTERVAL_SECS,
        timeoutSecs: TIMEOUT_SECS,
      });
      stopTimer();
      if (!commitOk) throw new Error('Commit transaction not confirmed in time.');

      phase = 'finalizing';
      console.log('[RegisterID] acquiring private address…');
      privateAddress = await invoke<string>('get_new_private_address');
      console.log('[RegisterID] private address =', privateAddress);

      const identityBundle = {
        txid: commitTxid,
        namereservation: commit.namereservation,
        identity: {
          name: identityNameForFinalize,
          primaryaddresses: [controlAddress],
          minimumsignatures: 1,
          revocationauthority: [""],
          recoveryauthority: [""],
          privateaddress: privateAddress,
        },
      };

      console.log('[RegisterID] register_identity', identityBundle);
      finalizeTxid = await invoke<string>('register_identity', { identityBundle });
      console.log('[RegisterID] finalize txid =', finalizeTxid);

      phase = 'waitingFinalize';
      startTimer();
      console.log('[RegisterID] waiting for next block (finalize)…');
      const finOk = await invoke<boolean>('wait_for_block_increase', {
        blocks: 1,
        intervalSecs: POLL_INTERVAL_SECS,
        timeoutSecs: TIMEOUT_SECS,
      });
      stopTimer();
      if (!finOk) throw new Error('Finalize transaction not confirmed in time.');

      // Readiness check: wait for identity to be available
      console.log('[RegisterID] waiting for identity to be ready', fullId);
      const identityReady = await invoke<boolean>('wait_for_identity_ready', {
        identityName: fullId,
        intervalSecs: POLL_INTERVAL_SECS,
        timeoutSecs: TIMEOUT_SECS,
      });
      if (!identityReady) {
        throw new Error('Identity not available after registration - this may indicate a network issue.');
      }

      phase = 'done';
      console.log('[RegisterID] registration completed');
      dispatch('completed', { controlAddress, privateAddress });
    } catch (e: any) {
      stopTimer();
      errorMsg = e?.message || String(e);
      phase = 'error';
      console.error('[RegisterID] error', errorMsg);
      dispatch('error', { message: errorMsg || 'Unknown error' });
    }
  }

  function getCurrentStep(): number {
    if (phase === 'committing' || phase === 'waitingCommit') return 1;
    if (phase === 'finalizing' || phase === 'waitingFinalize') return 2;
    return 1;
  }

  function getProgressPercentage(): number {
    if (phase === 'idle' || phase === 'committing') return 0;
    if (phase === 'waitingCommit') return 25;
    if (phase === 'finalizing') return 50;
    if (phase === 'waitingFinalize') return 75;
    if (phase === 'done') return 100;
    return 0;
  }

  function startTimer() {
    phaseStartTime = Date.now();
    elapsedSeconds = 0;
    
    if (timerInterval) {
      clearInterval(timerInterval);
    }
    
    timerInterval = setInterval(() => {
      elapsedSeconds = Math.floor((Date.now() - phaseStartTime) / 1000);
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function getWaitingMessage(): string {
    if (elapsedSeconds < 120) { // 0-2 minutes
      return "Waiting for next block (~1 minute average)...";
    } else if (elapsedSeconds < 300) { // 2-5 minutes
      return "Block creation is taking longer than average...";
    } else if (elapsedSeconds < 900) { // 5-15 minutes
      return "Blocks can naturally take longer to be found - this is normal";
    } else { // 15+ minutes
      return "Experiencing a longer block time, but your transaction will complete";
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
</script>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h3 class="text-lg font-medium text-dark-text-primary mb-2">Register ID</h3>
    <p class="text-sm text-dark-text-secondary">Creating your VerusID on the blockchain</p>
  </div>

  <!-- Registration Summary -->
  <div class="bg-black/20 border border-white/5 rounded-md px-3 py-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-2">
        <span class="text-xs text-white/60">Creating:</span>
        <span class="text-xs font-mono text-white font-medium">{fullId}</span>
      </div>
      <div class="text-right">
        <div class="text-xs text-white/50">
          {#if phase === 'done'}✓ Completed{:else if phase === 'error'}✗ Error{:else}In progress...{/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Progress Bar -->
  <div class="space-y-4">
    <div class="space-y-3">
      <div class="flex items-center justify-between text-sm">
        <span class="text-white/70">Progress</span>
        <span class="text-white/70">
          {#if phase === 'done'}Complete{:else if phase === 'error'}Error{:else}Step {getCurrentStep()} of 2{/if}
        </span>
      </div>
      
      <!-- Progress Bar -->
      <div class="w-full bg-white/10 rounded-full h-2">
        <div 
          class="bg-brand-green h-2 rounded-full transition-all duration-500"
          style="width: {getProgressPercentage()}%"
        ></div>
      </div>
      
      <!-- Current Status -->
      <div class="text-sm text-white/80">
        {#if phase === 'committing'}
          Committing name reservation...
        {:else if phase === 'waitingCommit'}
          Phase 1: Waiting for block confirmation
        {:else if phase === 'finalizing'}
          Creating identity on blockchain...
        {:else if phase === 'waitingFinalize'}
          Phase 2: Waiting for block confirmation
        {:else if phase === 'done'}
          Registration completed successfully!
        {:else}
          Preparing registration...
        {/if}
      </div>
    </div>

    <!-- Active Progress Indicator -->
    {#if phase !== 'idle' && phase !== 'error' && phase !== 'done'}
      <div class="bg-black/20 border border-white/10 rounded-lg p-4">
        <div class="flex items-center space-x-3">
          <div class="animate-spin h-5 w-5 border-2 border-brand-green border-t-transparent rounded-full"></div>
          <div class="flex-1">
            <div class="text-sm text-white">
              {#if phase === 'committing'}
                Submitting transaction to blockchain...
              {:else if phase === 'waitingCommit' || phase === 'waitingFinalize'}
                {getWaitingMessage()}
              {:else if phase === 'finalizing'}
                Creating final identity transaction...
              {:else}
                Processing your VerusID registration...
              {/if}
            </div>
            {#if (phase === 'waitingCommit' || phase === 'waitingFinalize') && elapsedSeconds > 0}
              <div class="text-xs text-white/60 mt-1">
                Elapsed: {formatElapsedTime()}
                {#if elapsedSeconds >= 120}
                  • Typically 2-4 minutes total
                {/if}
              </div>
            {/if}
          </div>
        </div>
        
        {#if phase === 'waitingCommit' || phase === 'waitingFinalize'}
          <div class="mt-3 text-xs text-white/50">
            <div class="flex items-center space-x-1">
              <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
              </svg>
              <span>Blocks are created through mining and staking (average 1 minute, but can vary up to 15 minutes)</span>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Error state -->
    {#if errorMsg}
      <div class="bg-red-900/40 border border-red-600/50 rounded-lg p-4">
        <div class="flex items-start space-x-3">
          <svg class="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div class="flex-1">
            <h4 class="text-sm font-medium text-red-400 mb-1">Registration Failed</h4>
            <p class="text-sm text-red-300 mb-3">{errorMsg}</p>
            <Button
              variant="secondary"
              size="small"
              on:click={() => startFlow()}
            >
              Try Again
            </Button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Success state -->
    {#if phase === 'done'}
      <div class="bg-green-900/40 border border-green-600/50 rounded-lg p-4">
        <div class="flex items-start space-x-3">
          <svg class="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
          </svg>
          <div class="flex-1">
            <h4 class="text-sm font-medium text-green-400 mb-1">Registration Complete!</h4>
            <p class="text-sm text-green-300">
              Your VerusID <span class="font-mono">{fullId}</span> has been successfully created and is now available on the blockchain.
            </p>
          </div>
        </div>
      </div>
    {/if}


  </div>
</div>

<style>
</style>

