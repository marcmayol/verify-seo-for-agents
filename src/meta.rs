//! The checks: where they belong, what they mean and how to fix them.

pub struct Meta {
    pub key: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub fix: &'static str,
    pub specs: &'static [&'static str],
}

pub const CATEGORIES: [(&str, &str); 5] = [
    ("discoverability", "Discoverability"),
    ("contentAccessibility", "Content"),
    ("botAccessControl", "Bot access control"),
    ("discovery", "API, auth, MCP and skills"),
    ("commerce", "Commerce"),
];

/// Reported, but never counted in the percentage.
pub const INFORMATIONAL: [&str; 1] = ["personalAgentProtocol"];
/// Opt-in: they only count in the percentage when asked for by name.
pub const OPT_IN: [&str; 2] = ["a2aAgentCard", "ap2"];

pub const PROFILE_CONTENT: [&str; 7] =
    ["robotsTxt", "sitemap", "linkHeaders", "dnsAid", "markdownNegotiation", "robotsTxtAiRules", "contentSignals"];
pub const PROFILE_API: [&str; 17] = [
    "robotsTxt", "sitemap", "linkHeaders", "dnsAid", "markdownNegotiation", "robotsTxtAiRules", "contentSignals",
    "webBotAuth", "apiCatalog", "oauthDiscovery", "oauthProtectedResource", "authMd", "mcpServerCard", "agentSkills",
    "webMcp", "ard", "personalAgentProtocol",
];

pub const CHECKS: [Meta; 23] = [
    Meta {
        key: "robotsTxt",
        category: "discoverability",
        title: "robots.txt",
        fix: "Serve /robots.txt as text/plain with status 200 and at least one User-agent group.",
        specs: &["https://www.rfc-editor.org/rfc/rfc9309"],
    },
    Meta {
        key: "sitemap",
        category: "discoverability",
        title: "Sitemap",
        fix: "Publish an XML sitemap at /sitemap.xml and point to it with a Sitemap line in robots.txt.",
        specs: &["https://www.sitemaps.org/protocol.html"],
    },
    Meta {
        key: "linkHeaders",
        category: "discoverability",
        title: "Link headers",
        fix: "Send a Link header on the homepage with relations agents can follow: service-desc, service-doc, describedby or api-catalog.",
        specs: &["https://www.rfc-editor.org/rfc/rfc8288", "https://www.rfc-editor.org/rfc/rfc9727#section-3"],
    },
    Meta {
        key: "dnsAid",
        category: "discoverability",
        title: "DNS for AI Discovery (DNS-AID)",
        fix: "Publish ServiceMode SVCB or HTTPS records such as _index._agents.example.com and sign the zone with DNSSEC.",
        specs: &["https://datatracker.ietf.org/doc/draft-mozleywilliams-dnsop-dnsaid/", "https://www.rfc-editor.org/rfc/rfc9460"],
    },
    Meta {
        key: "markdownNegotiation",
        category: "contentAccessibility",
        title: "Markdown negotiation",
        fix: "Answer requests with Accept: text/markdown with a Markdown version of the page and Content-Type: text/markdown. Browsers keep getting HTML.",
        specs: &["https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/"],
    },
    Meta {
        key: "robotsTxtAiRules",
        category: "botAccessControl",
        title: "AI bot rules",
        fix: "State your policy for AI crawlers in robots.txt. Search bots such as OAI-SearchBot decide whether you appear in AI answers; training bots such as GPTBot only collect training data.",
        specs: &["https://www.rfc-editor.org/rfc/rfc9309"],
    },
    Meta {
        key: "contentSignals",
        category: "botAccessControl",
        title: "Content Signals",
        fix: "Add a line to robots.txt with your preferences, for example: Content-Signal: ai-train=no, search=yes, ai-input=yes",
        specs: &["https://contentsignals.org/", "https://datatracker.ietf.org/doc/draft-romm-aipref-contentsignals/"],
    },
    Meta {
        key: "webBotAuth",
        category: "botAccessControl",
        title: "Web Bot Auth",
        fix: "Only if you run bots: publish your signing keys as a JWKS at /.well-known/http-message-signatures-directory.",
        specs: &["https://datatracker.ietf.org/wg/webbotauth/about/"],
    },
    Meta {
        key: "apiCatalog",
        category: "discovery",
        title: "API catalog",
        fix: "Serve /.well-known/api-catalog as application/linkset+json, with a linkset array holding an anchor and service-desc or service-doc links for each API.",
        specs: &["https://www.rfc-editor.org/rfc/rfc9727", "https://www.rfc-editor.org/rfc/rfc9264"],
    },
    Meta {
        key: "oauthDiscovery",
        category: "discovery",
        title: "OAuth/OIDC discovery",
        fix: "If your APIs need authentication, publish /.well-known/oauth-authorization-server or /.well-known/openid-configuration with issuer and endpoints.",
        specs: &["https://www.rfc-editor.org/rfc/rfc8414", "http://openid.net/specs/openid-connect-discovery-1_0.html"],
    },
    Meta {
        key: "oauthProtectedResource",
        category: "discovery",
        title: "OAuth Protected Resource",
        fix: "Publish /.well-known/oauth-protected-resource with resource and authorization_servers.",
        specs: &["https://www.rfc-editor.org/rfc/rfc9728"],
    },
    Meta {
        key: "authMd",
        category: "discovery",
        title: "Auth.md",
        fix: "Serve /auth.md as Markdown with an H1 containing auth.md. Document how an agent registers, or add an agent_auth block with register_uri to your OAuth server metadata.",
        specs: &["https://workos.com/auth-md", "https://github.com/workos/auth.md"],
    },
    Meta {
        key: "mcpServerCard",
        category: "discovery",
        title: "MCP Server Card",
        fix: "Serve /.well-known/mcp/server-card.json with serverInfo (name, version), the transport endpoint and capabilities.",
        specs: &["https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2127"],
    },
    Meta {
        key: "a2aAgentCard",
        category: "discovery",
        title: "A2A Agent Card",
        fix: "Serve /.well-known/agent-card.json with name, version and supportedInterfaces.",
        specs: &["https://a2a-protocol.org/latest/specification/"],
    },
    Meta {
        key: "agentSkills",
        category: "discovery",
        title: "Agent Skills index",
        fix: "Publish /.well-known/agent-skills/index.json with $schema and a skills array (name, type, description, url, digest).",
        specs: &["https://github.com/cloudflare/agent-skills-discovery-rfc"],
    },
    Meta {
        key: "webMcp",
        category: "discovery",
        title: "WebMCP",
        fix: "Register your main actions on page load with document.modelContext.registerTool(), or add toolname and tooldescription to your forms.",
        specs: &["https://webmachinelearning.github.io/webmcp/"],
    },
    Meta {
        key: "ard",
        category: "discovery",
        title: "Agentic Resource Discovery",
        fix: "Serve /.well-known/ai-catalog.json with specVersion and an entries array. Each entry needs identifier, displayName, type and exactly one of url or data.",
        specs: &["https://agenticresourcediscovery.org/", "https://github.com/Agent-Card/ai-catalog"],
    },
    Meta {
        key: "personalAgentProtocol",
        category: "discovery",
        title: "Personal Agent Protocol",
        fix: "Serve /.well-known/poppy.json with protocol_version, organization (name, domain) and the interfaces you implement.",
        specs: &["https://personalagentprotocol.org/docs/spec#3-discovery"],
    },
    Meta {
        key: "x402",
        category: "commerce",
        title: "x402",
        fix: "Answer unpaid requests to paid API routes with HTTP 402 and the x402 payment details.",
        specs: &["https://x402.org", "https://docs.x402.org"],
    },
    Meta {
        key: "mpp",
        category: "commerce",
        title: "Machine Payment Protocol",
        fix: "Publish /openapi.json with x-payment-info on the operations that are paid.",
        specs: &["https://mpp.dev"],
    },
    Meta {
        key: "ucp",
        category: "commerce",
        title: "Universal Commerce Protocol",
        fix: "Serve /.well-known/ucp with the protocol version, services and capabilities.",
        specs: &["https://ucp.dev/specification/overview/"],
    },
    Meta {
        key: "acp",
        category: "commerce",
        title: "Agentic Commerce Protocol",
        fix: "Serve /.well-known/acp.json with protocol.name, protocol.version, api_base_url, transports and capabilities.services.",
        specs: &["https://agenticcommerce.dev"],
    },
    Meta {
        key: "ap2",
        category: "commerce",
        title: "Agent Payments Protocol",
        fix: "Declare the AP2 extension and your role (merchant, shopper...) in your A2A Agent Card.",
        specs: &["https://ap2-protocol.org/"],
    },
];

pub fn meta(key: &str) -> &'static Meta {
    CHECKS.iter().find(|m| m.key == key).expect("known check")
}
