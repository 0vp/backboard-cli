> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## API Key Authentication

All Backboard API requests require authentication using an API key. Include your API key in the `X-API-Key` header with every request.

```bash
curl -X GET "https://app.backboard.io/api/assistants" \
  -H "X-API-Key: your_api_key"
```

## Getting Your API Key

1. Log in to [Backboard Dashboard](https://app.backboard.io)
2. Navigate to **Settings** → **API Keys**
3. Click **Create New API Key**
4. Copy and securely store your API key

<Warning>
API keys are only shown once when created. If you lose your key, you'll need to generate a new one.
</Warning>

## Using the API Key

<CodeGroup>
```python Python
import requests

headers = {"X-API-Key": "your_api_key"}
response = requests.get(
    "https://app.backboard.io/api/assistants",
    headers=headers
)
```

```javascript Node.js
const response = await fetch("https://app.backboard.io/api/assistants", {
  headers: {
    "X-API-Key": "your_api_key"
  }
});
```

```bash cURL
curl -X GET "https://app.backboard.io/api/assistants" \
  -H "X-API-Key: your_api_key"
```
</CodeGroup>

## Security Best Practices

<AccordionGroup>
  <Accordion title="Never expose your API key in client-side code">
    API keys should only be used in server-side code. Never include them in frontend JavaScript, mobile apps, or any code that runs on the client.
  </Accordion>
  <Accordion title="Use environment variables">
    Store your API key in environment variables rather than hardcoding it in your source code.
    ```bash
    export BACKBOARD_API_KEY="your_api_key"
    ```
  </Accordion>
  <Accordion title="Rotate keys regularly">
    Periodically generate new API keys and revoke old ones, especially if you suspect a key may have been compromised.
  </Accordion>
  <Accordion title="Use separate keys for different environments">
    Create different API keys for development, staging, and production environments.
  </Accordion>
</AccordionGroup>

## Error Responses

If authentication fails, you'll receive a `401 Unauthorized` response:

```json
{
  "detail": "Invalid or missing API key"
}
```
