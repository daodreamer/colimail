import { createClient } from '@supabase/supabase-js';

// Supabase configuration
// TODO: Replace with your actual Supabase project URL and anon key
const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || '';
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY || '';

if (!supabaseUrl || !supabaseAnonKey) {
  console.warn('Supabase URL or anon key not configured. Please set VITE_SUPABASE_URL and VITE_SUPABASE_ANON_KEY in .env file.');
}

// Create Supabase client with custom storage for Tauri
export const supabase = createClient(supabaseUrl, supabaseAnonKey, {
  auth: {
    autoRefreshToken: true,
    persistSession: true,
    detectSessionInUrl: true,
    storage: {
      getItem: async (key: string) => {
        // Try localStorage first (more reliable for large session tokens)
        if (typeof window !== 'undefined' && window.localStorage) {
          try {
            const value = localStorage.getItem(key);
            if (value) return value;
          } catch (error) {
            console.error('Failed to read from localStorage:', error);
          }
        }

        // Fallback to Tauri secure storage for smaller items
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            return await invoke<string>('get_secure_storage', { key });
          } catch (error) {
            // Silently fail, item might not exist
            return null;
          }
        }

        return null;
      },
      setItem: async (key: string, value: string) => {
        // Use localStorage for session storage (Windows Credential Manager has 2560 char limit)
        if (typeof window !== 'undefined' && window.localStorage) {
          try {
            localStorage.setItem(key, value);
            console.log(`[Supabase Storage] Saved ${key} to localStorage (${value.length} chars)`);
            return;
          } catch (error) {
            console.error('Failed to save to localStorage:', error);
          }
        }

        // Try Tauri secure storage as fallback (will fail for large tokens)
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            await invoke('set_secure_storage', { key, value });
            console.log(`[Supabase Storage] Saved ${key} to secure storage`);
          } catch (error) {
            console.warn('Failed to save to secure storage (likely too large):', error);
          }
        }
      },
      removeItem: async (key: string) => {
        // Remove from localStorage
        if (typeof window !== 'undefined' && window.localStorage) {
          try {
            localStorage.removeItem(key);
            console.log(`[Supabase Storage] Removed ${key} from localStorage`);
          } catch (error) {
            console.error('Failed to remove from localStorage:', error);
          }
        }

        // Also try to remove from Tauri secure storage
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            await invoke('delete_secure_storage', { key });
          } catch (error) {
            // Silently ignore errors
          }
        }
      },
    },
  },
});

// Type definitions for our app user
export interface AppUser {
  id: string;
  email: string;
  displayName?: string;
  avatarUrl?: string;
  subscriptionTier: 'free' | 'pro' | 'enterprise';
  subscriptionExpiresAt?: string;
  createdAt?: string;
}

// Get current user session
export async function getCurrentSession() {
  const { data, error } = await supabase.auth.getSession();
  if (error) {
    console.error('Error getting session:', error);
    return null;
  }
  return data.session;
}

// Get current user
export async function getCurrentUser(): Promise<AppUser | null> {
  const session = await getCurrentSession();
  if (!session?.user) {
    console.log('[Supabase] getCurrentUser: No session or user found');
    return null;
  }

  console.log('[Supabase] getCurrentUser: Found user', {
    id: session.user.id,
    email: session.user.email,
    metadata: session.user.user_metadata
  });

  return {
    id: session.user.id,
    email: session.user.email || '',
    displayName: session.user.user_metadata?.display_name || session.user.email?.split('@')[0],
    avatarUrl: session.user.user_metadata?.avatar_url,
    subscriptionTier: session.user.user_metadata?.subscription_tier || 'free',
    subscriptionExpiresAt: session.user.user_metadata?.subscription_expires_at,
    createdAt: session.user.created_at,
  };
}

// Sign up with email and password
export async function signUpWithEmail(email: string, password: string, displayName?: string) {
  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      data: {
        display_name: displayName,
        subscription_tier: 'free',
      },
    },
  });

  if (error) throw error;
  return data;
}

// Sign in with email and password
export async function signInWithEmail(email: string, password: string) {
  const { data, error } = await supabase.auth.signInWithPassword({
    email,
    password,
  });

  if (error) throw error;
  return data;
}

// Sign in with Google OAuth
export async function signInWithGoogle() {
  // Use Supabase's callback URL for OAuth
  // After authorization, Supabase will redirect to the callback page with the session tokens
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: 'google',
    options: {
      // Redirect back to our app's callback page after OAuth completes
      redirectTo: `${window.location.origin}/auth/callback`,
      queryParams: {
        access_type: 'offline',
        prompt: 'consent',
      },
    },
  });

  if (error) throw error;
  return data;
}

// Sign out
export async function signOut() {
  console.log('[Supabase] Signing out...');
  const { error } = await supabase.auth.signOut();
  if (error) {
    console.error('[Supabase] Sign out error:', error);
    throw error;
  }
  console.log('[Supabase] Sign out successful');
}

// Update user profile
export async function updateUserProfile(updates: {
  displayName?: string;
  avatarUrl?: string;
}) {
  const { data, error } = await supabase.auth.updateUser({
    data: updates,
  });

  if (error) throw error;
  return data;
}

// Reset password
export async function resetPassword(email: string) {
  const { error } = await supabase.auth.resetPasswordForEmail(email, {
    redirectTo: `${window.location.origin}/auth/reset-password`,
  });

  if (error) throw error;
}
