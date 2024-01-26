# Rudric

A simple secrets vault.

<p align="center">
<img src=https://github.com/mike-lloyd03/rudric/assets/49411532/3916d3f4-4ba8-43cc-b83a-2eb9bd42e33f />
</p>

Rudric makes managing secrets from the command line simple and straight-forward. Secrets are stored encrypted on disk and can be quickly decrypted and added to the environment where needed. Cleartext secrets are only ever stored in memory.

# Purpose

I use `.env` files a lot to store sensitive data like API keys and personal access tokens. While `.env` files are "hidden" by default on Linux and Mac, this isn't really _security_. Anyone could grep the filesystem and find all sorts of plaintext secrets. Storing secrets encrypted, and only decrypting them when needed would be a much better solution.

# Usage

`.env` files can instead be replaced with `.renv` files which use bash-like syntax for defining environment variables.

```bash
GITHUB_TOKEN={{personal_github_token}}
GITHUB_API=https://api.github.com
```

This would create an environment variable called `GITHUB_TOKEN` which is set to the value of a secret in your vault called `personal_github_token`. Additionally, non-secret values can be stored here as well and they will also be set in the environment.

Multiple secrets can be used in a single environment variable. Additionally, string interpolation is supported.

```bash
DATABASE_URL=postgres://{{pg_user}}:{{pg_pass}}@localhost:5432
```

Using `rudric env <shell_name>`, these variables can be set in your environment.

**fish**:

```fish
rudric env fish | source
```

**bash**:

```bash
source <(rudric env bash)
```

A default shell can be specified by setting it in the configuration file.

# Getting Started

## Initialization

This first step is to generate a new vault with `rudric init`. You will be asked to set a master password. Once done, your vault will be created.

## Interacting with the vault

Secrets can be created, fetched, edited, and deleted.

```bash
rudric create <new_secret_name>
```

A text editor will be opened (whatever is defined by `EDITOR` or `VISUAL` environment variables). Optionally, the user can create a secret from the content of a file with the `--file` flag. Or, if a session is active, the user can create a secret by piping text from another process:

```bash
pwgen -1 14 | rudric create <new_secret_name> -
```

> [!IMPORTANT]
> A valid session token must be set for reading from a pipe to work correctly. Currently, the Rust crate `dialoguer` has a bug which prevents input prompts from being displayed correctly when data is piped into the program.

The following other commands are supported:

- edit
- list
- delete (asks for confirmation)
- change-password

## Sessions

To avoid having to type your master password every time you interact with the vault, you can create a session token which must be set in the environment as `RUDRIC_SESSION`. A simple shorthand for this might look like.

```fish
set -x RUDRIC_SESSION $(rudric session)
```

Session tokens are valid for 8 hours by default but this can be configured. The current session token can be revoked with `rudric session end`.

# Encryption

The master password is salted and hashed using the Argon2i algorithm.

Secret values are encrypted before writing to the database using XChaCha20Poly1305.

A higher order key is derived from your master password using a key derivation function based on Argon2i. This key is used for encrypting and decrypting secrets.

## Session Tokens

Session token generation is a convenience and obviously makes some security compromises in order to achieve this convenience. However, with proper management of your session tokens, there is a low risk of compromise. Don't ever store your session tokens on the disk. If someone managed to get both your vault database and your session token, all of your secrets could be decrypted.

### Session Token Generation

Session tokens are generated using the following method:

- An expiration time is set.
- A random key (the session key) is generated and written to the vault.
- The user's master key is derived from the master password and stored salt.
- The master key is prepended with the expiration time. This is encrypted using the session key.
- The encrypted expiration time and master key are prepended again with the UUID of the session key. This is base64 encoded and returned to the user as the session token.

If the `RUDRIC_SESSION` token is set in the environment:

- The session key is fetched from the database using the UUID found in the session token
- The expiration time and master key are decrypted from the session token using the session key
- If the token is not expired, the master key will be used to interact with secrets in the vault.

# Configuration

Rudric can be configured with a toml file. By default, this file is stored in `XDG_CONFIG/rudric/config.toml` (`$HOME/.config/rudric/config.toml` on Linux and Mac). All config options are optional. An example config file might look like this:

```toml
# Options are: bash, zsh, fish, nu
default_shell = "fish"

# Specify the default length of time that a session token is valid for
session_lifetime = "6h"

# Specify the name of the file to use in the CWD for setting environment variables (default ".renv")
renv_filename = ".env"
```

# direnv

Rudric includes support for [`direnv`](https://github.com/direnv/direnv). However, in order to use it, you will have to start a Rudric session before using `direnv`.

Simply add the following to your `.envrc` file:

```
$(rudric env direnv)
```

Changing into the directory with both a `.envrc` and `.renv` file will automatically source your encrypted secrets in the environment.

> [!IMPORTANT]
> If a valid session token is not set, changing into a directory with a trusted `.envrc` will prompt you for your password. This will fail as input will not be passed to Rudric. I'm working on a solution for this but for now, start a session with `rudric session` first.

# Crates

Encryption is all accomplished using the fantastic [Orion](https://github.com/orion-rs/orion) library.

Additional crates in use:

- clap
- sqlx
- dialoguer
- serde
- colored_json
- tabled
