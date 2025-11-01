import { createClient, type Session } from '@supabase/supabase-js';

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
    // In Tauri, we manually handle OAuth via deep links and event listeners
    // so we should disable automatic URL detection to avoid conflicts
    detectSessionInUrl: typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window),
    // Handle OAuth redirect for desktop apps
    flowType: 'pkce',
    storage: {
      getItem: async (key: string) => {
        console.log(`[Supabase Storage] getItem called for key: ${key}`);
        // Try localStorage first (more reliable for large session tokens)
        if (typeof window !== 'undefined' && window.localStorage) {
          try {
            const value = localStorage.getItem(key);
            if (value) {
              console.log(`[Supabase Storage] Found ${key} in localStorage (${value.length} chars)`);
              return value;
            }
            console.log(`[Supabase Storage] ${key} not found in localStorage`);
          } catch (error) {
            console.error('Failed to read from localStorage:', error);
          }
        }

        // Fallback to Tauri secure storage for smaller items
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          console.log(`[Supabase Storage] Trying Tauri secure storage for ${key}...`);
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            const result = await invoke<string>('get_secure_storage', { key });
            console.log(`[Supabase Storage] Tauri secure storage result for ${key}:`, result ? `Found (${result.length} chars)` : 'Not found');
            return result;
          } catch (error) {
            // Silently fail, item might not exist
            console.log(`[Supabase Storage] Tauri secure storage failed for ${key}:`, error);
            return null;
          }
        }

        console.log(`[Supabase Storage] No storage available for ${key}`);
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
  console.log('[Supabase] getCurrentSession: Calling supabase.auth.getSession()...');
  try {
    const { data, error } = await supabase.auth.getSession();
    console.log('[Supabase] getCurrentSession: getSession() returned', { hasSession: !!data.session, error });
    if (error) {
      console.error('Error getting session:', error);
      return null;
    }
    return data.session;
  } catch (err) {
    console.error('[Supabase] getCurrentSession: Exception caught:', err);
    return null;
  }
}

// Get current user
export async function getCurrentUser(existingSession?: Session | null): Promise<AppUser | null> {
  console.log('[Supabase] getCurrentUser: Starting...', existingSession ? 'with existing session' : 'will fetch session');
  
  // Use provided session or fetch from storage
  const session = existingSession ?? await getCurrentSession();
  console.log('[Supabase] getCurrentUser: Got session:', !!session);
  
  if (!session?.user) {
    console.log('[Supabase] getCurrentUser: No session or user found');
    return null;
  }

  console.log('[Supabase] getCurrentUser: Found user', {
    id: session.user.id,
    email: session.user.email,
    metadata: session.user.user_metadata
  });

  const user = {
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
  
  console.log('[Supabase] getCurrentUser: Returning user:', user);
  return user;
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
      // Always use production URL for both development and deployed apps
      emailRedirectTo: 'https://www.colimail.net/auth/verify',
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
  // Check if running in Tauri (desktop app)
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: 'google',
    options: {
      // Always use HTTP callback URL (your deployed website)
      // This page will handle triggering the deep link for desktop app
      redirectTo: 'https://www.colimail.net/auth/callback',
      scopes: 'https://www.googleapis.com/auth/userinfo.email',
      queryParams: {
        access_type: 'offline',
        prompt: 'consent',
      },
      // CRITICAL: In Tauri, skip browser redirect since we manually handle via deep link
      // This prevents Supabase from trying to handle the callback in the desktop app
      skipBrowserRedirect: isTauri,
    },
  });

  if (error) throw error;
  return data;
}

// Exchange authorization code for session (OAuth PKCE flow)
export async function exchangeCodeForSession(code: string) {
  console.log('[Supabase] Exchanging authorization code for session...');
  
  const { data, error } = await supabase.auth.exchangeCodeForSession(code);
  
  if (error) {
    console.error('[Supabase] Code exchange error:', error);
    throw error;
  }
  
  console.log('[Supabase] Code exchange successful:', {
    hasSession: !!data.session,
    hasUser: !!data.user,
    userId: data.user?.id
  });
  
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
