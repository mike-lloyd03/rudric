# Rudric

## Session Tokens

- Get user master password
- Encrypt it using a new random key
- Store key to db
- Generate UUID and concatenate it with encrypted password. Encode and set as session token.
- Allow user to invalidate the token
- If env var is set, decrypt it
- Derive key from password using db salt
