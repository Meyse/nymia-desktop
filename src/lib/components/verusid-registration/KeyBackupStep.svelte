<!-- 
  Component: src/lib/components/verusid-registration/KeyBackupStep.svelte
  Description: Step 4 of VerusID registration - display/export keys and require acknowledgment
  Changes:
  - New step that fetches and displays control (WIF) and shielded (z) private keys
  - Copy-only UI, plus guidance to back up wallet/keys in Verus Desktop
  - Requires two acknowledgments to enable completion
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';

  export let controlAddress: string;
  export let privateAddress: string; // zs-addr

  const dispatch = createEventDispatcher<{ ready: void }>();

  let controlPrivKey = '';
  let shieldedPrivKey = '';
  let copiedA = false;
  let copiedB = false;
  let showControlKey = false;
  let showShieldedKey = false;
  let ack1 = false;
  let ack2 = false;
  let loading = true;
  let errorMsg: string | null = null;

  onMount(async () => {
    try {
      controlPrivKey = await invoke<string>('dump_privkey', { address: controlAddress });
      shieldedPrivKey = await invoke<string>('export_z_key', { zAddress: privateAddress });
    } catch (e: any) {
      errorMsg = e?.message || String(e);
    } finally {
      loading = false;
    }
  });

  $: canFinish = ack1 && ack2;
  $: if (canFinish) {
    dispatch('ready');
  }

  function copyControlKey() {
    navigator.clipboard.writeText(controlPrivKey);
    showControlKey = true;
    copiedA = true;
    setTimeout(() => copiedA = false, 1500);
  }

  function copyShieldedKey() {
    navigator.clipboard.writeText(shieldedPrivKey);
    showShieldedKey = true;
    copiedB = true;
    setTimeout(() => copiedB = false, 1500);
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-dark-text-primary mb-2">Keys & Backup</h3>
    <p class="text-sm text-dark-text-secondary">Your VerusID has been created! Save your private keys to secure access to your identity.</p>
  </div>

  {#if loading}
    <div class="bg-black/20 border border-white/10 rounded-lg p-4">
      <div class="h-4 bg-white/10 rounded animate-pulse w-40 mb-2"></div>
      <div class="h-4 bg-white/10 rounded animate-pulse w-64"></div>
    </div>
  {:else if errorMsg}
    <div class="bg-red-900/40 border border-red-600/50 rounded-lg p-4 text-sm text-red-300">{errorMsg}</div>
  {:else}
    <div class="space-y-6">
      <!-- Control Address Section -->
      <div class="bg-black/20 border border-white/10 rounded-lg p-4">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-sm font-medium text-white">Control Address</div>
            <div class="text-xs text-white/60">The address controls your VerusID</div>
          </div>
          <Button variant="secondary" size="small" on:click={copyControlKey}>
            <svg slot="icon" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            {copiedA ? 'Copied!' : 'Copy Private Key'}
          </Button>
        </div>
        
        <div class="bg-white/5 rounded-lg p-3">
          <div class="text-xs text-white/50 mb-2">Address:</div>
          <div class="font-mono text-white text-sm break-all">
            {controlAddress}
          </div>
        </div>
        
        {#if showControlKey}
          <div class="mt-3 bg-green-900/20 border border-green-600/30 rounded-lg p-3">
            <div class="text-xs text-green-400 mb-2">Private Key (copied to clipboard):</div>
            <div class="font-mono text-green-200 text-sm break-all">
              {controlPrivKey}
            </div>
          </div>
        {/if}
      </div>

      <!-- Private Address Section -->
      <div class="bg-black/20 border border-white/10 rounded-lg p-4">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-sm font-medium text-white">Private Address</div>
            <div class="text-xs text-white/60">Fund this address to enable messaging</div>
          </div>
          <div class="flex space-x-2">
            <Button variant="secondary" size="small" on:click={() => { navigator.clipboard.writeText(privateAddress); }}>
              <svg slot="icon" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              Copy Address
            </Button>
            <Button variant="secondary" size="small" on:click={copyShieldedKey}>
              <svg slot="icon" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
              {copiedB ? 'Copied!' : 'Copy Private Key'}
            </Button>
          </div>
        </div>
        
        <div class="bg-white/5 rounded-lg p-3">
          <div class="text-xs text-white/50 mb-2">Address:</div>
          <div class="font-mono text-white text-sm break-all">
            {privateAddress}
          </div>
        </div>
        
        {#if showShieldedKey}
          <div class="mt-3 bg-green-900/20 border border-green-600/30 rounded-lg p-3">
            <div class="text-xs text-green-400 mb-2">Private Key (copied to clipboard):</div>
            <div class="font-mono text-green-200 text-sm break-all">
              {shieldedPrivKey}
            </div>
          </div>
        {/if}
        
        <div class="mt-3 bg-amber-900/20 border border-amber-600/30 rounded-lg p-3">
          <div class="flex items-start space-x-2">
            <svg class="w-4 h-4 text-amber-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <div class="text-xs text-amber-300">
              <strong>Important:</strong> You need to fund this private address to send messages with Nymia.
            </div>
          </div>
        </div>
      </div>

      <!-- Backup Info -->
      <div class="bg-blue-900/20 border border-blue-600/30 rounded-lg p-3">
        <div class="flex items-start space-x-2">
          <svg class="w-4 h-4 text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
          <div class="text-xs text-blue-300">
            <strong>Tip:</strong> You can also back up your entire wallet in Verus Desktop for additional security.
          </div>
        </div>
      </div>

      <!-- Acknowledgment Checkboxes -->
      <div class="space-y-4">
        <label class="flex items-start space-x-3 text-sm text-white/80 cursor-pointer select-none">
          <div class="relative">
            <input type="checkbox" bind:checked={ack1} class="sr-only" />
            <div class="w-5 h-5 border-2 border-white/30 rounded flex items-center justify-center transition-colors duration-200 {ack1 ? 'bg-brand-green border-brand-green' : 'bg-transparent hover:border-white/50'}">
              {#if ack1}
                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              {/if}
            </div>
          </div>
          <span>I have securely saved the control address private key</span>
        </label>
        <label class="flex items-start space-x-3 text-sm text-white/80 cursor-pointer select-none">
          <div class="relative">
            <input type="checkbox" bind:checked={ack2} class="sr-only" />
            <div class="w-5 h-5 border-2 border-white/30 rounded flex items-center justify-center transition-colors duration-200 {ack2 ? 'bg-brand-green border-brand-green' : 'bg-transparent hover:border-white/50'}">
              {#if ack2}
                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              {/if}
            </div>
          </div>
          <span>I have securely saved the private address private key</span>
        </label>
      </div>
    </div>
  {/if}
</div>

<style>
</style>

