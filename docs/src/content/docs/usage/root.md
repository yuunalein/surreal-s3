---
title: Base
description: Get, put, and delete objects directly against an S3 bucket.
sidebar:
  order: 0
---

| Function                    | Description                                  |
| --------------------------- | -------------------------------------------- |
| [`s3::get()`](#s3get)       | Returns the contents of the specified object |
| [`s3::put()`](#s3put)       | Uploads content to the specified object      |
| [`s3::delete()`](#s3delete) | Deletes the specified object                 |

## `s3::get`

The `s3::get` function retrieves the contents of `key` from `bucket` via an S3 `GET` request.

### Parameters

| Parameter | Type   | Description                       |
| --------- | ------ | --------------------------------- |
| `bucket`  | String | The S3 bucket to read from        |
| `key`     | String | The key of the object to retrieve |

### Returns

| Type  | Description            |
| ----- | ---------------------- |
| Bytes | The contents of object |

### Example

```surql
<string> s3::get('main', 'hello.txt');
```

**Returns** `'Hello World!'`

### Errors

`s3::get` fails the query under the following conditions:

- `The specified bucket does not exist`
- `The specified key does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.

## `s3::put`

The `s3::put` function uploads `content` to `key` in `bucket` via an S3 `PUT` request.
If `key` already exists, its content is replaced.

### Parameters

| Parameter | Type   | Description                    |
| --------- | ------ | ------------------------------ |
| `bucket`  | String | The S3 bucket to write to      |
| `key`     | String | The key of the object to write |
| `content` | Bytes  | The content to upload          |

### Returns

| Type | Description                       |
| ---- | --------------------------------- |
| NONE | Returned once the upload succeeds |

### Example

```surql
s3::put('main', 'hello.txt', <bytes>'Hello World!');
```

**Returns** `NONE`

### Errors

`s3::put` fails the query under the following conditions:

- `The specified bucket does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.

## `s3::delete`

The `s3::delete` function deletes `key` from `bucket` via an S3 `DELETE` request.
Succeeds even if `key` does not exist.

### Parameters

| Parameter | Type   | Description                     |
| --------- | ------ | ------------------------------- |
| `bucket`  | String | The S3 bucket to delete from    |
| `key`     | String | The key of the object to delete |

### Returns

| Type | Description                         |
| ---- | ----------------------------------- |
| NONE | Returned once the object is deleted |

### Example

```surql
s3::delete('main', 'hello.txt');
```

**Returns** `NONE`

### Errors

`s3::delete` fails the query under the following conditions:

- `The specified bucket does not exist`

Any other error indicates a misconfigured bucket (credentials, permissions) or module,
and is reported as raised by the underlying S3/AWS SDK or HTTP client.
