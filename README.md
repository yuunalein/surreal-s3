<h1 align="center">Surreal S3</h1>

<p align="center">
  <strong>Surreal S3</strong> is a plugin for <a href="https://github.com/surrealdb/surrealdb">SurrealDB</a> that enables interfacing with S3 inside
  SurrealQL
</p>

<p align="center">
  <a href="https://surreal-s3.yuuna.dev"><img alt="Website" src="https://img.shields.io/website?url=https%3A%2F%2Fsurreal-s3.yuuna.dev&up_message=online&up_color=%23d255fe&down_message=offline&down_color=red&style=for-the-badge&label=docs"></a>
  <a href="https://github.com/yuunalein/surreal-s3/releases"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/yuunalein/surreal-s3?style=for-the-badge&label=version"></a>
  <a href="https://github.com/yuunalein/surreal-s3/releases"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yuunalein/surreal-s3/ci.yml?branch=main&style=for-the-badge"></a>
  <a href="https://github.com/surrealdb/surrealdb"><img src="https://img.shields.io/badge/built_with-Rust-dca282.svg?style=for-the-badge"></a>
  <a href="https://github.com/yuunalein/surreal-s3"><img alt="GitHub License" src="https://img.shields.io/github/license/yuunalein/surreal-s3?style=for-the-badge"></a>
</p>

## Features

- Get, put, and delete objects
- Generate pre-signed URIs
- Multipart uploads

## Example

Generate a pre-signed URL for an object in S3:

```surql
mod::s3::sign::get('main', 'hello.txt', 5m);
```

**Returns**

```surql
{
	"method": "GET",
	"uri": "https://main.s3.amazonaws.com/hello.txt?x-id=GetObject&...",
	"expires_at": "2026-09-15T12:05:00Z"
}
```

## Installation

See [Installation](https://surreal-s3.yuuna.dev/overview/installation) in the docs.

## Todo

- [ ] Automated tests, run in CI
- [ ] Automatically re-run those tests against the module's latest release whenever
      SurrealDB publishes a new release or pre-release
- [ ] Distribute the binary via [SurrealDB 3.3.0 Silo](https://surrealdb.com/docs/reference/query-language/statements/define/module#silo-packages)
- [ ] Create, delete, and set rules on buckets

## Development

This project uses the nightly Rust toolchain. Installing SurrealDB locally is
recommended, to test changes and build the `.surli` binary locally.

- Building the source code
  ```sh
  cargo build # --release for release builds
  ```
- Building the `.surli` binary

  ```sh
  surreal module build --debug # remove debug for release builds
  ```

  [Read more about `surreal module`](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/module)
  in the official SurrealDB docs.

The docs, under [`docs/`](docs), are an [Astro](https://astro.build) site built with Bun.

- Running the docs locally
  ```sh
  bun install
  bun run dev
  ```
- Building the docs
  ```sh
  bun run build
  ```

## Contributions

Contributions are welcome. Open an issue first to discuss the change before
implementing it, so effort isn't spent on a PR that won't be accepted.

Pull requests and issues that are entirely AI-generated, without a human author
reviewing and standing behind the content, will be closed without discussion.

A PR that doesn't meet the quality maintainers expect will be left open for the author
or another contributor to bring up to standard. If it isn't, it will eventually be
closed. Issues and PRs judged not to fit the project will be closed, and repeat
offenders will be blocked.

---

Surreal S3 is an independent project, not affiliated with or endorsed by SurrealDB.

Made with ❤️ by [yuunalein](https://github.com/yuunalein).
