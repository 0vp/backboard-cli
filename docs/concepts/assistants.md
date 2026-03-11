> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


An **Assistant** is an AI agent with specific instructions and capabilities. Think of it as a specialized AI persona that you configure once and use across multiple conversations.

## Key Properties

| Property | Description |
|----------|-------------|
| `assistant_id` | Unique identifier for the assistant |
| `name` | Human-readable name for your assistant |
| `system_prompt` | Instructions that define the assistant's behavior and personality |
| `tools` | Optional tools the assistant can use (web search, function calling, etc.) |
| `embedding_provider` | Model provider for RAG and memory operations |
| `embedding_model_name` | Specific embedding model to use |

## Use Cases

<CardGroup cols={2}>
  <Card title="Customer Support Bot" icon="headset">
    Create an assistant with specific product knowledge and support guidelines
  </Card>
  <Card title="Code Review Assistant" icon="code">
    Define coding standards and review criteria in the system prompt
  </Card>
  <Card title="Research Assistant" icon="magnifying-glass">
    Configure with domain expertise and research methodologies
  </Card>
  <Card title="Writing Assistant" icon="pen">
    Set up with specific writing styles and content guidelines
  </Card>
</CardGroup>

## Example

```python
import requests

response = requests.post(
    "https://app.backboard.io/api/assistants",
    headers={"X-API-Key": "your_api_key"},
    json={
        "name": "Technical Support",
        "system_prompt": """You are a technical support assistant for a SaaS product.
        
        Guidelines:
        - Be concise and helpful
        - Always verify the user's issue before suggesting solutions
        - Escalate complex issues to human support when needed
        """,
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "search_knowledge_base",
                    "description": "Search the product knowledge base",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string", "description": "Search query"}
                        },
                        "required": ["query"]
                    }
                }
            }
        ]
    }
)
assistant = response.json()
```

## Embedding Configuration

When creating an assistant, you can configure the embedding model used for document retrieval (RAG) and memory operations:

```python
{
    "name": "My Assistant",
    "embedding_provider": "openai",  # or "cohere", "google"
    "embedding_model_name": "text-embedding-3-large",
    "embedding_dims": 3072
}
```

<Note>
The embedding model cannot be changed after the assistant is created. Choose your embedding configuration carefully based on your needs.
</Note>

## Related Endpoints

- [Create Assistant](/api-reference/assistants/create)
- [List Assistants](/api-reference/assistants/list)
- [Update Assistant](/api-reference/assistants/update)
- [Delete Assistant](/api-reference/assistants/delete)
