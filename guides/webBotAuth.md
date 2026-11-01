# Web Bot Auth

This is for sites that send bots, not for those that receive them: it lets the sites your agent visits verify that the request really came from you. If you run no bots, skip it. It never lowers your score.

**Passes when** `/.well-known/http-message-signatures-directory` answers with a JSON Web Key Set holding at least one key.

```json file=/.well-known/http-message-signatures-directory
{
  "keys": [
    {
      "kty": "OKP",
      "crv": "Ed25519",
      "kid": "bot-key-2026",
      "x": "JrQLj5P_89iXES9-vFgrIy29clF9CC_oPPsw3c5D0bs"
    }
  ]
}
```

That `x` is a public test key from RFC 9421, so generate your own.

Publishing the key is half the job. Your bot also has to sign what it sends, with `Signature`, `Signature-Input` and `Signature-Agent` headers.

## Easy mistakes

- Publishing the private part of the key. A public JWK has `x` and never `d`.
- Replacing a key without a transition period. Keep the old one listed until requests signed with it have stopped.
