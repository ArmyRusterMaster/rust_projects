# WebAuthn And Passkeys

WebAuthn позволяет входить без пароля или использовать аппаратный фактор как MFA. Passkeys строятся поверх FIDO2/WebAuthn и дают устойчивость к phishing при корректной настройке.

## Компоненты

- Relying Party - ваше приложение или auth service.
- Authenticator - устройство или менеджер passkeys.
- Credential - публичный ключ и metadata.
- Challenge - одноразовое значение для регистрации или входа.

## Registration

При регистрации:

- Сгенерировать случайный challenge.
- Указать правильный RP ID.
- Проверить origin.
- Сохранить credential id, public key, sign counter и user handle.
- Привязать credential к пользователю и устройству.

## Authentication

При входе:

- Сгенерировать новый challenge.
- Проверить origin и RP ID hash.
- Проверить подпись authenticator.
- Проверить sign counter, если authenticator его поддерживает.
- Обновить данные credential.

## Режимы использования

- Passwordless login - passkey является основным способом входа.
- MFA - passkey или security key является вторым фактором.
- Step-up authentication - повторная проверка перед опасным действием.

## Практики

- Разрешить пользователю иметь несколько credentials.
- Сделать recovery flow, но защищать его не слабее основного входа.
- Логировать добавление и удаление credentials.
- Требовать повторную проверку перед удалением последнего сильного фактора.
