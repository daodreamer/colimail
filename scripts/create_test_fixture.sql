-- P3 E2E Test Database Fixture
-- This SQL script creates a test database with pre-populated data for E2E testing
-- Usage: sqlite3 test-fixture.db < create_test_fixture.sql

-- Create tables (same schema as main app)
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    imap_server TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_server TEXT NOT NULL,
    smtp_port INTEGER NOT NULL,
    auth_type TEXT NOT NULL DEFAULT 'basic',
    display_name TEXT
);

CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    delimiter TEXT,
    flags TEXT,
    is_local INTEGER DEFAULT 0,
    UNIQUE(account_id, name),
    FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS emails (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    folder_name TEXT NOT NULL,
    uid INTEGER NOT NULL,
    subject TEXT NOT NULL,
    from_addr TEXT NOT NULL,
    to_addr TEXT NOT NULL,
    cc_addr TEXT,
    date TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    body TEXT,
    raw_headers TEXT,
    has_attachments INTEGER DEFAULT 0,
    flags TEXT,
    seen INTEGER DEFAULT 0,
    flagged INTEGER DEFAULT 0,
    synced_at INTEGER NOT NULL,
    UNIQUE(account_id, folder_name, uid),
    FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS attachments (
    id INTEGER PRIMARY KEY,
    email_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    data BLOB,
    FOREIGN KEY(email_id) REFERENCES emails(id) ON DELETE CASCADE
);

-- Insert test accounts
INSERT INTO accounts (id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)
VALUES
    (1, 'test1@gmail.com', 'imap.gmail.com', 993, 'smtp.gmail.com', 587, 'oauth2', 'Test User 1'),
    (2, 'test2@outlook.com', 'outlook.office365.com', 993, 'smtp.office365.com', 587, 'oauth2', 'Test User 2');

-- Insert test folders for Account 1 (Gmail)
INSERT INTO folders (id, account_id, name, display_name, delimiter, flags, is_local)
VALUES
    (1, 1, 'INBOX', 'Inbox', '/', '', 0),
    (2, 1, 'Sent', 'Sent', '/', '', 0),
    (3, 1, 'Drafts', 'Drafts', '/', '', 0),
    (4, 1, 'Trash', 'Trash', '/', '', 0),
    (5, 1, 'Spam', 'Spam', '/', '', 0),
    (6, 1, 'Archive', 'Archive', '/', '', 0);

-- Insert test folders for Account 2 (Outlook)
INSERT INTO folders (id, account_id, name, display_name, delimiter, flags, is_local)
VALUES
    (7, 2, 'INBOX', 'Inbox', '/', '', 0),
    (8, 2, 'Sent Items', 'Sent Items', '/', '', 0),
    (9, 2, 'Drafts', 'Drafts', '/', '', 0),
    (10, 2, 'Deleted Items', 'Deleted Items', '/', '', 0);

-- Insert 50 test emails for Account 1
-- Mix of read/unread, flagged/unflagged, with/without attachments
INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr, cc_addr, date, timestamp, body, has_attachments, flags, seen, flagged, synced_at)
VALUES
    -- INBOX emails (30 emails)
    (1, 'INBOX', 1, 'Welcome to Colimail!', 'welcome@colimail.com', 'test1@gmail.com', NULL, '2025-01-15 10:00:00', 1736935200, '<html><body><h1>Welcome!</h1><p>Thank you for using Colimail.</p></body></html>', 0, '\Seen', 1, 0, 1736935200),
    (1, 'INBOX', 2, 'Project Update - Q1 2025', 'manager@company.com', 'test1@gmail.com', 'team@company.com', '2025-01-16 09:30:00', 1737020400, '<html><body><p>Here is the Q1 project update...</p></body></html>', 1, '\Seen', 1, 1, 1737020400),
    (1, 'INBOX', 3, 'Meeting Reminder: Team Standup', 'calendar@company.com', 'test1@gmail.com', NULL, '2025-01-16 14:00:00', 1737036000, '<html><body><p>Reminder: Daily standup at 3 PM</p></body></html>', 0, '\Seen', 1, 0, 1737036000),
    (1, 'INBOX', 4, '📧 Newsletter: Tech Weekly', 'newsletter@techweekly.com', 'test1@gmail.com', NULL, '2025-01-17 08:00:00', 1737097200, '<html><body><h2>This Week in Tech</h2><ul><li>Article 1</li><li>Article 2</li></ul></body></html>', 0, '', 0, 0, 1737097200),
    (1, 'INBOX', 5, 'Invoice #12345 - January 2025', 'billing@service.com', 'test1@gmail.com', NULL, '2025-01-17 11:00:00', 1737108000, '<html><body><p>Your invoice for January is attached.</p></body></html>', 1, '', 0, 1, 1737108000),
    (1, 'INBOX', 6, 'Re: Code Review Request', 'developer@company.com', 'test1@gmail.com', NULL, '2025-01-18 10:15:00', 1737187700, '<html><body><p>I''ve reviewed your PR. Looks good!</p></body></html>', 0, '\Seen', 1, 0, 1737187700),
    (1, 'INBOX', 7, 'Important: Security Update Required', 'security@company.com', 'test1@gmail.com', 'it@company.com', '2025-01-18 13:45:00', 1737200700, '<html><body><p><strong>Action Required:</strong> Please update your password.</p></body></html>', 0, '', 0, 1, 1737200700),
    (1, 'INBOX', 8, 'Weekend Plans?', 'friend@email.com', 'test1@gmail.com', NULL, '2025-01-19 18:30:00', 1737304200, '<html><body><p>Hey! Want to grab coffee this weekend?</p></body></html>', 0, '', 0, 0, 1737304200),
    (1, 'INBOX', 9, 'GitHub: New issue opened in your repository', 'notifications@github.com', 'test1@gmail.com', NULL, '2025-01-20 09:00:00', 1737356400, '<html><body><p>A new issue has been opened: "Bug in login flow"</p></body></html>', 0, '\Seen', 1, 0, 1737356400),
    (1, 'INBOX', 10, 'Conference Registration Confirmation', 'events@conference.com', 'test1@gmail.com', NULL, '2025-01-20 14:20:00', 1737375600, '<html><body><p>Thank you for registering! Your ticket is attached.</p></body></html>', 1, '\Seen', 1, 0, 1737375600),
    (1, 'INBOX', 11, 'Performance Review - Action Items', 'hr@company.com', 'test1@gmail.com', 'manager@company.com', '2025-01-21 11:00:00', 1737450000, '<html><body><p>Following up on your performance review...</p></body></html>', 1, '', 0, 1, 1737450000),
    (1, 'INBOX', 12, 'Package Delivered', 'delivery@courier.com', 'test1@gmail.com', NULL, '2025-01-21 16:45:00', 1737470700, '<html><body><p>Your package has been delivered to your doorstep.</p></body></html>', 0, '\Seen', 1, 0, 1737470700),
    (1, 'INBOX', 13, 'Subscription Renewal Notice', 'subscriptions@service.com', 'test1@gmail.com', NULL, '2025-01-22 08:30:00', 1737527400, '<html><body><p>Your subscription will renew on Feb 1, 2025.</p></body></html>', 0, '', 0, 0, 1737527400),
    (1, 'INBOX', 14, 'Flight Booking Confirmation', 'bookings@airline.com', 'test1@gmail.com', NULL, '2025-01-22 12:00:00', 1737540000, '<html><body><p>Your flight to New York is confirmed. Booking ref: ABC123</p></body></html>', 1, '\Seen', 1, 1, 1737540000),
    (1, 'INBOX', 15, 'Question about your recent PR', 'colleague@company.com', 'test1@gmail.com', NULL, '2025-01-23 09:15:00', 1737603300, '<html><body><p>I have a question about the changes you made...</p></body></html>', 0, '', 0, 0, 1737603300),
    (1, 'INBOX', 16, 'Monthly Report - December 2024', 'analytics@company.com', 'test1@gmail.com', 'team@company.com', '2025-01-23 15:00:00', 1737624000, '<html><body><p>Attached is the monthly analytics report.</p></body></html>', 1, '\Seen', 1, 0, 1737624000),
    (1, 'INBOX', 17, 'Birthday Party Invitation', 'friend2@email.com', 'test1@gmail.com', NULL, '2025-01-24 10:00:00', 1737693600, '<html><body><p>You''re invited to my birthday party on Saturday!</p></body></html>', 0, '', 0, 1, 1737693600),
    (1, 'INBOX', 18, 'Docker Hub: New tag pushed', 'notifications@docker.com', 'test1@gmail.com', NULL, '2025-01-24 14:30:00', 1737709800, '<html><body><p>A new tag "v2.1.0" has been pushed to your repository.</p></body></html>', 0, '\Seen', 1, 0, 1737709800),
    (1, 'INBOX', 19, 'Survey: Help us improve our service', 'feedback@service.com', 'test1@gmail.com', NULL, '2025-01-25 09:00:00', 1737784800, '<html><body><p>We value your feedback! Please take 5 minutes to complete our survey.</p></body></html>', 0, '', 0, 0, 1737784800),
    (1, 'INBOX', 20, 'Reminder: Dentist Appointment Tomorrow', 'appointments@dentist.com', 'test1@gmail.com', NULL, '2025-01-25 18:00:00', 1737817200, '<html><body><p>This is a reminder about your appointment at 10 AM tomorrow.</p></body></html>', 0, '', 0, 1, 1737817200),
    (1, 'INBOX', 21, 'Bank Statement - January 2025', 'statements@bank.com', 'test1@gmail.com', NULL, '2025-01-26 07:00:00', 1737864000, '<html><body><p>Your monthly bank statement is now available.</p></body></html>', 1, '\Seen', 1, 0, 1737864000),
    (1, 'INBOX', 22, 'New comment on your blog post', 'blog@myblog.com', 'test1@gmail.com', NULL, '2025-01-26 11:30:00', 1737880200, '<html><body><p>Someone commented on "Getting Started with Rust"</p></body></html>', 0, '\Seen', 1, 0, 1737880200),
    (1, 'INBOX', 23, 'Electricity Bill - February 2025', 'billing@utility.com', 'test1@gmail.com', NULL, '2025-01-27 08:00:00', 1737954000, '<html><body><p>Your electricity bill for February is $85.50</p></body></html>', 1, '', 0, 0, 1737954000),
    (1, 'INBOX', 24, 'Webinar Recording Available', 'webinars@education.com', 'test1@gmail.com', NULL, '2025-01-27 13:00:00', 1737972000, '<html><body><p>The recording of yesterday''s webinar is now available.</p></body></html>', 0, '\Seen', 1, 0, 1737972000),
    (1, 'INBOX', 25, 'Test Email with Special Characters: 你好世界 🌍', 'test@test.com', 'test1@gmail.com', NULL, '2025-01-27 16:00:00', 1737982800, '<html><body><p>Testing UTF-8 encoding: 你好世界 Здравствуй мир مرحبا بالعالم</p></body></html>', 0, '', 0, 0, 1737982800),
    (1, 'INBOX', 26, 'Code deployment successful', 'ci-cd@company.com', 'test1@gmail.com', 'devops@company.com', '2025-01-28 09:00:00', 1738065600, '<html><body><p>Your code has been successfully deployed to production.</p></body></html>', 0, '\Seen', 1, 0, 1738065600),
    (1, 'INBOX', 27, 'LinkedIn: You have 5 new connection requests', 'notifications@linkedin.com', 'test1@gmail.com', NULL, '2025-01-28 12:00:00', 1738076400, '<html><body><p>You have pending connection requests from...</p></body></html>', 0, '', 0, 0, 1738076400),
    (1, 'INBOX', 28, 'Grocery Delivery Scheduled', 'delivery@grocery.com', 'test1@gmail.com', NULL, '2025-01-29 10:00:00', 1738144800, '<html><body><p>Your grocery delivery is scheduled for today between 2-4 PM.</p></body></html>', 0, '', 0, 0, 1738144800),
    (1, 'INBOX', 29, 'Tax Documents Ready', 'tax@accountant.com', 'test1@gmail.com', NULL, '2025-01-29 14:00:00', 1738159200, '<html><body><p>Your tax documents for 2024 are ready for review.</p></body></html>', 1, '', 0, 1, 1738159200),
    (1, 'INBOX', 30, 'System Maintenance Notice', 'admin@company.com', 'test1@gmail.com', 'all@company.com', '2025-01-30 08:00:00', 1738224000, '<html><body><p>Scheduled maintenance on Sunday 2 AM - 6 AM.</p></body></html>', 0, '\Seen', 1, 0, 1738224000),

    -- Sent folder emails (10 emails)
    (1, 'Sent', 31, 'Re: Project proposal', 'test1@gmail.com', 'client@client.com', NULL, '2025-01-15 15:00:00', 1736953200, '<html><body><p>Thank you for your proposal. I have some questions...</p></body></html>', 0, '\Seen', 1, 0, 1736953200),
    (1, 'Sent', 32, 'Meeting notes from today', 'test1@gmail.com', 'team@company.com', NULL, '2025-01-16 17:00:00', 1737046800, '<html><body><p>Here are the notes from today''s meeting...</p></body></html>', 1, '\Seen', 1, 0, 1737046800),
    (1, 'Sent', 33, 'Thank you for your help!', 'test1@gmail.com', 'colleague@company.com', NULL, '2025-01-18 11:00:00', 1737190800, '<html><body><p>I really appreciate your help with the bug fix.</p></body></html>', 0, '\Seen', 1, 0, 1737190800),
    (1, 'Sent', 34, 'Vacation request', 'test1@gmail.com', 'hr@company.com', 'manager@company.com', '2025-01-20 10:00:00', 1737360000, '<html><body><p>I would like to request vacation from Feb 10-17.</p></body></html>', 0, '\Seen', 1, 0, 1737360000),
    (1, 'Sent', 35, 'Feedback on new feature', 'test1@gmail.com', 'product@company.com', NULL, '2025-01-22 14:00:00', 1737547200, '<html><body><p>I tested the new feature and have some feedback...</p></body></html>', 0, '\Seen', 1, 0, 1737547200),
    (1, 'Sent', 36, 'RSVP: Conference attendance', 'test1@gmail.com', 'events@conference.com', NULL, '2025-01-23 09:00:00', 1737599400, '<html><body><p>I confirm my attendance at the conference.</p></body></html>', 0, '\Seen', 1, 0, 1737599400),
    (1, 'Sent', 37, 'Question about API documentation', 'test1@gmail.com', 'support@api-provider.com', NULL, '2025-01-25 11:00:00', 1737792000, '<html><body><p>I have a question about the authentication endpoint...</p></body></html>', 0, '\Seen', 1, 0, 1737792000),
    (1, 'Sent', 38, 'Fwd: Important announcement', 'test1@gmail.com', 'team@company.com', NULL, '2025-01-26 10:00:00', 1737874800, '<html><body><p>Forwarding this important announcement from management...</p></body></html>', 0, '\Seen', 1, 0, 1737874800),
    (1, 'Sent', 39, 'Bug report: Login issue', 'test1@gmail.com', 'support@company.com', NULL, '2025-01-28 15:00:00', 1738087200, '<html><body><p>I''m experiencing a login issue with the mobile app...</p></body></html>', 1, '\Seen', 1, 0, 1738087200),
    (1, 'Sent', 40, 'Happy Birthday!', 'test1@gmail.com', 'friend@email.com', NULL, '2025-01-29 09:00:00', 1738141200, '<html><body><p>🎉 Happy Birthday! Hope you have a wonderful day!</p></body></html>', 0, '\Seen', 1, 0, 1738141200),

    -- Drafts (5 emails)
    (1, 'Drafts', 41, 'Draft: Quarterly report', 'test1@gmail.com', 'boss@company.com', NULL, '2025-01-30 10:00:00', 1738231200, '<html><body><p>Draft of Q1 quarterly report...</p></body></html>', 0, '\Draft', 0, 0, 1738231200),
    (1, 'Drafts', 42, 'Draft: Newsletter idea', 'test1@gmail.com', 'marketing@company.com', NULL, '2025-01-30 11:00:00', 1738234800, '<html><body><p>Ideas for next month''s newsletter...</p></body></html>', 0, '\Draft', 0, 0, 1738234800),
    (1, 'Drafts', 43, 'Draft: Proposal response', 'test1@gmail.com', 'client@client.com', NULL, '2025-01-30 12:00:00', 1738238400, '<html><body><p>Response to the project proposal...</p></body></html>', 0, '\Draft', 0, 0, 1738238400),
    (1, 'Drafts', 44, 'Draft: Team announcement', 'test1@gmail.com', 'team@company.com', NULL, '2025-01-30 13:00:00', 1738242000, '<html><body><p>Announcement about new team structure...</p></body></html>', 0, '\Draft', 0, 0, 1738242000),
    (1, 'Drafts', 45, 'Draft: Training request', 'test1@gmail.com', 'hr@company.com', NULL, '2025-01-30 14:00:00', 1738245600, '<html><body><p>Request for training budget...</p></body></html>', 0, '\Draft', 0, 0, 1738245600),

    -- Trash (5 emails)
    (1, 'Trash', 46, 'Spam: You won the lottery!', 'spam@spam.com', 'test1@gmail.com', NULL, '2025-01-15 08:00:00', 1736928000, '<html><body><p>Congratulations! Click here to claim your prize...</p></body></html>', 0, '\Deleted', 1, 0, 1736928000),
    (1, 'Trash', 47, 'Old meeting notes', 'test1@gmail.com', 'team@company.com', NULL, '2025-01-10 10:00:00', 1736496000, '<html><body><p>Notes from January 10 meeting...</p></body></html>', 0, '\Deleted \Seen', 1, 0, 1736496000),
    (1, 'Trash', 48, 'Expired offer', 'deals@shopping.com', 'test1@gmail.com', NULL, '2025-01-05 12:00:00', 1736078400, '<html><body><p>50% off sale - expires Jan 15!</p></body></html>', 0, '\Deleted \Seen', 1, 0, 1736078400),
    (1, 'Trash', 49, 'Unsubscribe confirmation', 'noreply@newsletter.com', 'test1@gmail.com', NULL, '2025-01-20 14:00:00', 1737371400, '<html><body><p>You have been unsubscribed from our mailing list.</p></body></html>', 0, '\Deleted \Seen', 1, 0, 1737371400),
    (1, 'Trash', 50, 'Old password reset email', 'security@service.com', 'test1@gmail.com', NULL, '2025-01-12 16:00:00', 1736697600, '<html><body><p>Reset your password (link expires in 24 hours)</p></body></html>', 0, '\Deleted \Seen', 1, 0, 1736697600);

-- Insert 20 test emails for Account 2
INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr, cc_addr, date, timestamp, body, has_attachments, flags, seen, flagged, synced_at)
VALUES
    (2, 'INBOX', 1, 'Welcome to Outlook', 'welcome@outlook.com', 'test2@outlook.com', NULL, '2025-01-15 09:00:00', 1736931600, '<html><body><h1>Welcome to Outlook!</h1></body></html>', 0, '\Seen', 1, 0, 1736931600),
    (2, 'INBOX', 2, 'Team Meeting Agenda', 'manager@company.com', 'test2@outlook.com', 'team@company.com', '2025-01-16 10:00:00', 1737021600, '<html><body><p>Agenda for tomorrow''s meeting...</p></body></html>', 1, '', 0, 1, 1737021600),
    (2, 'INBOX', 3, 'Calendar Invite: 1:1 with Manager', 'calendar@company.com', 'test2@outlook.com', NULL, '2025-01-17 11:00:00', 1737111600, '<html><body><p>You''ve been invited to a meeting...</p></body></html>', 0, '\Seen', 1, 0, 1737111600),
    (2, 'INBOX', 4, 'Document Review Request', 'colleague@company.com', 'test2@outlook.com', NULL, '2025-01-18 14:00:00', 1737208800, '<html><body><p>Please review the attached document.</p></body></html>', 1, '', 0, 0, 1737208800),
    (2, 'INBOX', 5, 'Office365: Storage almost full', 'storage@microsoft.com', 'test2@outlook.com', NULL, '2025-01-20 09:00:00', 1737356400, '<html><body><p>Your OneDrive storage is 90% full.</p></body></html>', 0, '', 0, 1, 1737356400),
    (2, 'INBOX', 6, 'Microsoft Teams: New message', 'teams@microsoft.com', 'test2@outlook.com', NULL, '2025-01-22 10:00:00', 1737536400, '<html><body><p>You have a new message in Teams...</p></body></html>', 0, '\Seen', 1, 0, 1737536400),
    (2, 'INBOX', 7, 'Expense Report Approved', 'finance@company.com', 'test2@outlook.com', NULL, '2025-01-23 11:00:00', 1737626400, '<html><body><p>Your expense report has been approved.</p></body></html>', 0, '\Seen', 1, 0, 1737626400),
    (2, 'INBOX', 8, 'Training: Excel Advanced Course', 'training@company.com', 'test2@outlook.com', NULL, '2025-01-24 09:00:00', 1737702000, '<html><body><p>Sign up for advanced Excel training...</p></body></html>', 0, '', 0, 0, 1737702000),
    (2, 'INBOX', 9, 'IT: Password Expiring Soon', 'it@company.com', 'test2@outlook.com', NULL, '2025-01-25 10:00:00', 1737792000, '<html><body><p>Your password will expire in 7 days.</p></body></html>', 0, '', 0, 1, 1737792000),
    (2, 'INBOX', 10, 'New Feature Announcement', 'product@company.com', 'test2@outlook.com', 'all@company.com', '2025-01-26 14:00:00', 1737892800, '<html><body><p>We''re excited to announce a new feature...</p></body></html>', 1, '\Seen', 1, 0, 1737892800),
    (2, 'INBOX', 11, 'Survey: Employee Satisfaction', 'hr@company.com', 'test2@outlook.com', NULL, '2025-01-27 09:00:00', 1737954000, '<html><body><p>Please complete our annual survey...</p></body></html>', 0, '', 0, 0, 1737954000),
    (2, 'INBOX', 12, 'Reminder: Submit Timesheet', 'payroll@company.com', 'test2@outlook.com', NULL, '2025-01-28 16:00:00', 1738083600, '<html><body><p>Timesheet submission deadline is Friday.</p></body></html>', 0, '', 0, 1, 1738083600),
    (2, 'INBOX', 13, 'Project Milestone Reached', 'pm@company.com', 'test2@outlook.com', 'team@company.com', '2025-01-29 11:00:00', 1738148400, '<html><body><p>Congratulations! We''ve reached our Q1 milestone!</p></body></html>', 0, '\Seen', 1, 1, 1738148400),
    (2, 'INBOX', 14, 'Lunch & Learn: Cloud Security', 'events@company.com', 'test2@outlook.com', NULL, '2025-01-30 09:00:00', 1738224000, '<html><body><p>Join us for a lunch & learn session...</p></body></html>', 0, '', 0, 0, 1738224000),
    (2, 'INBOX', 15, 'Benefits Enrollment Reminder', 'benefits@company.com', 'test2@outlook.com', NULL, '2025-01-30 12:00:00', 1738234800, '<html><body><p>Open enrollment ends next week.</p></body></html>', 1, '', 0, 1, 1738234800),
    (2, 'Sent Items', 16, 'Re: Document review', 'test2@outlook.com', 'colleague@company.com', NULL, '2025-01-19 10:00:00', 1737277200, '<html><body><p>I''ve reviewed the document. Here are my comments...</p></body></html>', 0, '\Seen', 1, 0, 1737277200),
    (2, 'Sent Items', 17, 'Thank you!', 'test2@outlook.com', 'manager@company.com', NULL, '2025-01-21 14:00:00', 1737464400, '<html><body><p>Thank you for approving my request!</p></body></html>', 0, '\Seen', 1, 0, 1737464400),
    (2, 'Sent Items', 18, 'Timesheet submitted', 'test2@outlook.com', 'payroll@company.com', NULL, '2025-01-28 17:00:00', 1738087200, '<html><body><p>My timesheet for this week is submitted.</p></body></html>', 0, '\Seen', 1, 0, 1738087200),
    (2, 'Drafts', 19, 'Draft: Team update', 'test2@outlook.com', 'team@company.com', NULL, '2025-01-30 15:00:00', 1738249200, '<html><body><p>Update on our progress...</p></body></html>', 0, '\Draft', 0, 0, 1738249200),
    (2, 'Deleted Items', 20, 'Old notification', 'system@company.com', 'test2@outlook.com', NULL, '2025-01-10 10:00:00', 1736496000, '<html><body><p>System notification...</p></body></html>', 0, '\Deleted \Seen', 1, 0, 1736496000);

-- Insert some test attachments
INSERT INTO attachments (email_id, filename, content_type, size)
VALUES
    -- Email 2 attachment
    (2, 'Q1_Project_Update.pdf', 'application/pdf', 245680),
    -- Email 5 attachment
    (5, 'Invoice_12345.pdf', 'application/pdf', 45123),
    -- Email 10 attachment
    (10, 'ticket.pdf', 'application/pdf', 12456),
    -- Email 11 attachment
    (11, 'performance_review.docx', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', 78234),
    -- Email 14 attachment
    (14, 'flight_booking.pdf', 'application/pdf', 34567),
    -- Email 16 attachment
    (16, 'december_report.xlsx', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', 156789),
    -- Email 21 attachment
    (21, 'bank_statement_jan2025.pdf', 'application/pdf', 234567),
    -- Email 23 attachment
    (23, 'electricity_bill_feb2025.pdf', 'application/pdf', 23456),
    -- Email 29 attachment
    (29, 'tax_documents_2024.zip', 'application/zip', 567890),
    -- Email 32 attachment (sent email)
    (32, 'meeting_notes.txt', 'text/plain', 4567),
    -- Email 39 attachment
    (39, 'screenshot.png', 'image/png', 89456),
    -- Account 2 attachments
    (2, 'meeting_agenda.docx', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', 23456),
    (4, 'review_document.pdf', 'application/pdf', 156789),
    (10, 'feature_specs.pdf', 'application/pdf', 234567),
    (15, 'benefits_guide.pdf', 'application/pdf', 345678);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_emails_account_folder ON emails(account_id, folder_name);
CREATE INDEX IF NOT EXISTS idx_emails_timestamp ON emails(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_emails_seen ON emails(seen);
CREATE INDEX IF NOT EXISTS idx_emails_flagged ON emails(flagged);
CREATE INDEX IF NOT EXISTS idx_folders_account ON folders(account_id);

-- Show summary
SELECT
    'Database fixture created successfully!' as status,
    (SELECT COUNT(*) FROM accounts) as total_accounts,
    (SELECT COUNT(*) FROM folders) as total_folders,
    (SELECT COUNT(*) FROM emails) as total_emails,
    (SELECT COUNT(*) FROM attachments) as total_attachments;
