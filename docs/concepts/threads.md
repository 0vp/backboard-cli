> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


A **Thread** represents a persistent conversation session. It maintains the full context and history of messages between a user and an assistant.

## Key Properties

| Property | Description |
|----------|-------------|
| `thread_id` | Unique identifier for the conversation thread |
| `assistant_id` | The assistant associated with this thread |
| `messages` | Array of messages in the conversation |
| `created_at` | Timestamp when the thread was created |

## Important Notes

- Threads maintain conversation history across multiple API calls
- Each thread is tied to a specific assistant
- You can have multiple threads per assistant (e.g., different users or topics)
- Threads persist indefinitely unless explicitly deleted

## Example Flow

```
User creates Thread A → sends messages → conversation is saved
Days later → same Thread A → assistant remembers full context
```

## Creating a Thread

```python
import requests

assistant_id = "your-assistant-id"

response = requests.post(
    f"https://app.backboard.io/api/assistants/{assistant_id}/threads",
    headers={"X-API-Key": "your_api_key"},
    json={}
)
thread = response.json()
print(f"Thread ID: {thread['thread_id']}")
```

## Retrieving a Thread with Messages

```python
thread_id = "your-thread-id"

response = requests.get(
    f"https://app.backboard.io/api/threads/{thread_id}",
    headers={"X-API-Key": "your_api_key"}
)
thread = response.json()

for message in thread["messages"]:
    print(f"{message['role']}: {message['content']}")
```

## Thread Lifecycle

```mermaid
graph LR
    A[Create Thread] --> B[Send Messages]
    B --> C[Continue Conversation]
    C --> B
    C --> D[Delete Thread]
```

<Info>
Threads don't expire automatically. Delete threads when you no longer need them to manage storage and maintain clean data.
</Info>

## Related Endpoints

- [Create Thread](/api-reference/assistants/create-thread)
- [List Threads](/api-reference/threads/list)
- [Get Thread](/api-reference/threads/get)
- [Delete Thread](/api-reference/threads/delete)
