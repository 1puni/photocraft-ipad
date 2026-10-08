# Security

## Report a vulnerability

Email **[gg@1puni.com](mailto:gg@1puni.com)** with the subject
“PhotoCraft iPad security”. Include the affected revision, reproduction steps,
and impact. Use a synthetic example instead of a personal document where possible.
Do not put exploit details or private documents in a public issue.

You can also use [GitHub private vulnerability reporting](https://github.com/1puni/photocraft-ipad/security/advisories/new).

## Local preview

The included HTTP server is a development preview for a trusted local network,
not a public hosting service. It serves only `public/`, binds all network
interfaces, and provides no authentication or TLS. Do not expose it to the internet.
Editing happens in the browser; this server has no document-upload endpoint.

For a vulnerability reproduced in unmodified PhotoCraft, follow
[upstream's security policy](https://github.com/storytold/photocraft/security/policy).
