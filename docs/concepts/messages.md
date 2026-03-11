> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


A **Message** is a single interaction within a thread - either from the user or the assistant's response.

## Key Properties

| Property | Description |
|----------|-------------|
| `content` | The text content of the message |
| `role` | Either `user`, `assistant`, or `tool` |
| `thread_id` | Which thread this message belongs to |
| `message_id` | Unique identifier for the message |
| `stream` | Whether to stream the response |
| `memory` | Memory mode: `Auto`, `On`, `Off`, or `Readonly` |

## Sending a Message

```python
import requests

thread_id = "your-thread-id"

response = requests.post(
    f"https://app.backboard.io/api/threads/{thread_id}/messages",
    headers={"X-API-Key": "your_api_key"},
    data={
        "content": "What is machine learning?",
        "stream": "false",
        "memory": "Auto"
    }
)
result = response.json()
print(f"Assistant: {result['content']}")
```

## Streaming vs Non-Streaming

<Tabs>
  <Tab title="Non-Streaming">
    Wait for the complete response. Simpler to implement, best for batch processing.
    
    ```python
    response = requests.post(
        f"{BASE_URL}/threads/{thread_id}/messages",
        headers=headers,
        data={"content": "Hello", "stream": "false"}
    )
    print(response.json()["content"])
    ```
  </Tab>
  <Tab title="Streaming">
    Receive response in real-time chunks. Better UX for chat interfaces.
    
    ```python
    response = requests.post(
        f"{BASE_URL}/threads/{thread_id}/messages",
        headers=headers,
        data={"content": "Hello", "stream": "true"},
        stream=True
    )
    for line in response.iter_lines():
        if line:
            print(line.decode())
    ```
  </Tab>
</Tabs>

## Message with Attachments

You can attach files directly when sending a message:

```python
files = [
    ("files", open("document.pdf", "rb"))
]

response = requests.post(
    f"https://app.backboard.io/api/threads/{thread_id}/messages",
    headers={"X-API-Key": "your_api_key"},
    data={
        "content": "Can you summarize this document?",
        "stream": "false"
    },
    files=files
)
```

## Model Selection

You can specify which LLM to use for each message:

```python
response = requests.post(
    f"https://app.backboard.io/api/threads/{thread_id}/messages",
    headers={"X-API-Key": "your_api_key"},
    data={
        "content": "Explain quantum computing",
        "llm_provider": "anthropic",
        "model_name": "claude-3-5-sonnet-20241022",
        "stream": "false"
    }
)
```

<Info>
If not specified, defaults to `openai` provider with `gpt-4o` model.
</Info>

## Memory Modes

| Mode | Description |
|------|-------------|
| `Off` | No persistent memory, only uses conversation history |
| `On` | Explicitly saves and retrieves memories |
| `Auto` | Intelligently determines when to use memory (recommended) |
| `Readonly` | Only retrieves memories, doesn't create new ones |

## Related Endpoints

- [Add Message](/api-reference/threads/add-message)
- [Submit Tool Outputs](/api-reference/threads/submit-tool-outputs)
