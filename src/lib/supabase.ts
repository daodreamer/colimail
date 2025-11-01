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
    // Handle OAuth redirect for desktop apps
    flowType: 'pkce',
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
  name?: string;
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
    // Try multiple possible name fields for better compatibility
    name: session.user.user_metadata?.name ||
          session.user.user_metadata?.full_name ||
          session.user.user_metadata?.display_name ||
          session.user.email?.split('@')[0],
    avatarUrl: session.user.user_metadata?.avatar_url,
    subscriptionTier: session.user.user_metadata?.subscription_tier || 'free',
    subscriptionExpiresAt: session.user.user_metadata?.subscription_expires_at,
    createdAt: session.user.created_at,
  };
}

// Sign up with email and password
export async function signUpWithEmail(email: string, password: string, name: string) {
  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      data: {
        name: name,
        full_name: name, // For Supabase Dashboard compatibility
        display_name: name, // For Supabase Dashboard Display Name column
        subscription_tier: 'free',
      },
      // Redirect to verification page after email confirmation
      emailRedirectTo: typeof window !== 'undefined'
        ? `${window.location.origin}/auth/verify`
        : undefined,
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
      // Explicitly request email scope for Google Suite accounts
      scopes: 'https://www.googleapis.com/auth/userinfo.email',
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
  name?: string;
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

// Validate password strength (matches Supabase requirements)
export function validatePassword(password: string): { valid: boolean; message: string } {
  // Minimum length check
  if (password.length < 8) {
    return { valid: false, message: 'Password must be at least 8 characters long' };
  }

  // Maximum length check (PostgreSQL limit)
  if (password.length > 72) {
    return { valid: false, message: 'Password must be less than 72 characters' };
  }

  // Check for at least one lowercase letter
  if (!/[a-z]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one lowercase letter' };
  }

  // Check for at least one uppercase letter
  if (!/[A-Z]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one uppercase letter' };
  }

  // Check for at least one digit
  if (!/\d/.test(password)) {
    return { valid: false, message: 'Password must contain at least one digit' };
  }

  // Check for at least one symbol (special character)
  if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one symbol (!@#$%^&*...)' };
  }

  return { valid: true, message: 'Password is strong' };
}

// Helper function to get user-friendly error messages
export function getAuthErrorMessage(error: any): string {
  const errorMessage = error?.message || '';

  // Map common Supabase auth errors to user-friendly messages
  if (errorMessage.includes('Invalid login credentials')) {
    return 'Invalid email or password. Please try again.';
  }
  if (errorMessage.includes('Email not confirmed')) {
    return 'Please verify your email address before signing in.';
  }
  if (errorMessage.includes('User already registered')) {
    return 'An account with this email already exists. Please sign in instead.';
  }
  if (errorMessage.includes('Password should be at least')) {
    return 'Password must be at least 6 characters long.';
  }
  if (errorMessage.includes('Unable to validate email address')) {
    return 'Please enter a valid email address.';
  }
  if (errorMessage.includes('Email rate limit exceeded')) {
    return 'Too many requests. Please wait a few minutes before trying again.';
  }
  if (errorMessage.includes('Invalid refresh token')) {
    return 'Your session has expired. Please sign in again.';
  }
  if (errorMessage.includes('network')) {
    return 'Network error. Please check your internet connection.';
  }

  // Return the original error message if no match found
  return errorMessage || 'An unexpected error occurred. Please try again.';
}
