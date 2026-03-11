> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


**Memory** is an advanced feature that enables assistants to remember facts, preferences, and context across conversations and even across different threads.

## How It Works

1. **Extraction** - Automatically extracts key facts from conversations
2. **Storage** - Stores them in a semantic knowledge base
3. **Retrieval** - Retrieves relevant memories for future messages
4. **Cross-thread** - Works across different threads with the same assistant

## Memory Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `Off` | No persistent memory | Privacy-sensitive conversations |
| `On` | Always saves and retrieves | Maximum context retention |
| `Auto` | Intelligently decides (recommended) | General use |
| `Readonly` | Only retrieves, doesn't save | Consistent behavior with existing memories |

## Example

```python
import requests

thread_id = "your-thread-id"
headers = {"X-API-Key": "your_api_key"}

# Message with memory enabled
response = requests.post(
    f"https://app.backboard.io/api/threads/{thread_id}/messages",
    headers=headers,
    data={
        "content": "I prefer Python over JavaScript for backend development",
        "stream": "false",
        "memory": "Auto"
    }
)

# Later, in a different thread with the same assistant:
# The assistant will remember this preference!
```

## Cross-Thread Memory

```
Thread 1: User mentions "I prefer Python over JavaScript"
         → Memory is automatically saved

Thread 2 (days later): User asks "What language should I use?"
         → Assistant remembers and suggests Python
```

## Managing Memories

### List All Memories

```python
assistant_id = "your-assistant-id"

response = requests.get(
    f"https://app.backboard.io/api/assistants/{assistant_id}/memories",
    headers={"X-API-Key": "your_api_key"}
)

memories = response.json()
for memory in memories["memories"]:
    print(f"Memory: {memory['content']}")
```

### Add a Memory Manually

```python
response = requests.post(
    f"https://app.backboard.io/api/assistants/{assistant_id}/memories",
    headers={"X-API-Key": "your_api_key"},
    json={
        "content": "User is a senior software engineer with 10 years of experience"
    }
)
```

### Delete a Memory

```python
memory_id = "your-memory-id"

response = requests.delete(
    f"https://app.backboard.io/api/assistants/{assistant_id}/memories/{memory_id}",
    headers={"X-API-Key": "your_api_key"}
)
```

## Memory Stats

Check memory usage and limits:

```python
response = requests.get(
    f"https://app.backboard.io/api/assistants/{assistant_id}/memories/stats",
    headers={"X-API-Key": "your_api_key"}
)

stats = response.json()
print(f"Total memories: {stats['total_count']}")
```

<Info>
Memory operations may be asynchronous. Use the operation status endpoint to track the progress of memory additions, updates, and deletions.
</Info>

## Related Endpoints

- [List Memories](/api-reference/memories/list)
- [Add Memory](/api-reference/memories/add)
- [Update Memory](/api-reference/memories/update)
- [Delete Memory](/api-reference/memories/delete)
- [Memory Stats](/api-reference/memories/stats)
