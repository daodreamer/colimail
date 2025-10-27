// Email provider presets for common email services
// These presets do not include OAuth2 providers (Gmail, Outlook)

export interface EmailProviderPreset {
  name: string;
  value: string;
  imap_server: string;
  imap_port: number;
  smtp_server: string;
  smtp_port: number;
  description?: string;
}

export const EMAIL_PROVIDER_PRESETS: EmailProviderPreset[] = [
  {
    name: "Yahoo Mail",
    value: "yahoo",
    imap_server: "imap.mail.yahoo.com",
    imap_port: 993,
    smtp_server: "smtp.mail.yahoo.com",
    smtp_port: 465,
    description: "Yahoo Mail (requires app password)"
  },
  {
    name: "iCloud Mail",
    value: "icloud",
    imap_server: "imap.mail.me.com",
    imap_port: 993,
    smtp_server: "smtp.mail.me.com",
    smtp_port: 587,
    description: "Apple iCloud Mail (requires app-specific password)"
  },
  {
    name: "Zoho Mail",
    value: "zoho",
    imap_server: "imap.zoho.com",
    imap_port: 993,
    smtp_server: "smtp.zoho.com",
    smtp_port: 465,
    description: "Zoho Mail"
  },
  {
    name: "ProtonMail Bridge",
    value: "protonmail",
    imap_server: "127.0.0.1",
    imap_port: 1143,
    smtp_server: "127.0.0.1",
    smtp_port: 1025,
    description: "ProtonMail (requires ProtonMail Bridge)"
  },
  {
    name: "Fastmail",
    value: "fastmail",
    imap_server: "imap.fastmail.com",
    imap_port: 993,
    smtp_server: "smtp.fastmail.com",
    smtp_port: 465,
    description: "Fastmail (requires app password)"
  },
  {
    name: "GMX Mail (Germany)",
    value: "gmx",
    imap_server: "imap.gmx.net",
    imap_port: 993,
    smtp_server: "smtp.gmx.net",
    smtp_port: 587,
    description: "GMX Mail (Germany)"
  },
  {
    name: "Mail.com",
    value: "mailcom",
    imap_server: "imap.mail.com",
    imap_port: 993,
    smtp_server: "smtp.mail.com",
    smtp_port: 465,
    description: "Mail.com"
  },
  {
    name: "AOL Mail",
    value: "aol",
    imap_server: "imap.aol.com",
    imap_port: 993,
    smtp_server: "smtp.aol.com",
    smtp_port: 465,
    description: "AOL Mail (requires app password)"
  },
  {
    name: "QQ Mail (腾讯邮箱)",
    value: "qq",
    imap_server: "imap.qq.com",
    imap_port: 993,
    smtp_server: "smtp.qq.com",
    smtp_port: 465,
    description: "QQ Mail (requires authorization code)"
  },
  {
    name: "163 Mail (网易邮箱)",
    value: "163",
    imap_server: "imap.163.com",
    imap_port: 993,
    smtp_server: "smtp.163.com",
    smtp_port: 465,
    description: "163 Mail (requires authorization code)"
  },
  {
    name: "126 Mail (网易126邮箱)",
    value: "126",
    imap_server: "imap.126.com",
    imap_port: 993,
    smtp_server: "smtp.126.com",
    smtp_port: 465,
    description: "126 Mail (requires authorization code)"
  },
  {
    name: "Sina Mail (新浪邮箱)",
    value: "sina",
    imap_server: "imap.sina.com",
    imap_port: 993,
    smtp_server: "smtp.sina.com",
    smtp_port: 465,
    description: "Sina Mail"
  },
  {
    name: "Custom (Manual Setup)",
    value: "custom",
    imap_server: "",
    imap_port: 993,
    smtp_server: "",
    smtp_port: 465,
    description: "Configure your own email server settings"
  }
];
