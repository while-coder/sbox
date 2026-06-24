# XiaoAI Login

Sign in to your Mi account and export the credentials that sbot's `channel.xiaoai` needs to talk to your XiaoAI speakers.

## What you get

| Field | Where it comes from |
| --- | --- |
| `userId` | Your Mi account ID |
| `passToken` | Long-lived login token |
| `deviceId` | Login device identifier |

Plus, for **each speaker** found on the account:

| Field | Description |
| --- | --- |
| `name` | Speaker model name |
| `deviceID` | Speaker device ID |
| `miotDID` | MIoT device ID |

## Steps

1. Open sbox and choose **XiaoAI Login**.
2. Click **Open Mi Login**. The official Mi login page opens in a window.
   - Account + password, SMS code, QR scan and captcha are all supported — just follow the Mi page.
   - If it auto-logs into a previously used account, click **Clear login cache** first and try again.
3. After a successful login, sbox automatically fetches the speaker list.
4. Each field has its own **Copy** button — copy what you need.

## Notes

- If login succeeds but the speaker list fails to load, you'll still have the account credentials; retry the fetch.
- If the account has no XiaoAI speakers, you'll see *"No XiaoAI speaker found on this account"*.
- Use **Clear login cache** / **Switch account** to log in with a different Mi account.

## Using the values in sbot

Fill the corresponding fields in your `channel.xiaoai` config (or paste them into sbot admin). At minimum `userId`, `passToken`, `deviceId`, plus the target speaker's `deviceID` / `miotDID`.
