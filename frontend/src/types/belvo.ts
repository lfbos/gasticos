/**
 * Types for Belvo Open Banking integration.
 */

/**
 * Status of a bank connection.
 */
export type BelvoLinkStatus = 'valid' | 'invalid' | 'token_required' | 'unconfirmed';

/**
 * Belvo-supported Colombian bank institution codes.
 */
export type BelvoInstitution =
  | 'bancolombia_retail_co'
  | 'nequi_co'
  | 'daviplata_co'
  | 'nubank_co'
  | 'davivienda_retail_co';

/**
 * Bank information for the institution selector.
 */
export interface BelvoBank {
  code: BelvoInstitution;
  name: string;
  color?: string;
}

/**
 * Widget token response from the API.
 */
export interface WidgetTokenResponse {
  access_token: string;
}

/**
 * Connected bank (link) from the API.
 */
export interface ConnectedBank {
  id: string;
  institution: string;
  institution_name: string;
  status: BelvoLinkStatus;
  last_synced_at: string | null;
  created_at: string;
}

/**
 * Account type from Belvo.
 */
export type BelvoAccountType = 'checking' | 'savings' | 'credit_card' | 'loan' | 'other';

/**
 * Bank account from Belvo.
 */
export interface BelvoAccount {
  id: string;
  link_id: string;
  name: string | null;
  number_masked: string | null;
  account_type: BelvoAccountType;
  currency: string;
  balance_current: number | null;
  balance_available: number | null;
}

/**
 * Request to create a new link.
 */
export interface CreateLinkRequest {
  link_id: string;
  institution: string;
}

/**
 * Sync status response.
 */
export interface SyncStatusResponse {
  status: 'sync_queued';
  link_id: string;
}

/**
 * List of supported Colombian banks for Belvo.
 */
export const BELVO_SUPPORTED_BANKS: BelvoBank[] = [
  { code: 'bancolombia_retail_co', name: 'Bancolombia', color: '#FFDD00' },
  { code: 'nequi_co', name: 'Nequi', color: '#E2007A' },
  { code: 'daviplata_co', name: 'Daviplata', color: '#ED1C24' },
  { code: 'nubank_co', name: 'Nu Colombia', color: '#820AD1' },
  { code: 'davivienda_retail_co', name: 'Davivienda', color: '#ED1C24' },
];

/**
 * Get display name for a Belvo institution code.
 */
export function getBankDisplayName(code: string): string {
  const bank = BELVO_SUPPORTED_BANKS.find((b) => b.code === code);
  return bank?.name || code;
}

/**
 * Belvo widget success event data.
 */
export interface BelvoWidgetSuccessEvent {
  link: string;
  institution: string;
}

/**
 * Belvo widget error event data.
 */
export interface BelvoWidgetErrorEvent {
  error_code: string;
  error_message: string;
}
