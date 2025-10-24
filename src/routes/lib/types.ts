// Type definitions for email client
// This file contains all TypeScript interfaces used throughout the application

export interface AccountConfig {
  id: number;
  email: string;
  password: string;
  imap_server: string;
  imap_port: number;
  smtp_server: string;
  smtp_port: number;
}

export interface EmailHeader {
  uid: number;
  subject: string;
  from: string;
  to: string;
  date: string;
  timestamp: number; // Unix timestamp in seconds
  has_attachments?: boolean;
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
