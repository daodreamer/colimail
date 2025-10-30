// Email parsing from IMAP fetch results
// This module handles parsing IMAP FETCH responses into EmailHeader structs

use crate::commands::emails::codec::{
    check_for_attachments, decode_bytes_to_string, decode_header, parse_email_date_with_fallback,
};
use crate::models::EmailHeader;

/// Helper function to parse IMAP fetch results into EmailHeader
/// In imap 3.0.0, Fetch type requires lifetime parameter
pub fn parse_email_headers<'a, I>(messages: I) -> Vec<EmailHeader>
where
    I: Iterator<Item = &'a imap::types::Fetch<'a>>,
{
    let mut headers = Vec::new();

    for msg in messages {
        let envelope = match msg.envelope() {
            Some(e) => e,
            None => continue,
        };

        let subject = envelope
            .subject
            .as_ref()
            .map(|s| {
                // In imap 3.0.0, envelope fields are Cow<[u8]> instead of &[u8]
                let raw_subject = decode_bytes_to_string(s.as_ref());
                decode_header(&raw_subject)
            })
            .unwrap_or_else(|| "(No Subject)".to_string());

        let from = envelope
            .from
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(Unknown Sender)".to_string());

        let date = envelope
            .date
            .as_ref()
            .map(|d| decode_bytes_to_string(d.as_ref()))
            .unwrap_or_else(|| "(No Date)".to_string());

        // Get INTERNALDATE as a fallback for date parsing
        let internal_date = msg
            .internal_date()
            .map(|d| format!("{}", d.format("%a, %d %b %Y %H:%M:%S %z")));

        // Use INTERNALDATE as fallback if Date header parsing fails
        let timestamp = parse_email_date_with_fallback(&date, internal_date.as_deref());

        let to = envelope
            .to
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(Unknown Recipient)".to_string());

        let cc = envelope
            .cc
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "".to_string());

        // Check if email has attachments by examining BODYSTRUCTURE
        let has_attachments = msg
            .bodystructure()
            .map(check_for_attachments)
            .unwrap_or(false);

        // Check if email has been read by examining FLAGS
        let seen = msg
            .flags()
            .iter()
            .any(|flag| matches!(flag, imap::types::Flag::Seen));

        // Check if email has been flagged/starred by examining FLAGS
        let flagged = msg
            .flags()
            .iter()
            .any(|flag| matches!(flag, imap::types::Flag::Flagged));

        headers.push(EmailHeader {
            uid: msg.uid.unwrap_or(0),
            subject,
            from,
            to,
            cc,
            date,
            timestamp,
            has_attachments,
            seen,
            flagged,
        });
    }

    // Note: Sorting is now done by the caller
    headers
}
