# Agent Skills index

An index of the skills you publish, each being a document that teaches an agent to do something with your product.

**Passes when** `/.well-known/agent-skills/index.json`, or the older `/.well-known/skills/index.json`, has a non-empty `skills` array.

```json file=/.well-known/agent-skills/index.json
{
  "$schema": "https://schemas.agentskills.io/discovery/0.2.0/schema.json",
  "skills": [
    {
      "name": "track-order",
      "type": "skill-md",
      "description": "Look up the state of an order. Use when a user asks where their parcel is.",
      "url": "/.well-known/agent-skills/track-order/SKILL.md",
      "digest": "sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
    }
  ]
}
```

Work out each digest from the published file, for example with `sha256sum SKILL.md`, and do it in your build.

## Easy mistakes

- Writing the digest by hand. It goes stale on the next edit, and the skill then fails verification.
- A vague `description`. It is what an agent reads to decide on using the skill, so say when it applies.
