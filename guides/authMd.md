# Auth.md

`/auth.md` is a document written for agents, explaining how to get credentials without a person filling in a form.

**Passes when** `/auth.md` is Markdown rather than HTML, has an H1 containing `auth.md`, and either of these holds:

1. The OAuth server metadata has an `agent_auth` block with a `register_uri`.
2. The document stands alone, with a registration endpoint written as `POST /path` and a mention of the credential the agent receives.

```markdown file=/auth.md
# Example auth.md

You are an agent. Follow these steps to get a credential.

1. Register with `POST /agent/register` and a JSON body holding your name.
2. The response holds an `api_key`. Send it as a bearer token on every request.
3. Your user can claim the account later at https://example.com/claim.
```

With OAuth metadata, add this to `/.well-known/oauth-authorization-server`:

```json
{
  "issuer": "https://example.com",
  "agent_auth": {
    "register_uri": "https://example.com/agent/register",
    "identity_types_supported": ["anonymous"]
  }
}
```

## Easy mistakes

- An app shell answering `/auth.md` with HTML. The check treats that as a missing page.
- Telling agents to call an endpoint that creates accounts or sends mail with no way to undo it. Say what each step does.
