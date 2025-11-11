// CMVH Verifier - Frontend verification logic

import { invoke } from "@tauri-apps/api/core";
import type {
  CMVHHeaders,
  EmailContent,
  VerificationResult,
  VerificationState,
} from "./types";

/**
 * Check if email has CMVH headers
 */
export async function hasCMVHHeaders(rawHeaders: string): Promise<boolean> {
  try {
    return await invoke<boolean>("has_cmvh_headers", { rawHeaders });
  } catch (error) {
    console.error("Failed to check CMVH headers:", error);
    return false;
  }
}

/**
 * Parse CMVH headers from raw email headers
 */
export async function parseCMVHHeaders(
  rawHeaders: string
): Promise<CMVHHeaders | null> {
  try {
    return await invoke<CMVHHeaders>("parse_email_cmvh_headers", {
      rawHeaders,
    });
  } catch (error) {
    console.error("Failed to parse CMVH headers:", error);
    return null;
  }
}

/**
 * Verify CMVH signature locally (fast, no blockchain required)
 */
export async function verifyCMVHLocally(
  headers: CMVHHeaders,
  content: EmailContent
): Promise<VerificationResult> {
  try {
    return await invoke<VerificationResult>("verify_cmvh_signature", {
      headers,
      content,
    });
  } catch (error) {
    console.error("Failed to verify CMVH signature:", error);
    return {
      is_valid: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Hash email content (for debugging)
 */
export async function hashEmailContent(
  content: EmailContent
): Promise<string | null> {
  try {
    return await invoke<string>("hash_email_content", { content });
  } catch (error) {
    console.error("Failed to hash email content:", error);
    return null;
  }
}

/**
 * Verify email with CMVH (complete flow)
 */
export async function verifyEmail(
  rawHeaders: string,
  content: EmailContent
): Promise<VerificationState> {
  try {
    // Check if email has CMVH headers
    const hasCMVH = await hasCMVHHeaders(rawHeaders);
    if (!hasCMVH) {
      return {
        status: "idle",
      };
    }

    // Parse headers
    const headers = await parseCMVHHeaders(rawHeaders);
    if (!headers) {
      return {
        status: "error",
        error: "Failed to parse CMVH headers",
      };
    }

    // Verify signature locally
    const result = await verifyCMVHLocally(headers, content);

    return {
      status: result.is_valid ? "verified-local" : "invalid",
      result,
      error: result.error,
    };
  } catch (error) {
    console.error("Failed to verify email:", error);
    return {
      status: "error",
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
