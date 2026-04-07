/**
 * Belvo Open Banking API client.
 */

import { apiClient } from "./client";
import type {
  BelvoAccount,
  ConnectedBank,
  CreateLinkRequest,
  SyncStatusResponse,
  WidgetTokenResponse,
} from "@/types/belvo";

/**
 * Belvo API endpoints.
 */
export const belvoApi = {
  /**
   * Get a widget access token for the Belvo Connect widget.
   */
  async getWidgetToken(): Promise<WidgetTokenResponse> {
    const response = await apiClient.get<WidgetTokenResponse>("/belvo/token");
    return response.data;
  },

  /**
   * Create a new bank link after successful widget flow.
   */
  async createLink(data: CreateLinkRequest): Promise<ConnectedBank> {
    const response = await apiClient.post<ConnectedBank>("/belvo/links", data);
    return response.data;
  },

  /**
   * List all connected banks for the current user.
   */
  async listLinks(): Promise<ConnectedBank[]> {
    const response = await apiClient.get<{ data: ConnectedBank[] }>(
      "/belvo/links",
    );
    return response.data.data;
  },

  /**
   * Delete (disconnect) a bank link.
   */
  async deleteLink(linkId: string): Promise<void> {
    await apiClient.delete(`/belvo/links/${linkId}`);
  },

  /**
   * List all accounts for the current user.
   */
  async listAccounts(): Promise<BelvoAccount[]> {
    const response = await apiClient.get<{ data: BelvoAccount[] }>(
      "/belvo/accounts",
    );
    return response.data.data;
  },

  /**
   * Trigger a manual sync for a link.
   */
  async triggerSync(linkId: string): Promise<SyncStatusResponse> {
    const response = await apiClient.post<SyncStatusResponse>(
      `/belvo/sync/${linkId}`,
    );
    return response.data;
  },
};
