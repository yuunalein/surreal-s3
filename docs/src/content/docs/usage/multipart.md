---
title: Multipart
description: Upload large objects to S3 in parts, signed for direct client upload.
---

| Function                                            | Description                                                                      |
| --------------------------------------------------- | -------------------------------------------------------------------------------- |
| [`s3::multipart::create()`](#s3multipartcreate)     | Starts a multipart upload for the specified object and returns its upload ID     |
| [`s3::multipart::uri()`](#s3multiparturi)           | Returns URIs for PUT actions to upload a range of parts for the specified upload |
| [`s3::multipart::complete()`](#s3multipartcomplete) | Completes the specified multipart upload using each part's part number and ETag  |
| [`s3::multipart::abort()`](#s3multipartabort)       | Aborts the specified multipart upload and discards any uploaded parts            |

## `s3::multipart::create`

The `s3::multipart::create` function starts a multipart upload for `key` in `bucket` and
returns its upload ID.

### Parameters

| Parameter | Type   | Description                     |
| --------- | ------ | ------------------------------- |
| `bucket`  | String | The S3 bucket to upload to      |
| `key`     | String | The key of the object to upload |

### Returns

| Type   | Description                            |
| ------ | -------------------------------------- |
| String | The ID of the started multipart upload |

### Example

```surql
s3::multipart::create('main', 'movie.mp4');
```

**Returns** `'3857b672471...'`

### Errors

`s3::multipart::create` fails the query under the following conditions:

- `The specified bucket does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.

## `s3::multipart::uri`

The `s3::multipart::uri` function returns pre-signed URIs that each perform a `PUT` request
for one part of the multipart upload identified by `upload_id`, valid for `expires_in`,
without requiring SurrealDB credentials. `count` URIs are returned, for part numbers
starting at `start`.

### Parameters

| Parameter    | Type             | Description                                                        |
| ------------ | ---------------- | ------------------------------------------------------------------ |
| `bucket`     | String           | The S3 bucket the upload was started in                            |
| `key`        | String           | The key of the object being uploaded                               |
| `upload_id`  | String           | The upload ID returned by `s3::multipart::create`                  |
| `start`      | Number           | The first part number to sign a URI for                            |
| `count`      | Number           | How many consecutive part numbers to sign URIs for                 |
| `expires_in` | Duration         | How long each URI remains valid                                    |
| `part_size`  | Option\<Number\> | If set, each URI only accepts an upload of exactly this many bytes |

### Returns

| Field        | Type            | Description                               |
| ------------ | --------------- | ----------------------------------------- |
| `parts`      | Array\<Object\> | One entry per signed part, see below      |
| `expires_at` | String          | The RFC 3339 timestamp the URIs expire at |

Each entry in `parts`:

| Field         | Type   | Description                      |
| ------------- | ------ | -------------------------------- |
| `part_number` | Number | The part number the URI is for   |
| `uri`         | String | The pre-signed URI for that part |

### Example

```surql
s3::multipart::uri('main', 'movie.mp4', '3857b672471...', 0, 2, 5m, none);
```

**Returns**

```json
{
	"expires_at": "2026-09-15T12:05:00Z",
	"parts": [
		{ "part_number": 0, "uri": "https://main.s3.amazonaws.com/movie.mp4?x-id=UploadPart&partNumber=..." },
		{ "part_number": 1, "uri": "https://main.s3.amazonaws.com/movie.mp4?x-id=UploadPart&partNumber=..." }
	]
}
```

### Errors

`s3::multipart::uri` does not contact S3. It fails the query only if `expires_in`
exceeds one week, the maximum S3 permits for a pre-signed URI. It does
not verify that `bucket`, `key`, or `upload_id` exist.

## `s3::multipart::complete`

The `s3::multipart::complete` function completes the multipart upload identified by
`upload_id`, assembling `key` in `bucket` from the parts in `parts`. Each entry in
`parts` supplies the part number and ETag returned by that part's `PUT` request.

### Parameters

| Parameter   | Type            | Description                                       |
| ----------- | --------------- | ------------------------------------------------- |
| `bucket`    | String          | The S3 bucket the upload was started in           |
| `key`       | String          | The key of the object being uploaded              |
| `upload_id` | String          | The upload ID returned by `s3::multipart::create` |
| `parts`     | Array\<Object\> | The uploaded parts, see below                     |

Each entry in `parts`:

| Field         | Type   | Description                                    |
| ------------- | ------ | ---------------------------------------------- |
| `part_number` | Number | The part's number                              |
| `e_tag`       | String | The ETag returned by that part's `PUT` request |

### Returns

| Type | Description                           |
| ---- | ------------------------------------- |
| NONE | Returned once the upload is completed |

### Example

```surql
s3::multipart::complete('main', 'movie.mp4', '3857b672471...', [
	{ part_number: 0, e_tag: "0622e44bd2c..." },
	{ part_number: 1, e_tag: "c436bbe50a1..." }
]);
```

**Returns** `NONE`

### Errors

`s3::multipart::complete` fails the query under the following conditions:

- `The specified bucket does not exist`
- `The specified multipart upload does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.

## `s3::multipart::abort`

The `s3::multipart::abort` function aborts the multipart upload identified by `upload_id`
and discards any parts already uploaded for `key` in `bucket`.

### Parameters

| Parameter   | Type   | Description                                       |
| ----------- | ------ | ------------------------------------------------- |
| `bucket`    | String | The S3 bucket the upload was started in           |
| `key`       | String | The key of the object being uploaded              |
| `upload_id` | String | The upload ID returned by `s3::multipart::create` |

### Returns

| Type | Description                         |
| ---- | ----------------------------------- |
| NONE | Returned once the upload is aborted |

### Example

```surql
s3::multipart::abort('main', 'movie.mp4', '3857b672471...');
```

**Returns** `NONE`

### Errors

`s3::multipart::abort` fails the query under the following conditions:

- `The specified bucket does not exist`
- `The specified multipart upload does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.
