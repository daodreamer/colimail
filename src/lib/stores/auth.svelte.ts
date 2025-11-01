import { supabase, type AppUser, getCurrentUser } from '$lib/supabase';
import type { AuthChangeEvent, Session } from '@supabase/supabase-js';

// Auth state management using Svelte 5 runes
class AuthStore {
  user = $state<AppUser | null>(null);
  session = $state<Session | null>(null);
  loading = $state(true);
  initialized = $state(false);

  constructor() {
    this.initialize();
    this.setupDeepLinkHandler();
  }

  private setupDeepLinkHandler() {
    // Listen for deep link events (OAuth callbacks)
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen('deep-link', async (event: any) => {
          const url = event.payload as string;
          console.log('Received deep link:', url);

          // Parse the URL to extract OAuth tokens
          try {
            const urlObj = new URL(url);
            // Handle auth callback (e.g., colimail://auth/callback#access_token=...)
            if (urlObj.pathname === '/auth/callback' || urlObj.pathname === '//auth/callback') {
              // Redirect to the callback page which will handle the token extraction
              window.location.href = `/auth/callback${urlObj.hash}`;
            }
          } catch (error) {
            console.error('Failed to parse deep link URL:', error);
          }
        });
      });
    }
  }

  async initialize() {
    try {
      console.log('[AuthStore] Initializing auth store...');

      // Get current session
      const { data: { session } } = await supabase.auth.getSession();
      this.session = session;

      console.log('[AuthStore] Initial session:', session ? `User: ${session.user.email}` : 'No session');

      if (session) {
        this.user = await getCurrentUser();
        console.log('[AuthStore] Loaded user:', this.user);
      }

      // Listen for auth changes
      supabase.auth.onAuthStateChange(this.handleAuthStateChange.bind(this));
      console.log('[AuthStore] Auth state change listener registered');
    } catch (error) {
      console.error('[AuthStore] Error initializing auth:', error);
    } finally {
      this.loading = false;
      this.initialized = true;
      console.log('[AuthStore] Initialization complete. Authenticated:', this.isAuthenticated);
    }
  }

  private async handleAuthStateChange(event: AuthChangeEvent, session: Session | null) {
    console.log('[AuthStore] Auth state changed:', event, session?.user?.email);

    this.session = session;

    // Handle session refresh errors
    if (event === 'TOKEN_REFRESHED' && !session) {
      console.error('[AuthStore] Token refresh failed, session expired');
      this.user = null;
      // Optionally notify user that they need to log in again
      return;
    }

    if (session) {
      console.log('[AuthStore] Session detected, loading user...');
      this.user = await getCurrentUser();
      console.log('[AuthStore] User loaded:', this.user);

      // Sync user to local database
      if (this.user) {
        console.log('[AuthStore] Syncing user to database...');
        await this.syncUserToDatabase(this.user);
        console.log('[AuthStore] User synced to database');
      }
    } else {
      console.log('[AuthStore] No session, clearing user');
      this.user = null;
    }

    console.log('[AuthStore] Auth state updated. Authenticated:', this.isAuthenticated);
  }

  private async syncUserToDatabase(user: AppUser) {
    // Sync user data to local SQLite database
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('sync_app_user', {
          userId: user.id,
          email: user.email,
          name: user.name,
          avatarUrl: user.avatarUrl,
          subscriptionTier: user.subscriptionTier,
          subscriptionExpiresAt: user.subscriptionExpiresAt
            ? new Date(user.subscriptionExpiresAt).getTime() / 1000
            : null,
        });
      } catch (error) {
        console.error('Failed to sync user to database:', error);
      }
    }
  }

  get isAuthenticated() {
    return this.user !== null && this.session !== null;
  }

  get isFreeUser() {
    return this.user?.subscriptionTier === 'free';
  }

  get isProUser() {
    return this.user?.subscriptionTier === 'pro';
  }

  get isEnterpriseUser() {
    return this.user?.subscriptionTier === 'enterprise';
  }

  async refreshUser() {
    console.log('[AuthStore] Manual refresh requested');

    if (!this.session) {
      console.log('[AuthStore] No session found, fetching session...');
      const { data: { session } } = await supabase.auth.getSession();
      this.session = session;
      console.log('[AuthStore] Session fetched:', session ? `User: ${session.user.email}` : 'No session');
    }

    if (!this.session) {
      console.log('[AuthStore] Still no session after fetch, cannot refresh user');
      return;
    }

    console.log('[AuthStore] Refreshing user data...');
    this.user = await getCurrentUser();
    console.log('[AuthStore] User refreshed:', this.user);

    if (this.user) {
      console.log('[AuthStore] Syncing refreshed user to database...');
      await this.syncUserToDatabase(this.user);
      console.log('[AuthStore] Refresh complete. Authenticated:', this.isAuthenticated);
    }
  }
}

// Export singleton instance
export const authStore = new AuthStore();
