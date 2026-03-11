> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.

# Submit Tool Outputs for a Run

> Submit the outputs of tool calls that an assistant message previously requested. This will continue the run. If stream=true, returns a Server-Sent Events stream.

## OpenAPI

```yaml openapi-filtered.json post /threads/{thread_id}/runs/{run_id}/submit-tool-outputs
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
  /threads/{thread_id}/runs/{run_id}/submit-tool-outputs:
    post:
      tags:
        - Threads
      summary: Submit Tool Outputs for a Run
      description: >-
        Submit the outputs of tool calls that an assistant message previously requested. This will continue the run. If
        stream=true, returns a Server-Sent Events stream.
      operationId: submit_tool_outputs_for_run_threads__thread_id__runs__run_id__submit_tool_outputs_post
      security:
        - APIKeyHeader: []
      parameters:
        - name: thread_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Thread Id
        - name: run_id
          in: path
          required: true
          schema:
            type: string
            title: Run Id
        - name: stream
          in: query
          required: false
          schema:
            type: boolean
            default: false
            title: Stream
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/SubmitToolOutputsRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ToolOutputsResponse'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    SubmitToolOutputsRequest:
      properties:
        tool_outputs:
          items:
            $ref: '#/components/schemas/ToolOutput'
          type: array
          title: Tool Outputs
          description: A list of tool outputs to submit.
      type: object
      required:
        - tool_outputs
      title: SubmitToolOutputsRequest
    ToolOutput:
      properties:
        tool_call_id:
          type: string
          title: Tool Call Id
          description: The ID of the tool call this output is for.
        output:
          type: string
          title: Output
          description: The output of the tool call (stringified).
      type: object
      required:
        - tool_call_id
        - output
      title: ToolOutput
    ToolOutputsResponse:
      properties:
        message:
          type: string
          title: Message
        thread_id:
          type: string
          format: uuid
          title: Thread Id
        run_id:
          type: string
          title: Run Id
        content:
          anyOf:
            - type: string
            - type: 'null'
          title: Content
        message_id:
          anyOf:
            - type: string
              format: uuid
            - type: 'null'
          title: Message Id
        role:
          anyOf:
            - $ref: '#/components/schemas/MessageRole'
            - type: 'null'
        status:
          anyOf:
            - $ref: '#/components/schemas/MessageStatus'
            - type: 'null'
        tool_calls:
          anyOf:
            - items:
                additionalProperties: true
                type: object
              type: array
            - type: 'null'
          title: Tool Calls
        memory_operation_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Memory Operation Id
        retrieved_memories:
          anyOf:
            - items:
                $ref: '#/components/schemas/RetrievedMemory'
              type: array
            - type: 'null'
          title: Retrieved Memories
        retrieved_files:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Retrieved Files
        model_provider:
          anyOf:
            - type: string
            - type: 'null'
          title: Model Provider
        model_name:
          anyOf:
            - type: string
            - type: 'null'
          title: Model Name
        input_tokens:
          anyOf:
            - type: integer
            - type: 'null'
          title: Input Tokens
        output_tokens:
          anyOf:
            - type: integer
            - type: 'null'
          title: Output Tokens
        total_tokens:
          anyOf:
            - type: integer
            - type: 'null'
          title: Total Tokens
        created_at:
          anyOf:
            - type: string
              format: date-time
            - type: 'null'
          title: Created At
        timestamp:
          type: string
          format: date-time
          title: Timestamp
      type: object
      required:
        - message
        - thread_id
        - run_id
        - timestamp
      title: ToolOutputsResponse
      description: Response for tool outputs submission
    MessageRole:
      type: string
      enum:
        - user
        - assistant
        - tool
      title: MessageRole
    MessageStatus:
      type: string
      enum:
        - IN_PROGRESS
        - REQUIRES_ACTION
        - COMPLETED
        - FAILED
        - CANCELLED
      title: MessageStatus
    RetrievedMemory:
      properties:
        id:
          anyOf:
            - type: string
            - type: 'null'
          title: Id
        memory:
          type: string
          title: Memory
        score:
          anyOf:
            - type: number
            - type: 'null'
          title: Score
      type: object
      required:
        - memory
      title: RetrievedMemory
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