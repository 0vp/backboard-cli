> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.

# Get Embedding Model by Name

> Get detailed information about a specific embedding model by its name.

## OpenAPI

```yaml openapi-filtered.json get /models/embedding/{model_name}
openapi: 3.1.0
info:
  title: Backboard API
  description: >-

    # Welcome to Backboard API


    Build conversational AI applications with persistent memory and intelligent document processing.


    ## Endpoint URL

    ```

    https://app.backboard.io/api

    ```


    ## API Architecture


    Understanding the core concepts of Backboard API will help you build powerful conversational AI applications.


    ### **Assistant**

    An **Assistant** is an AI agent with specific instructions and capabilities. Think of it as a specialized AI persona
    that you configure once and use across multiple conversations.


    **Key Properties:**

    - `assistant_id` - Unique identifier for the assistant

    - `name` - Human-readable name for your assistant

    - `system_prompt` - Instructions that define the assistant's behavior and personality

    - `llm_provider` - AI provider (e.g., "openai", "anthropic", "google")

    - `model_name` - Specific model to use (e.g., "gpt-4o", "claude-3-5-sonnet-20241022")

    - `tools` - Optional tools the assistant can use (web search, function calling, etc.)

    - `embedding_provider` & `embedding_model_name` - Models used for RAG and memory operations


    **Use Cases:**

    - Customer support bot with specific product knowledge

    - Code review assistant with particular coding standards

    - Research assistant with domain expertise


    ---


    ### **Thread**

    A **Thread** represents a persistent conversation session. It maintains the full context and history of messages
    between a user and an assistant.


    **Key Properties:**

    - `thread_id` - Unique identifier for the conversation thread

    - `assistant_id` - The assistant associated with this thread

    - Messages are automatically stored and retrieved within the thread


    **Important Notes:**

    - Threads maintain conversation history across multiple API calls

    - Each thread is tied to a specific assistant

    - You can have multiple threads per assistant (e.g., different users or topics)

    - Threads persist indefinitely unless explicitly deleted


    **Example Flow:**

    ```

    User creates Thread A → sends messages → conversation is saved

    Days later → same Thread A → assistant remembers full context

    ```


    ---


    ### **Message**

    A **Message** is a single interaction within a thread - either from the user or the assistant's response.


    **Key Properties:**

    - `content` - The text content of the message

    - `role` - Either "user" or "assistant"

    - `thread_id` - Which thread this message belongs to

    - `stream` - Whether to stream the response (true/false)

    - `memory` - Memory mode: "Auto", "On", "Off" (controls persistent memory features)


    **Streaming vs Non-Streaming:**

    - **Non-streaming**: Wait for the complete response (simpler, use for batch processing)

    - **Streaming**: Receive response in real-time chunks (better UX for chat interfaces)


    ---


    ### **Document**

    **Documents** are files you upload to provide context to your assistant. They can be attached at the assistant level
    (available to all threads) or thread level (specific conversation only).


    **Key Properties:**

    - `document_id` - Unique identifier for the document

    - `filename` - Original filename

    - `status` - Processing status: "processing", "completed", "error"

    - `assistant_id` or `thread_id` - Where the document is attached


    **Supported Formats:**

    - PDF documents

    - Text files (.txt, .md)

    - Microsoft Office (.docx, .xlsx, .pptx)

    - CSV and JSON files

    - Source code files


    **Processing Pipeline:**

    1. Upload document via API

    2. Backboard chunks and indexes the content

    3. Status changes from "processing" to "completed"

    4. Document content is available for RAG (Retrieval-Augmented Generation)


    ---


    ### **Memory**

    **Memory** is an advanced feature that enables assistants to remember facts, preferences, and context across
    conversations and even across different threads.


    **Memory Modes:**

    - **"Off"**: No persistent memory, only uses conversation history

    - **"On"**: Explicitly saves and retrieves memories for context

    - **"Auto"**: Intelligently determines when to use memory (recommended)


    **How It Works:**

    - Automatically extracts key facts from conversations

    - Stores them in a semantic knowledge base

    - Retrieves relevant memories for future messages

    - Works across different threads with the same assistant


    **Example:**

    ```

    Thread 1: User mentions "I prefer Python over JavaScript"

    Thread 2 (days later): Assistant remembers this preference

    ```


    ---


    ## Typical Workflow


    Here's how these components work together:


    ```

    1. Create an Assistant
       └─ Define behavior, choose model, configure tools
       
    2. Create a Thread (per conversation/user)
       └─ Links to the assistant you created
       
    3. Upload Documents (optional)
       └─ Attach to assistant (all threads) or specific thread
       
    4. Send Messages
       └─ User messages → Assistant responses
       └─ Conversation history is automatically maintained
       
    5. Memory (optional)
       └─ Enable with memory="Auto" to persist learnings
    ```


    ---


    ## Core Features


    ### **Persistent Conversations**

    Create conversation threads that maintain context across multiple messages and support file attachments.


    ### **Intelligent Document Processing**

    Upload and process documents (PDF, text, Office files) with automatic chunking and indexing for retrieval.


    ### **AI Assistants**

    Create specialized assistants with custom instructions, document access, and tool capabilities.


    ## Quickstart


    ```python

    import requests


    API_KEY = "your_api_key"

    BASE_URL = "https://app.backboard.io/api"

    HEADERS = {"X-API-Key": API_KEY}


    # 1) Create assistant

    response = requests.post(
        f"{BASE_URL}/assistants",
        json={"name": "Support Bot", "system_prompt": "After every response, pass a joke at the end of the response!"},
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
        data={"content": "Tell me about Canada in detail.", "stream": "false", "memory": "Auto"},
    )

    print(response.json().get("content"))

    ```


    ---


    Explore the **Assistants**, **Threads**, and **Documents** sections in the sidebar.
        
  version: 1.0.0
security:
  - APIKeyHeader: []
paths:
  /models/embedding/{model_name}:
    get:
      tags:
        - Models
      summary: Get Embedding Model by Name
      description: Get detailed information about a specific embedding model by its name.
      operationId: get_embedding_model_by_name_models_embedding__model_name__get
      security:
        - APIKeyHeader: []
      parameters:
        - name: model_name
          in: path
          required: true
          schema:
            type: string
            title: Model Name
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/EmbeddingModelRead'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    EmbeddingModelRead:
      properties:
        name:
          type: string
          title: Name
          description: Name of the embedding model
        provider:
          type: string
          title: Provider
          description: Provider of the model (e.g., openai, cohere, google)
        embedding_dimensions:
          type: integer
          title: Embedding Dimensions
          description: Dimension of the embedding vectors
        context_limit:
          type: integer
          title: Context Limit
          description: Maximum context window size in tokens
        last_updated:
          anyOf:
            - type: string
              format: date-time
            - type: 'null'
          title: Last Updated
          description: Last time the model was updated
      type: object
      required:
        - name
        - provider
        - embedding_dimensions
        - context_limit
      title: EmbeddingModelRead
      description: Schema specifically for reading embedding model information.
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ValidationError:
      properties:
        loc:
          items:
            anyOf:
              - type: string
              - type: integer
          type: array
          title: Location
        msg:
          type: string
          title: Message
        type:
          type: string
          title: Error Type
      type: object
      required:
        - loc
        - msg
        - type
      title: ValidationError
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-API-Key
```