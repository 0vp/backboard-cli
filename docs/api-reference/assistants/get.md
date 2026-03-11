> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.

# Get Assistant

> Retrieve a specific assistant by its UUID.

## OpenAPI

```yaml openapi-filtered.json get /assistants/{assistant_id}
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
  /assistants/{assistant_id}:
    get:
      tags:
        - Assistants
      summary: Get Assistant
      description: Retrieve a specific assistant by its UUID.
      operationId: get_assistant_assistants__assistant_id__get
      security:
        - APIKeyHeader: []
      parameters:
        - name: assistant_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Assistant Id
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Assistant'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    Assistant:
      properties:
        name:
          type: string
          maxLength: 255
          minLength: 1
          title: Name
          description: Name of the assistant
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Optional system prompt for the assistant
        system_prompt:
          anyOf:
            - type: string
            - type: 'null'
          title: System Prompt
          description: Optional system prompt (alias for description)
        tools:
          anyOf:
            - items:
                $ref: '#/components/schemas/ToolDefinition-Output'
              type: array
            - type: 'null'
          title: Tools
          description: List of tools available to the assistant
        tok_k:
          type: integer
          maximum: 100
          minimum: 1
          title: Tok K
          description: Document search top_k for the internal search_documents tool (default 10).
          default: 10
        embedding_provider:
          anyOf:
            - type: string
            - type: 'null'
          title: Embedding Provider
          description: Embedding provider (openai, google, cohere, etc.)
        embedding_model_name:
          anyOf:
            - type: string
            - type: 'null'
          title: Embedding Model Name
          description: Embedding model name (e.g., text-embedding-3-large, text-embedding-004)
        embedding_dims:
          anyOf:
            - type: integer
            - type: 'null'
          title: Embedding Dims
          description: Embedding dimensions (e.g., 1024 for Cohere, 3072 for OpenAI text-embedding-3-large)
        assistant_id:
          type: string
          format: uuid
          title: Assistant Id
        created_at:
          type: string
          format: date-time
          title: Created At
      type: object
      required:
        - name
        - assistant_id
        - created_at
      title: Assistant
    ToolDefinition-Output:
      properties:
        type:
          type: string
          title: Type
          description: Type of the tool, e.g., 'function'
          default: function
        function:
          $ref: '#/components/schemas/FunctionDefinition-Output'
      type: object
      required:
        - function
      title: ToolDefinition
    FunctionDefinition-Output:
      properties:
        name:
          type: string
          title: Name
          description: Name of the function to be called
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Description of what the function does
        parameters:
          $ref: '#/components/schemas/ToolParameters'
          description: Parameters the function accepts
      type: object
      required:
        - name
        - parameters
      title: FunctionDefinition
    ToolParameters:
      properties:
        type:
          type: string
          title: Type
          description: The type of the parameters object, typically 'object'
          default: object
        properties:
          additionalProperties:
            $ref: '#/components/schemas/ToolParameterProperties'
          type: object
          title: Properties
          description: Parameter definitions
        required:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Required
          description: List of required parameter names
      type: object
      required:
        - properties
      title: ToolParameters
    ToolParameterProperties:
      properties:
        type:
          type: string
          title: Type
          description: Parameter type, e.g., 'string', 'integer', 'object'
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Description of the parameter
        enum:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Enum
          description: Allowed enum values for the parameter
        properties:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Properties
          description: Nested properties for object types
        items:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Items
          description: Defines the schema of array items if type is array
      type: object
      required:
        - type
      title: ToolParameterProperties
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