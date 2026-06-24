# Keystore Generator

Generate an Android APK signing keystore (PKCS12) and the four GitHub Actions secrets you need to sign release builds in CI.

## Prerequisites

- A **Java (JDK)** installation — the tool shells out to `keytool`. sbox checks for Java on startup and shows its path. If Java isn't found, install a JDK and reopen the tool.

## Steps

1. Open sbox and choose **Keystore Generator**.
2. Fill in the form:
   - **Alias** — the key alias inside the keystore.
   - **Store password** / **Key password** — at least 6 characters each, entered twice to confirm.
   - **Validity** — *Never expires* (100 years) by default, or set a custom number of years.
   - **Distinguished name (advanced)** — Common Name is required; OU / Org / Locality / State / Country (defaults to `CN`) are optional.
3. Click generate and choose where to save the `.keystore` file.
4. sbox shows the keystore details and a ready-to-paste block of GitHub Actions secrets.

## GitHub Actions secrets

The generated secret block contains the four values your release workflow needs:

| Secret | Meaning |
| --- | --- |
| `KEYSTORE_BASE64` | The keystore file, Base64-encoded |
| `KEYSTORE_PASSWORD` | The store password |
| `KEY_ALIAS` | The key alias |
| `KEY_PASSWORD` | The key password |

Add them under **Settings → Secrets and variables → Actions** in your repository. Each value has its own **Copy** button.

## Notes

- Keep the `.keystore` file and its passwords safe — losing them means you can no longer ship updates that match the original signature.
- A long validity (the default 100 years) is recommended for app signing keys so they outlive the app.
