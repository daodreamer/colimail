// RFC 2047 encoding/decoding utilities and email parsing helpers
// This module handles character encoding conversions for email headers

use chrono::{DateTime, Utc};
use encoding_rs::Encoding;
use std::collections::HashMap;

/// Helper function to decode RFC 2047 encoded words (e.g., "=?UTF-8?Q?...?=")
/// RFC 2047 format: =?charset?encoding?encoded-text?=
/// where encoding can be Q (Quoted-Printable) or B (Base64)
pub fn decode_header(encoded: &str) -> String {
    // Check if the string contains RFC 2047 encoded words
    if !encoded.contains("=?") {
        return encoded.to_string();
    }

    let mut result = String::new();
    let mut remaining = encoded;

    while let Some(start_pos) = remaining.find("=?") {
        // Add any text before the encoded word
        result.push_str(&remaining[..start_pos]);

        // RFC 2047 format: =?charset?encoding?encoded-text?=
        // Parse step by step to avoid finding ? or = within the encoded content
        let after_start = &remaining[start_pos + 2..];

        // Find the first ? (end of charset)
        if let Some(charset_end) = after_start.find('?') {
            let charset = &after_start[..charset_end];
            let after_charset = &after_start[charset_end + 1..];

            // Find the second ? (end of encoding)
            if let Some(encoding_end) = after_charset.find('?') {
                let encoding = &after_charset[..encoding_end];
                let after_encoding = &after_charset[encoding_end + 1..];

                // Find ?= (end of encoded text)
                if let Some(text_end) = after_encoding.find("?=") {
                    let encoded_text = &after_encoding[..text_end];

                    // Calculate the full length of the encoded word
                    let full_length =
                        2 + charset.len() + 1 + encoding.len() + 1 + encoded_text.len() + 2;
                    let full_encoded = &remaining[start_pos..start_pos + full_length];

                    let encoding_upper = encoding.to_uppercase();

                    let decoded = match encoding_upper.as_str() {
                        "Q" => decode_quoted_printable(encoded_text),
                        "B" => decode_base64(encoded_text),
                        _ => None,
                    };

                    if let Some(decoded_bytes) = decoded {
                        // Convert bytes to string using the specified charset
                        // Use encoding_rs to handle various character encodings
                        let encoding = Encoding::for_label(charset.as_bytes());

                        let decoded_str = if let Some(enc) = encoding {
                            // Use encoding_rs to decode
                            let (cow, _encoding_used, _had_errors) = enc.decode(&decoded_bytes);
                            Some(cow.into_owned())
                        } else {
                            // If encoding not recognized, try UTF-8 as fallback
                            String::from_utf8(decoded_bytes).ok()
                        };

                        if let Some(s) = decoded_str {
                            result.push_str(&s);
                        } else {
                            // Decoding failed, keep original
                            result.push_str(full_encoded);
                        }
                    } else {
                        // Decoding failed, keep original
                        result.push_str(full_encoded);
                    }

                    // Move past the encoded word
                    remaining = &remaining[start_pos + full_length..];

                    // RFC 2047: whitespace between encoded words should be ignored
                    if remaining.starts_with(' ')
                        && remaining.len() > 1
                        && remaining[1..].starts_with("=?")
                    {
                        remaining = &remaining[1..];
                    }
                } else {
                    // No ?= found, not a valid encoded word
                    result.push_str(&remaining[start_pos..start_pos + 2]);
                    remaining = &remaining[start_pos + 2..];
                }
            } else {
                // No second ? found
                result.push_str(&remaining[start_pos..start_pos + 2]);
                remaining = &remaining[start_pos + 2..];
            }
        } else {
            // No first ? found
            result.push_str(&remaining[start_pos..start_pos + 2]);
            remaining = &remaining[start_pos + 2..];
        }
    }

    // Add any remaining text
    result.push_str(remaining);
    result
}

/// Decode Quoted-Printable (Q encoding) for RFC 2047
fn decode_quoted_printable(encoded: &str) -> Option<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut chars = encoded.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '_' => decoded.push(b' '), // underscore represents space in Q encoding
            '=' => {
                // Get next two hex digits
                if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                        decoded.push(byte);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ if ch.is_ascii() => decoded.push(ch as u8),
            _ => return None, // Non-ASCII in Q encoding is invalid
        }
    }

    Some(decoded)
}

/// Decode Base64 (B encoding) for RFC 2047
fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
    // Simple base64 decoding
    let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut char_map = HashMap::new();
    for (i, c) in b64_chars.chars().enumerate() {
        char_map.insert(c, i as u8);
    }

    let mut decoded = Vec::new();
    let chars: Vec<char> = encoded.chars().filter(|c| !c.is_whitespace()).collect();

    for chunk in chars.chunks(4) {
        if chunk.len() < 2 {
            return None;
        }

        let b1 = char_map.get(&chunk[0])?;
        let b2 = char_map.get(&chunk[1])?;

        decoded.push((b1 << 2) | (b2 >> 4));

        if chunk.len() > 2 && chunk[2] != '=' {
            let b3 = char_map.get(&chunk[2])?;
            decoded.push(((b2 & 0x0F) << 4) | (b3 >> 2));

            if chunk.len() > 3 && chunk[3] != '=' {
                let b4 = char_map.get(&chunk[3])?;
                decoded.push(((b3 & 0x03) << 6) | b4);
            }
        }
    }

    Some(decoded)
}

/// Helper function to safely decode bytes to UTF-8 string
/// Handles both raw UTF-8 and potential encoding issues with emoji
pub fn decode_bytes_to_string(bytes: &[u8]) -> String {
    // First try to parse as valid UTF-8
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // If invalid UTF-8, use lossy conversion but be more careful
            String::from_utf8_lossy(bytes).to_string()
        }
    }
}

/// Parse RFC 2822 date string to Unix timestamp
/// Email dates are in format like: "Mon, 15 Jan 2024 14:30:00 +0800"
/// If the Date header cannot be parsed and an INTERNALDATE is provided, use it instead
#[allow(dead_code)]
pub fn parse_email_date(date_str: &str) -> i64 {
    parse_email_date_with_fallback(date_str, None)
}

/// Parse email date with optional INTERNALDATE fallback
/// INTERNALDATE is the server's received time (more reliable than Date header)
pub fn parse_email_date_with_fallback(date_str: &str, internaldate: Option<&str>) -> i64 {
    // Try to parse the RFC 2822 format date
    if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
        return dt.timestamp();
    }

    // Try alternative RFC 3339 format (ISO 8601)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return dt.timestamp();
    }

    // If parsing fails, try to extract timestamp from various formats
    // Some servers might send non-standard date formats
    if let Ok(dt) = chrono::DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S %z") {
        return dt.timestamp();
    }

    // If Date header parsing failed, try to use INTERNALDATE as fallback
    if let Some(internal_date_str) = internaldate {
        // Only log if the Date header was not just empty
        // "(No Date)" means the email didn't have a Date header, which is common
        if date_str != "(No Date)" {
            eprintln!("⚠️ Failed to parse Date header: {}", date_str);
            eprintln!("   Using INTERNALDATE as fallback: {}", internal_date_str);
        }

        // Try to parse INTERNALDATE (also RFC 2822 format)
        if let Ok(dt) = DateTime::parse_from_rfc2822(internal_date_str) {
            return dt.timestamp();
        }

        // Only log INTERNALDATE parsing failure if it actually fails
        eprintln!(
            "⚠️ Failed to parse both Date ('{}') and INTERNALDATE ('{}')",
            date_str, internal_date_str
        );
    } else if date_str != "(No Date)" {
        // Only log if we had a Date header that failed to parse and no INTERNALDATE
        eprintln!("⚠️ Failed to parse Date header: {}", date_str);
    }

    // If all parsing fails, return current timestamp as last resort fallback
    eprintln!("⚠️ Using current time as fallback for date parsing");
    Utc::now().timestamp()
}

/// Helper function to check if a BODYSTRUCTURE contains attachments
/// We use Debug format as a simple way to detect attachment keywords
pub fn check_for_attachments<T: std::fmt::Debug>(body: &T) -> bool {
    let debug_str = format!("{:?}", body);
    let lower = debug_str.to_lowercase();
    // Check for common attachment indicators in BODYSTRUCTURE
    lower.contains("attachment") || lower.contains("filename")
}
