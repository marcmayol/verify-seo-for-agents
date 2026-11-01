# AI bot rules

AI crawlers do different jobs, so one blanket rule rarely says what you mean.

**Passes when** `robots.txt` names a known AI bot, or has a `User-agent: *` group with at least one `Allow` or `Disallow` under it.

The two families to tell apart:

| Job | Bots | If you block them |
|---|---|---|
| Answering users | OAI-SearchBot, ChatGPT-User, Claude-SearchBot, Claude-User, PerplexityBot | You may stop appearing in AI answers |
| Collecting training data | GPTBot, ClaudeBot, Google-Extended, CCBot, Bytespider | Your pages can still appear in AI answers |

For example, staying visible in answers without feeding training:

```text file=/robots.txt
User-agent: GPTBot
User-agent: ClaudeBot
User-agent: Google-Extended
Disallow: /

User-agent: *
Allow: /
```

## Easy mistakes

- Treating a pass as approval of your policy. It only confirms that rules apply, so read `details.botPolicies` to see the access each bot really gets.
- Leaving `User-agent: *` alone at the end of the file. With no rule under it, it states nothing.
- Putting a blank line between the `User-agent` lines of one group, which some parsers read as two groups.
