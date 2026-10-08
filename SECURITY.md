# Security

This experimental browser editor imports complex, potentially untrusted files.
Keep backups. Automatic document recovery is not implemented in this web build.

The included HTTP server is a development preview for a trusted local network,
not a public hosting service. It serves only `public/`, binds all network
interfaces, and provides no authentication or TLS. Do not expose it to the internet.
Editing happens in the browser; this server has no document-upload endpoint.

For a suspected extension vulnerability, contact the repository owner privately
via an existing channel while this repository is private. A public reporting
channel will be confirmed before release. Do not put exploit details or private
documents in a public issue.

For a vulnerability reproduced in unmodified PhotoCraft, follow
[upstream's security policy](https://github.com/storytold/photocraft/security/policy).
