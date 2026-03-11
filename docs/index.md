> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


# Welcome to Backboard API

Backboard API enables you to build powerful conversational AI applications with:

- **Persistent Conversations** - Create conversation threads that maintain context across multiple messages
- **Intelligent Document Processing** - Upload and process documents (PDF, text, Office files) with automatic chunking and indexing
- **AI Assistants** - Create specialized assistants with custom instructions, document access, and tool capabilities
- **Memory** - Enable assistants to remember facts and preferences across conversations

## Base URL

```
https://app.backboard.io/api
```

## Quick Example

```python
import requests

API_KEY = "your_api_key"
BASE_URL = "https://app.backboard.io/api"
HEADERS = {"X-API-Key": API_KEY}

# 1) Create assistant
response = requests.post(
    f"{BASE_URL}/assistants",
    json={"name": "Support Bot", "system_prompt": "You are a helpful assistant."},
    headers=HEADERS,
)
assistant_id = response.json()["assistant_id"]

# 2) Create thread
response = requests.post(
    f"{BASE_URL}/assistants/{assistant_id}/threads",
    json={},
    headers=HEADERS,
)
thread_id = response.json()["thread_id"]

# 3) Send message
response = requests.post(
    f"{BASE_URL}/threads/{thread_id}/messages",
    headers=HEADERS,
    data={"content": "Hello!", "stream": "false"},
)
print(response.json().get("content"))
```

## Architecture Overview

```
╔═══════════════════════════════════════════════╗
║                   Assistant                   ║
║                                               ║
║   ╔══════════╗  ╔══════════╗  ╔══════════╗    ║
║   ║ Thread 1 ║  ║ Thread 2 ║  ║ Thread N ║    ║
║   ╚══════════╝  ╚══════════╝  ╚══════════╝    ║
║        ⇡            ⇡             ⇡           ║
║        ┊            ┊             ┊           ║
║        ┊            ┊             ┊           ║
║   ╔══════════════════════════════════════╗    ║
║   ║          Shared Memory               ║    ║
║   ╚══════════════════════════════════════╝    ║
╚═══════════════════════════════════════════════╝
```

## Next Steps

<CardGroup cols={2}>
  <Card title="Quickstart" icon="rocket" href="/quickstart">
    Get up and running in 5 minutes
  </Card>
  <Card title="Authentication" icon="key" href="/authentication">
    Learn how to authenticate your requests
  </Card>
  <Card title="Core Concepts" icon="book" href="/concepts/assistants">
    Understand the API architecture
  </Card>
  <Card title="API Reference" icon="code" href="/api-reference/assistants/create">
    Explore all available endpoints
  </Card>
</CardGroup>
