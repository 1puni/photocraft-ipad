# Security

## Report a vulnerability

Email **[gg@1puni.com](mailto:gg@1puni.com)** with the subject
“PhotoCraft iPad security”. Include the affected revision, reproduction steps,
and impact. Use a synthetic example instead of a personal document where possible.
This is 1puni's existing private GG inbox; access was verified on 8 October 2026.
Do not put exploit details or private documents in a public issue.

Once this repository is public, GitHub private vulnerability reporting can also
be enabled. The email route works independently of that visibility change.

## Local preview

The included HTTP server is a development preview for a trusted local network,
not a public hosting service. It serves only `public/`, binds all network
interfaces, and provides no authentication or TLS. Do not expose it to the internet.
Editing happens in the browser; this server has no document-upload endpoint.

For a vulnerability reproduced in unmodified PhotoCraft, follow
[upstream's security policy](https://github.com/storytold/photocraft/security/policy).
