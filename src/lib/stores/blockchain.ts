// File: src/lib/stores/blockchain.ts
// Description: Svelte store for managing blockchain selection state
// This store holds information about the currently selected blockchain
// and can be accessed from any component for reactive updates

import { writable } from 'svelte/store';
import type { Credentials } from '$lib/types';

export interface SelectedBlockchain {
  id: string | null;
  name: string | null;
  credentials: Credentials | null;
  blockHeight: number | null;
}

// Create the writable store with initial null state
export const selectedBlockchain = writable<SelectedBlockchain>({
  id: null,
  name: null,
  credentials: null,
  blockHeight: null
});

// Helper function to update the selected blockchain
export function setSelectedBlockchain(blockchain: {
  id: string;
  name: string;
  credentials: Credentials;
  blockHeight: number;
}) {
  selectedBlockchain.set({
    id: blockchain.id,
    name: blockchain.name,
    credentials: blockchain.credentials,
    blockHeight: blockchain.blockHeight
  });
}

// Helper function to clear the blockchain selection
export function clearSelectedBlockchain() {
  selectedBlockchain.set({
    id: null,
    name: null,
    credentials: null,
    blockHeight: null
  });
} 