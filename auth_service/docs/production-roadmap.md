# Production Roadmap

## Stage 1: Runtime hardening
- Remove build artifacts from git, enforce `.gitignore`.
- Typed startup errors and predictable exit codes.
- Health/readiness endpoints for orchestration.
- In-flight request limiting.

## Stage 2: Security controls
- IP/email-aware rate limiting for login/refresh.
- Account lockout/backoff policy.
- Security headers and strict CORS policy.
- Redacted API errors for internal failures.

## Stage 3: Identity features
- Password reset flow (request/confirm).
- Email verification.
- MFA (TOTP + recovery codes).
- Session/device management APIs.

## Stage 4: Auth platform features
- OAuth2/OIDC endpoints.
- JWKS and key rotation.
- Token revocation/introspection RFC compatibility.
- Policy-based authorization (RBAC/ABAC).
