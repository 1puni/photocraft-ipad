# Discoverability

The project website is https://1puni.github.io/photocraft-ipad/.
Its source is `site/`; GitHub Pages serves the root of the `gh-pages` branch,
containing only that directory. After verifying and committing site changes on
`main`, publish with `git subtree push --prefix site origin gh-pages`.
The current credential cannot publish Actions workflows, so this uses Pages'
built-in branch deployment without expanding credential scopes.
It is a static introduction with screenshots and local setup links,
not a hosted editor. No Rust build, personal documents or local preview files
are included. Update the canonical URL, social URLs and sitemap together if
the site moves. Screenshot sources and attribution remain in `docs/visuals.md`
and `ATTRIBUTION.md`.

On 19 October 2026 the maintainer requested discoverability fixes. This extends
the source-release direction to this public project landing page and repository
metadata. A compiled editor demo still needs a reproducible licensed build and
live browser acceptance before it can be advertised.

Use accurate GitHub topics for the product, audience and technology. Stars are
not an indexing prerequisite; neither topics nor deployment guarantee rankings.
Keep the README's language natural and distinguish current workflows from the
physical-device acceptance and recovery work on the roadmap. Do not label the
workspace an official ArtCraft edition or promise full Photoshop compatibility.

## Next discovery checks

- A site owner can add the URL-prefix property to Google Search Console, verify
  ownership, submit `sitemap.xml`, and inspect the live URL. Publishing a sitemap
  does not itself submit it or prove indexing.
- Link this project from relevant owned project directories when their owners
  accept it. Useful inbound links and actual feedback matter more than adding
  unrelated popular topics.
- Community announcements require a separate instruction to send. Previously
  withdrawn outreach must not be reused. Share a concrete workflow and the v0.1
  limitations when an announcement is authorized.

References: [GitHub topics](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/classifying-your-repository-with-topics),
[Google JavaScript SEO](https://developers.google.com/search/docs/crawling-indexing/javascript/javascript-seo-basics),
[Google crawling and indexing](https://developers.google.com/search/docs/fundamentals/how-search-works).
