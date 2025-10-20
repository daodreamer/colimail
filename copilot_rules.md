# Copilot Collaboration Rules

1. **Language Policy**
   - Provide all assistant responses in Chinese.
   - When writing code or code comments, use English only.

2. **API and Library Usage**
   - Prefer the newest versions of project dependencies and referenced APIs.
   - If the required latest API details are not already known, request the user to supply the official documentation or examples.

3. **Static Checks After Edits**
   - After any code additions or modifications, run the appropriate static checks (e.g., `cargo fmt`, `cargo check`, `npm run lint`) to confirm there are no syntax errors before reporting back.
