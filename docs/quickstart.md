> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Prerequisites

- A Backboard account with an API key
- Python 3.7+ or Node.js 14+ (or any HTTP client)

## Step 1: Get Your API Key

1. Log in to [Backboard Dashboard](https://app.backboard.io)
2. Navigate to Settings → API Keys
3. Create a new API key and copy it

## Step 2: Install the SDK (Optional)

If you prefer using our SDK instead of raw HTTP requests, install it:

<Tabs>
  <Tab title="Python">
    ```bash
    pip install backboard-sdk
    ```
  </Tab>
  <Tab title="JavaScript">
    ```bash
    npm install backboard-sdk
    ```
  </Tab>
  <Tab title="TypeScript">
    ```bash
    npm install backboard-sdk
    ```
  </Tab>
</Tabs>

## Step 3: Create an Assistant

An Assistant is an AI agent with specific instructions. Create one with a simple request:

<Tabs>
  <Tab title="Python SDK">
    ```python
    import asyncio
    from backboard import BackboardClient

    async def main():
        client = BackboardClient(api_key="YOUR_API_KEY")

        assistant = await client.create_assistant(
            name="My First Assistant",
            system_prompt="You are a helpful assistant that responds concisely."
        )
        print(f"Created assistant: {assistant.assistant_id}")

    asyncio.run(main())
    ```
  </Tab>
  <Tab title="JavaScript SDK">
    ```javascript
    const { BackboardClient } = require('backboard-sdk');

    async function main() {
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'You are a helpful assistant that responds concisely.'
        });
        console.log(`Created assistant: ${assistant.assistantId}`);
    }

    main();
    ```
  </Tab>
  <Tab title="TypeScript SDK">
    ```typescript
    import { BackboardClient } from 'backboard-sdk';

    async function main(): Promise<void> {
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'You are a helpful assistant that responds concisely.'
        });
        console.log(`Created assistant: ${assistant.assistantId}`);
    }

    main();
    ```
  </Tab>
  <Tab title="REST API">
    ```python
    import requests

    API_KEY = "your_api_key"
    BASE_URL = "https://app.backboard.io/api"
    HEADERS = {"X-API-Key": API_KEY}

    response = requests.post(
        f"{BASE_URL}/assistants",
        json={
            "name": "My First Assistant",
            "system_prompt": "You are a helpful assistant that responds concisely."
        },
        headers=HEADERS,
    )
    assistant = response.json()
    print(f"Created assistant: {assistant['assistant_id']}")
    ```
  </Tab>
</Tabs>

## Step 4: Create a Thread

A Thread represents a conversation session. Create one for your assistant:

<Tabs>
  <Tab title="Python SDK">
    ```python
    thread = await client.create_thread(assistant.assistant_id)
    print(f"Created thread: {thread.thread_id}")
    ```
  </Tab>
  <Tab title="JavaScript SDK">
    ```javascript
    const thread = await client.createThread(assistant.assistantId);
    console.log(`Created thread: ${thread.threadId}`);
    ```
  </Tab>
  <Tab title="TypeScript SDK">
    ```typescript
    const thread = await client.createThread(assistant.assistantId);
    console.log(`Created thread: ${thread.threadId}`);
    ```
  </Tab>
  <Tab title="REST API">
    ```python
    assistant_id = assistant["assistant_id"]

    response = requests.post(
        f"{BASE_URL}/assistants/{assistant_id}/threads",
        json={},
        headers=HEADERS,
    )
    thread = response.json()
    print(f"Created thread: {thread['thread_id']}")
    ```
  </Tab>
</Tabs>

## Step 5: Send a Message

Now you can send messages and get AI responses:

<Tabs>
  <Tab title="Python SDK">
    ```python
    response = await client.add_message(
        thread_id=thread.thread_id,
        content="What is the capital of France?",
        stream=False
    )
    print(f"Assistant: {response.content}")
    ```
  </Tab>
  <Tab title="JavaScript SDK">
    ```javascript
    const response = await client.addMessage({
        threadId: thread.threadId,
        content: 'What is the capital of France?',
        stream: false
    });
    console.log(`Assistant: ${response.content}`);
    ```
  </Tab>
  <Tab title="TypeScript SDK">
    ```typescript
    const response = await client.addMessage({
        threadId: thread.threadId,
        content: 'What is the capital of France?',
        stream: false
    });
    console.log(`Assistant: ${response.content}`);
    ```
  </Tab>
  <Tab title="REST API">
    ```python
    thread_id = thread["thread_id"]

    response = requests.post(
        f"{BASE_URL}/threads/{thread_id}/messages",
        headers={"X-API-Key": API_KEY},
        data={
            "content": "What is the capital of France?",
            "stream": "false",
            "memory": "Auto"
        },
    )
    result = response.json()
    print(f"Assistant: {result['content']}")
    ```
  </Tab>
</Tabs>

## Step 6: Continue the Conversation

The thread maintains context, so you can have a natural conversation:

<Tabs>
  <Tab title="Python SDK">
    ```python
    response = await client.add_message(
        thread_id=thread.thread_id,
        content="What is its population?",
        stream=False
    )
    print(f"Assistant: {response.content}")  # Will know you're asking about Paris
    ```
  </Tab>
  <Tab title="JavaScript SDK">
    ```javascript
    const response = await client.addMessage({
        threadId: thread.threadId,
        content: 'What is its population?',
        stream: false
    });
    console.log(`Assistant: ${response.content}`);  // Will know you're asking about Paris
    ```
  </Tab>
  <Tab title="TypeScript SDK">
    ```typescript
    const response = await client.addMessage({
        threadId: thread.threadId,
        content: 'What is its population?',
        stream: false
    });
    console.log(`Assistant: ${response.content}`);  // Will know you're asking about Paris
    ```
  </Tab>
  <Tab title="REST API">
    ```python
    response = requests.post(
        f"{BASE_URL}/threads/{thread_id}/messages",
        headers={"X-API-Key": API_KEY}, 
            "content": "What is its population?",
            "stream": "false"
        },
    )
    result = response.json()
    print(f"Assistant: {result['content']}")  # Will know you're asking about Paris
    ```
  </Tab>
</Tabs>

## Next Steps

<CardGroup cols={2}>
  <Card title="Tool Calls" icon="wrench" href="/sdk/tool-calls">
    Define custom functions for your assistant to call
  </Card>
  <Card title="Upload Documents" icon="file" href="/sdk/documents">
    Add context with document uploads
  </Card>
  <Card title="Enable Memory" icon="brain" href="/sdk/memory">
    Let your assistant remember across sessions
  </Card>
  <Card title="Stream Responses" icon="stream" href="/sdk/first-message">
    Get real-time response streaming
  </Card>
</CardGroup>
