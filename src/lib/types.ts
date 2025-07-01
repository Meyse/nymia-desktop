// File: src/lib/types.ts
// Description: Centralized type definitions for the application.
// Changes:
// - Added ChatMessage type for imported history.
// - Added optional 'status' field to ChatMessage.
// - Added Conversation type, including recipient_private_address.
// - BREAKING: Updated ChatMessage to use Unix timestamp in seconds for timestamp-based ordering.
// - Removed sentAtBlockHeight field as block-height sorting is replaced by timestamp sorting.
// - MAJOR: Added blockchain detection types for new automatic onboarding system
// - Added UtxoInfo type for Fast Messages feature
// - Added NamespaceOption type for VerusID registration namespace selection

// Credentials for Verus RPC connection
export interface Credentials {
    rpc_user: string;
    rpc_pass: string;
    rpc_port: number; // NEW: Port support for different blockchains
}

// Structure for Verus identity details returned from backend
export interface FormattedIdentity {
    formatted_name: string;
    i_address: string;
    private_address: string; // Now required (not nullable)
    balance: number | null; // Private balance (null while loading or on error)
}

// Generic structure for dropdown options
export interface DropdownOption {
    id: string | number | null;
    name: string;
    enabled: boolean;
    balance?: string | null; // Optional balance display (formatted string)
}

// Payload for successful login event
export interface LoginPayload {
    identity: FormattedIdentity;
    blockHeight: number | null;
    blockchainId: string | null; // NEW: Track selected blockchain for currency symbol
}

// Potential application states (can be expanded)
export type AppStatus = 'loading' | 'onboarding' | 'loggedIn' | 'error';

// Onboarding step names
export type OnboardingStep = 'welcome' | 'responsibility' | 'blockchain' | 'verusid'; 

// Type alias for the private balance
export type PrivateBalance = number | null;

// Type alias for the pending balance (0 confirmations)
export type PendingBalance = number | null;

// Structure for chat messages (with timestamp-based ordering)
export interface ChatMessage {
    id: string; // txid or generated ID for sent messages
    sender: string | 'self'; // VerusID of the sender (@ format) or 'self'
    text: string;
    timestamp: number; // Unix timestamp in seconds (UTC) when message was sent to blockchain
    amount: number;
    confirmations: number;
    direction: 'received' | 'sent';
    status?: 'sent' | 'delivered' | 'failed'; // Optional delivery status for sent messages
}

// Structure for conversation entries in the list
export type Conversation = {
    id: string;         // Unique ID, typically the recipient's VerusID name (e.g., user@)
    name: string;       // Display name (VerusID name)
    recipient_private_address: string; // The recipient's z-address needed for sending
    unread?: boolean;   // Optional flag for unread messages
  }; 

// NEW: UTXO information structure for Fast Messages feature
export interface UtxoInfo {
    total_utxos: number;           // Total count including dust
    usable_utxos: number;          // Count with amount >= 0.0001 (Fast Messages count)
    total_spendable_value: number; // Sum of usable UTXOs only
    largest_utxo: number;          // Largest single UTXO amount
    smallest_utxo: number;         // Smallest usable UTXO amount (>= 0.0001)
}

// NEW: Blockchain detection types
export type BlockchainStatus = 'Available' | 'Loading' | 'Error' | 'NotFound' | 'Timeout' | 'ParseError';

export interface BlockchainDetectionResult {
    blockchain_id: string;
    blockchain_name: string;
    status: BlockchainStatus;
    credentials: Credentials | null;
    config_path: string | null;
    error_message: string | null;
    block_height: number | null;
}

export interface ParallelDetectionResult {
    blockchains: BlockchainDetectionResult[];
    total_detected: number;
    detection_duration_ms: number;
}

// NEW: VerusID registration types
export interface NamespaceOption {
    name: string;
    currency_id: string;
    registration_fee: number;
    fully_qualified_name: string;
    fee_currency_name: string; // NEW: The actual currency name for the registration fee (e.g. "VRSCTEST", "SECOND")
} 