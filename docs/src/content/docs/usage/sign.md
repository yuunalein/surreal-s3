---
title: Pre-signed URIs
description: Generate short-lived, pre-signed URIs for direct GET, PUT, HEAD, and DELETE requests against S3 objects.
---

| Function                              | Description                                               |
| ------------------------------------- | --------------------------------------------------------- |
| [`s3::sign::get()`](#s3signget)       | Returns a URI for a GET action on the specified object    |
| [`s3::sign::put()`](#s3signput)       | Returns a URI for a PUT action on the specified object    |
| [`s3::sign::head()`](#s3signhead)     | Returns a URI for a HEAD action on the specified object   |
| [`s3::sign::delete()`](#s3signdelete) | Returns a URI for a DELETE action on the specified object |

:::note[About Errors]
The `s3::sign::*` functions do not contact S3. They fail the query only if
`expires_in` exceeds one week, the maximum S3 permits for a
pre-signed URI. `bucket` and `key` are not verified to exist.
:::

## `s3::sign::get`

The `s3::sign::get` function returns a pre-signed URI for a `GET` request on `key` in
`bucket`, valid for `expires_in`.

### Parameters

| Parameter    | Type     | Description                         |
| ------------ | -------- | ----------------------------------- |
| `bucket`     | String   | The S3 bucket containing the object |
| `key`        | String   | The key of the object to retrieve   |
| `expires_in` | Duration | How long the URI remains valid      |

### Returns

| Field        | Type   | Description                                   |
| ------------ | ------ | --------------------------------------------- |
| `method`     | String | The HTTP method the URI is signed for (`GET`) |
| `uri`        | String | The pre-signed URI                            |
| `expires_at` | String | The RFC 3339 timestamp the URI expires at     |

### Example

```surql
s3::sign::get('main', 'hello.txt', 5m);
```

**Returns**

```json
{
	"method": "GET",
	"uri": "https://main.s3.amazonaws.com/hello.txt?x-id=GetObject&...",
	"expires_at": "2026-09-15T12:05:00Z"
}
```

## `s3::sign::put`

The `s3::sign::put` function returns a pre-signed URI for a `PUT` request on `key` in
`bucket`, valid for `expires_in`.

### Parameters

| Parameter      | Type             | Description                                                       |
| -------------- | ---------------- | ----------------------------------------------------------------- |
| `bucket`       | String           | The S3 bucket to write to                                         |
| `key`          | String           | The key of the object to write                                    |
| `expires_in`   | Duration         | How long the URI remains valid                                    |
| `content_size` | Option\<Number\> | If set, the URI only accepts an upload of exactly this many bytes |

### Returns

| Field        | Type   | Description                                   |
| ------------ | ------ | --------------------------------------------- |
| `method`     | String | The HTTP method the URI is signed for (`PUT`) |
| `uri`        | String | The pre-signed URI                            |
| `expires_at` | String | The RFC 3339 timestamp the URI expires at     |

### Example

```surql
s3::sign::put('main', 'hello.txt', 5m, none);
```

**Returns**

```json
{
	"method": "PUT",
	"uri": "https://main.s3.amazonaws.com/hello.txt?x-id=PutObject&...",
	"expires_at": "2026-09-15T12:05:00Z"
}
```

## `s3::sign::head`

The `s3::sign::head` function returns a pre-signed URI for a `HEAD` request on `key` in
`bucket`, valid for `expires_in`.

### Parameters

| Parameter    | Type     | Description                         |
| ------------ | -------- | ----------------------------------- |
| `bucket`     | String   | The S3 bucket containing the object |
| `key`        | String   | The key of the object to inspect    |
| `expires_in` | Duration | How long the URI remains valid      |

### Returns

| Field        | Type   | Description                                    |
| ------------ | ------ | ---------------------------------------------- |
| `method`     | String | The HTTP method the URI is signed for (`HEAD`) |
| `uri`        | String | The pre-signed URI                             |
| `expires_at` | String | The RFC 3339 timestamp the URI expires at      |

### Example

```surql
s3::sign::head('main', 'hello.txt', 5m);
```

**Returns**

```json
{
	"method": "HEAD",
	"uri": "https://main.s3.amazonaws.com/hello.txt?X-Amz-Algorithm=...",
	"expires_at": "2026-09-15T12:05:00Z"
}
```

## `s3::sign::delete`

The `s3::sign::delete` function returns a pre-signed URI for a `DELETE` request on `key`
in `bucket`, valid for `expires_in`.

### Parameters

| Parameter    | Type     | Description                         |
| ------------ | -------- | ----------------------------------- |
| `bucket`     | String   | The S3 bucket containing the object |
| `key`        | String   | The key of the object to delete     |
| `expires_in` | Duration | How long the URI remains valid      |

### Returns

| Field        | Type   | Description                                      |
| ------------ | ------ | ------------------------------------------------ |
| `method`     | String | The HTTP method the URI is signed for (`DELETE`) |
| `uri`        | String | The pre-signed URI                               |
| `expires_at` | String | The RFC 3339 timestamp the URI expires at        |

### Example

```surql
s3::sign::delete('main', 'hello.txt', 5m);
```

**Returns**

```json
{
	"method": "DELETE",
	"uri": "https://main.s3.amazonaws.com/hello.txt?x-id=DeleteObject&...",
	"expires_at": "2026-09-15T12:05:00Z"
}
```
