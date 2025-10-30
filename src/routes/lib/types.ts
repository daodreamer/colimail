// Type definitions for email client
// This file contains all TypeScript interfaces used throughout the application

export type AuthType = "basic" | "oauth2";

export interface AccountConfig {
  id: number;
  email: string;
  password?: string;
  imap_server: string;
  imap_port: number;
  smtp_server: string;
  smtp_port: number;
  auth_type?: AuthType;
  access_token?: string;
  refresh_token?: string;
  token_expires_at?: number;
}

export interface EmailHeader {
  uid: number;
  subject: string;
  from: string;
  to: string;
  cc?: string; // CC recipients
  date: string;
  timestamp: number; // Unix timestamp in seconds
  has_attachments?: boolean;
  seen?: boolean; // Read/unread status
  flagged?: boolean; // Starred/flagged status
}

export interface AttachmentInfo {
  id: number;
  filename: string;
  content_type: string;
  size: number;
}

export interface Attachment {
  id?: number;
  filename: string;
  content_type: string;
  size: number;
  data?: number[]; // Uint8Array as number array for JSON
}

export interface Folder {
  id: number | null;
  account_id: number;
  name: string; // Original IMAP folder name (for operations)
  display_name: string; // User-friendly display name
  delimiter: string | null;
  flags: string | null;
  is_local?: boolean; // True for local-only folders, False for remote IMAP folders
}

export interface IdleEvent {
  account_id: number;
  folder_name: string;
  event_type: {
    type: "NewMessages" | "Expunge" | "FlagsChanged" | "ConnectionLost";
    count?: number;
    uid?: number;
  };
}

export type DraftType = "compose" | "reply" | "forward";

export interface DraftAttachment {
  filename: string;
  content_type: string;
  data: number[]; // Vec<u8> as number array for JSON
}

export interface DraftListItem {
  id: number;
  account_id: number;
  to_addr: string;
  cc_addr: string;
  subject: string;
  draft_type: DraftType;
  created_at: number;
  updated_at: number;
}
