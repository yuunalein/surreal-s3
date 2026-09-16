---
title: Configuration
description: Configure SurrealDB-S3.
---

The module reads its configuration from a single SurrealQL param, `$s3_module_config`,
which must be defined before it is installed. The module reads it once, during
its `init` export, so changes to the param after installation have no effect until the
module is reinstalled.

### Example

```surql
DEFINE PARAM $s3_module_config VALUE {
	access_key_id: "2241a7b748d...",
	secret_access_key: "84d789e9ebe...",
	endpoint_url: "https://s3.amazonaws.com"
} PERMISSIONS FULL;
```

### Fields

:::note
`name_servers` currently must be defined, since SurrealDB neither exposes enough
access through its `http::*` functions nor offers an HTTP binding in wasm for the
module to use instead. The module needs them to resolve S3 hostnames itself, directly
via DNS request. If `endpoint_url` is set to an IP address rather than a hostname, no
DNS resolution will happen and `name_servers` is not consulted.
:::

| Field                         | Type                      | Description                                                                                             |
| ----------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------- |
| `access_key_id`               | String                    | The access key ID used to sign requests against `endpoint_url`                                          |
| `secret_access_key`           | String                    | The secret access key used to sign requests against `endpoint_url`                                      |
| `endpoint_url`                | String                    | The S3-compatible endpoint the module talks to directly                                                 |
| `region`                      | Option\<String\>          | The AWS region to sign requests for. Defaults to `us-east-1`                                            |
| `force_path_style`            | Option\<Bool\>            | Whether `endpoint_url` requests address buckets as path segments. Defaults to `false`                   |
| `public_endpoint_url`         | Option\<String\>          | A separate endpoint to sign pre-signed URIs against, if different from `endpoint_url`                   |
| `public_force_path_style`     | Option\<Bool\>            | Whether `public_endpoint_url` requests address buckets as path segments. Defaults to `force_path_style` |
| `name_servers`                | Option\<Array\<String\>\> | Additional DNS name-servers, as `ip:port`, used to resolve S3 hostnames                                 |
| `enable_default_name_servers` | Option\<Bool\>            | Whether to also resolve against the built-in default name-servers. Defaults to `true`                   |

### Network access

SurrealDB must be started with a bare `--allow-net`, allowing full network access
from SurrealQL. Until SurrealDB implements a true wildcard network access
capability, this can't be restricted any further: `endpoint_url` and `name_servers`
aren't known until `$s3_module_config` is read, so the module can't declare the
specific hosts it needs at compile time, and instead works around this by declaring a
range covering all possible hosts, which forces full network access just to interact
with the module.

In practice, the module only ever talks to:

- `endpoint_url`, to issue the S3 request.
- The configured DNS name-servers, to resolve `endpoint_url` when it's a hostname
  rather than an IP address. Unless `enable_default_name_servers` is set to `false`,
  this also includes the built-in defaults:
  - `127.0.0.11:53`, Docker's embedded DNS server
  - `127.0.0.53:53`, systemd-resolved

`public_endpoint_url` is never contacted. It's only used to sign pre-signed URIs, and
signing is done locally, without a network request.

For both DNS lookups and the HTTP request itself, the module races every candidate
host at once and takes whichever response comes back first, only erroring if none of
them respond at all. Only one configured DNS name-server needs to actually be
reachable, so disabling the built-in defaults is usually unnecessary.
