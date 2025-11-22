# Authentication System Documentation

This document describes the authentication and authorization system for Root backend.

| Section | Description |
|--------|-------------|
| [Overview](#overview) | Summary of supported authentication methods |
| [Roles and Permissions](#roles-and-permissions) | Roles (Admin, Member, Bot) and protected mutations |
| [Setup](#setup) | How to configure OAuth, env vars, and database migration |
| [OAuth Flow](#usage) | Full OAuth login flow and response structure |
| [Bot Management](#bot-management) | Bot creation and API key handling |
| [Role Based Access Control (RBAC)](#permission-checking-in-code) | GraphQL guards and access control |
| [Troubleshooting](#troubleshooting) | Common issues and how to fix them |
| [API Reference](#api-reference) | GraphQL mutations and their inputs/outputs |
| [Example](#example) | Complete OAuth authentication flow example |
| [Architecture Notes](#architecture-notes) | Middleware flow, bot member structure, key management |



## Overview

The authentication system supports three types of authentication:
1. **GitHub OAuth** - For human members who are part of the amfoss GitHub organization
2. **API Keys** - For headless bots and automated services
3. **Session Tokens** - For maintaining logged-in state after OAuth

## Roles and Permissions

### Roles

- **Admin**: Full system access, can create/manage bots, access all mutations and queries
- **Member**: Standard user permissions, authenticated via GitHub OAuth, limited access
- **Bot**: Service accounts with API key authentication, can access protected mutations

Note that unauthenticated users have essentially no access to the system.

### Protected Mutations

The following mutations require Admin or Bot role:
- `markAttendance`
- `markStatusUpdate`
- `createStatusBreak`

Regular Members cannot access these mutations.

## Setup

### 1. GitHub OAuth Application

Create a GitHub OAuth application at: https://github.com/settings/developers

**Settings:**
- Application name: Root Backend (or your choice)
- Homepage URL: `http://localhost:3000` (or your domain)
- Authorization callback URL: `http://localhost:5000/auth/github/callback`

After creating, note down the **Client ID** and **Client Secret**.

### 2. Environment Variables

Add the following to your `.env` file:

```bash
# GitHub OAuth credentials
GITHUB_CLIENT_ID=your_client_id_here
GITHUB_CLIENT_SECRET=your_client_secret_here
GITHUB_REDIRECT_URL=http://localhost:5000/auth/github/callback # Oauth Callback
FRONTEND_URL=http://localhost:3000/dashboard # Redirect after OAuth
GITHUB_ORG_NAME=amfoss  # Organization that users must be part of
```

### 3. Database Migration

Run the authentication system migration:

```bash
sqlx migrate run
```

This creates the following tables:
- `Member` - Stores authenticated members with their details
- `Sessions` - Stores session tokens
- `ApiKeys` - Stores hashed API keys for bots

### 4. Making Your First Admin

After setting up the system, you need to create the first admin user manually:

1. Register via GitHub OAuth (see below)
2. Connect to the database:
   ```bash
   psql $ROOT_DB_URL
   ```
3. Promote your user to admin:
   ```sql
   UPDATE Member SET role = 'Admin' WHERE email = 'your@email.com';
   ```
4. Verify:
   ```sql
   SELECT member_id, name, email, role FROM Member WHERE email = 'your@email.com';
   ```

Now you can create bots and manage the system!
## Usage

### Member Registration & Login (GitHub OAuth)

The OAuth flow is now handled entirely by the backend, simplifying the frontend implementation.

1.  **Initiate Login**: The user visits `http://localhost:5000/auth/github`.
2.  **GitHub Authorization**: The user is redirected to GitHub to authorize the application.
3.  **Backend Callback**: After authorization, GitHub redirects the user to the backend's callback URL: `http://localhost:5000/auth/github/callback`.
4.  **Session Creation**: The backend exchanges the OAuth code for an access token, fetches user info, and either registers a new member or logs in an existing one. A session is created for the user.
5.  **Cookie and Redirect**: The backend sets a secure, HTTP-only `session_token` cookie in the user's browser and redirects them to the `FRONTEND_URL` specified in your `.env` file.
6.  **Authenticated State**: The user is now logged in. The browser will automatically send the session cookie with all subsequent requests to the backend API.

**Important:**
- First time users are automatically registered
- Users must be members of the amfoss GitHub organization
- Non-members receive an error

### Making Authenticated Requests

**For Members (Browser):**
After logging in via GitHub OAuth, the browser automatically handles authentication by sending the `session_token` cookie with every request. No manual header management is needed in the frontend code.

**For Bots (API Keys):**
Bots must include their API key in the `Authorization` header:
```
Authorization: Bearer <api_key>
```

Example with curl (simulating a bot request):
```bash
curl -X POST http://localhost:5000/ \
  -H "Authorization: Bearer root_wnTK5uRq8FECFSvSC8OVZ8h0SSJefTMlvGWJmsS4" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ member(memberId: 1) { name email } }"
  }'
```

## Bot Management
### Creating Bots (Admin Only)

Admins can create bot accounts with API keys:

```graphql
mutation {
  createBot(name: "Presence Bot")
}
```

**Response:**
```json
{
  "data": {
    "createBot": {
      "apiKey": "root_wnTK5uRq8FECFSvSC8OVZ8h0SSJefTMlvGWJmsS4"
    }
  }
}
```

**⚠️ Important:** The API key is only returned ONCE. Store it securely!

### Using API Keys (Bots)

Bots use API keys instead of session tokens. Include the API key in the `Authorization` header in the same format as before.


## Permission Checking in Code

### In GraphQL Resolvers

Protected mutations use guards:

```rust
use crate::auth::guards::{AdminGuard, AdminOrBotGuard, AuthGuard};

#[Object]
impl SomeMutations {
    #[graphql(name = "protectedMutation", guard = "AdminOrBotGuard")]
    async fn protected_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Only admins and bots can reach here
        Ok("Success".to_string())
    }

    #[graphql(name = "memberOnlyMutation", guard = "AuthGuard")]
    async fn member_only_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Any authenticated user can reach here
        Ok("Success".to_string())
    }

    #[graphql(name = "adminOnlyMutation", guard = "AdminGuard")]
    async fn admin_only_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Only admins can reach here
        Ok("Success".to_string())
    }
}
```

### Available Guards

- `AuthGuard` - Requires any authenticated user (Member, Admin, or Bot)
- `AdminGuard` - Requires Admin role
- `AdminOrBotGuard` - Requires Admin or Bot role

## Troubleshooting

### "Authentication context not found" Error

The auth middleware is not properly configured. Ensure:

- That the `Authorization:` header is present in the HTTP Request.

### "User is not a member of the amfoss organization"

The GitHub account is not part of the specified organization. Either:
1. Add the user to the organization
2. Change `GITHUB_ORG_NAME` in .env (for testing)

## API Reference

### GraphQL Mutations

#### `createBot(name: String!): String!` 🔒 Admin only

Create a new bot with API key.

**Input:**
- `name`: Bot name/description

**Returns:** The API key string (only shown once!)

## Example

### Complete Member Authentication Flow

```javascript
// 1. Redirect user to the backend's OAuth initiation endpoint
function login() {
  window.location.href = 'http://localhost:5000/auth/github';
}

// After the user authorizes on GitHub, the backend handles everything
// and redirects the user back to the frontend (e.g., to the dashboard).
// The user is now authenticated, and the session cookie is set.

// 2. Make authenticated requests
// The browser will automatically include the session cookie.
// No need to manually set Authorization headers.
async function fetchMemberData() {
  const response = await fetch('http://localhost:5000/', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: '{ member(memberId: 1) { name } }'
    })
  });
  
  const result = await response.json();
  console.log(result.data.member);
}
```
## Architecture Notes

### Authentication Flow

1. **Request arrives** → `auth_middleware` is executed.
2. **Cookie check** → The middleware first checks for a `session_token` cookie. If valid, the associated member is found.
3. **API Key check** → If no valid session cookie is found, it checks the `Authorization: Bearer <token>` header for an API key.
4. **Member lookup** → If a valid key is found, the associated bot member is retrieved.
5. **Context injection** → An `AuthContext` with the member (or `None`) is added to the request extensions.
6. **GraphQL execution** → Guards check `AuthContext` for permissions.
7. **Response** → Returns data or a permission error.

### Bot Members

Bots are represented as synthetic Member objects with:
- Negative member_id (to distinguish from real members)
- Role set to Bot
- Email format: `bot-{api_key_id}@internal.amfoss.in`
- Name from API key name


### API Key Management

- API keys are prefixed with `root_`
- Keys are hashed with bcrypt (cost factor 12) before storage
- Last used timestamp is updated on each validation
- Keys can only be deleted by admins
