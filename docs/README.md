# Documentation

The reference documentation for Wickra Verify lives at
**[verify.wickra.org](https://verify.wickra.org)** — quickstarts, the API
surface for every binding, and the guides.

What stays here, beside the code, is the material that only makes sense next to
the implementation and has to change in the same commit as it:

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`CANONICALIZATION.md`](CANONICALIZATION.md)
- [`CLAIM_FORMAT.md`](CLAIM_FORMAT.md)
- [`Cookbook.md`](Cookbook.md)
- [`DETERMINISM.md`](DETERMINISM.md)
- [`VERDICT.md`](VERDICT.md)

That split is deliberate. A page describing *how the thing works* belongs where
the thing is, so a change to the mechanism and a change to its description are
reviewed together. A page describing *how to use it* belongs on the site, where
it can be versioned, searched, and read without cloning anything.

## Editing

The site is a separate repository at
`https://github.com/wickra-lib/wickra-verify-site`; open a pull request there for
anything under verify.wickra.org. Everything in this directory is edited
here, in the pull request that changes the behaviour it describes.
