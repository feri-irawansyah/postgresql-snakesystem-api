# Authentication Specification

## Login Endpoint
### POST `/auth/login`

Login user with email and password.

### Request Body
```json
{
    "email": "email",
    "password": "password"
}
```

### Response `200 OK`
```json
{
    "data": "Login successfully"
}
```

### Response `400 Bad Request`
```json
{
    "error": "Invalid email or password"
}
```

### Response `500 Internal Server Error`
```json
{
    "error": "Internal server error"
}
```

## Check Session Endpoint
### GET `/auth/session`

Check user session.

### Response `200 OK`
```json
{
    "data": {
        "app_name": "snakesystem-api",
        "comp_name": "Unknown Device",
        "disabled_login": false,
        "email": "example@gmail.com",
        "exp": 1748847227,
        "expired_date": "2025-06-02 06:53:47",
        "expired_token": 1748847227,
        "fullname": "Satria Baja Ringan",
        "ip_address": "127.0.0.1",
        "picture": "",
        "register_date": "2025-05-31T13:53:47",
        "result": true,
        "usernid": 1
    }
}
```

### Response `401 Unauthorized`
```json
{
    "error": "Unauthorized"
}
```

### Response `400 Bad Request`
```json
{
    "error": "Invalid token"
}
```

### Response `500 Internal Server Error`
```json
{
    "error": "Internal server error"
}
```