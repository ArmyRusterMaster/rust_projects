# Auth Protocols Summary

## OAuth 2.0
- Основной flow: Authorization Code + PKCE.
- Для M2M: Client Credentials с минимальными scope.
- Избегать Implicit и ROPC (кроме legacy).
- Обязательно проверять redirect URI, state, audience/issuer/exp.

## OpenID Connect
- Использовать для login/identity поверх OAuth 2.0.
- ID Token только для identity, не для API access.
- Проверки: `iss`, `aud`, `exp`, `iat`, `nonce`, подпись через JWKS.

## Tokens
- Access token короткоживущий (обычно 5-15 мин).
- Refresh token только в hashed-виде + rotation.
- Reuse detection должен отзывать token family.
- JWT нужен, когда требуется оффлайн-валидация на многих resource server.

## Sessions/Cookies
- Для браузера предпочтительна серверная сессия + `HttpOnly` cookie.
- Флаги: `HttpOnly`, `Secure`, `SameSite=Lax/Strict`.
- При cookie-аутентификации нужна CSRF защита.

## SAML 2.0
- Нужен для enterprise SSO совместимости.
- Проверять подпись, issuer/audience, time bounds, replay-защиту.

## WebAuthn/Passkeys
- Подходит для passwordless/MFA/step-up.
- Проверять challenge, origin, RP ID hash, подпись и sign counter.
