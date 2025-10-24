// Utility functions for email client
// This file contains helper functions for formatting and validation

/**
 * Format file size in bytes to human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1024 * 1024 * 1024)
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + " GB";
}

/**
 * Format time since last event (e.g., "5m ago")
 */
export function formatTimeSince(
  timestamp: number,
  currentTime: number
): string {
  const seconds = currentTime - timestamp;

  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

/**
 * Format timestamp to local time (compact for list view)
 * Shows "HH:MM" for today, "Yesterday HH:MM" for yesterday, etc.
 */
export function formatLocalDateTime(timestamp: number): string {
  const date = new Date(timestamp * 1000); // Convert seconds to milliseconds
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const emailDate = new Date(
    date.getFullYear(),
    date.getMonth(),
    date.getDate()
  );

  // Format time as HH:MM
  const timeStr = date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });

  // If today, show time only
  if (emailDate.getTime() === today.getTime()) {
    return timeStr;
  }

  // If yesterday, show "Yesterday HH:MM"
  if (emailDate.getTime() === yesterday.getTime()) {
    return `Yesterday ${timeStr}`;
  }

  // If within this year, show "MMM DD HH:MM"
  if (date.getFullYear() === now.getFullYear()) {
    const monthDay = date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    });
    return `${monthDay} ${timeStr}`;
  }

  // Otherwise show full date "MMM DD, YYYY HH:MM"
  return (
    date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    }) + ` ${timeStr}`
  );
}

/**
 * Format timestamp to full local time (for detail view)
 */
export function formatFullLocalDateTime(timestamp: number): string {
  const date = new Date(timestamp * 1000); // Convert seconds to milliseconds

  // Format: "Day, Month DD, YYYY at HH:MM:SS"
  return (
    date.toLocaleDateString(undefined, {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    }) +
    " at " +
    date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    })
  );
}

/**
 * Check if current folder is a trash/deleted folder
 */
export function isTrashFolder(folderName: string): boolean {
  const lowerName = folderName.toLowerCase();
  return (
    lowerName.includes("trash") ||
    lowerName.includes("deleted") ||
    lowerName.includes("垃圾") ||
    lowerName.includes("bin")
  );
}
