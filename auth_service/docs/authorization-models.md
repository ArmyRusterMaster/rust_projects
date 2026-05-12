# Authorization Models

Авторизация должна быть отдельной частью архитектуры. Не смешивайте проверку прав с проверкой пароля, токена или сессии.

## RBAC

Role-Based Access Control выдает права через роли.

Подходит, если:

- Модель доступа стабильная.
- Пользователи группируются в понятные роли.
- Нужны простые административные панели.

Риск: роли быстро превращаются в слишком крупные наборы прав.

## ABAC

Attribute-Based Access Control принимает решение по атрибутам субъекта, ресурса, действия и контекста.

Примеры атрибутов:

- department
- tenant_id
- resource_owner_id
- request_ip
- risk_score
- time_of_day

Подходит для сложных правил и multi-tenant систем.

## ReBAC

Relationship-Based Access Control принимает решение на основе связей между субъектами и ресурсами.

Примеры:

- user owns document
- user belongs to organization
- user is manager of employee
- team has access to repository

Подходит для collaborative SaaS и систем с графом доступа.

## Policy-Based Authorization

Политики лучше хранить отдельно от HTTP handlers. Тогда их можно тестировать независимо.

Пример решения:

```text
subject: user:123
action: invoice:approve
resource: invoice:456
context: tenant=acme, ip=10.0.0.5
decision: allow или deny
```

## Практики

- Deny by default.
- Проверять tenant boundary в каждой политике.
- Логировать deny для расследований.
- Версионировать политики.
- Писать unit tests для критичных правил.
- Не доверять claims токена без проверки issuer, audience и freshness.
