> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Overview

Get started with Backboard by creating an assistant, initializing a thread, and sending your first message.

## Non-Streaming

<Tabs>
  <Tab title="Python">
    ```python
    # Install: pip install backboard-sdk
    import asyncio
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")

        # Create an assistant
        assistant = await client.create_assistant(
            name="My First Assistant",
            system_prompt="A helpful assistant"
        )

        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)

        # Send a message and get the complete response
        response = await client.add_message(
            thread_id=thread.thread_id,
            content="Hello! Tell me a fun fact about space.",
            llm_provider="openai",
            model_name="gpt-4o",
            stream=False
        )

        # Print the AI's response
        print(response.content)

    if __name__ == "__main__":
        asyncio.run(main())
    ```
  </Tab>
  <Tab title="JavaScript">
    ```javascript
    // Install: npm install backboard-sdk
    const { BackboardClient } = require('backboard-sdk');

    async function main() {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Create an assistant
        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'A helpful assistant'
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message and get the complete response
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: 'Hello! Tell me a fun fact about space.',
            llmProvider: 'openai',
            modelName: 'gpt-4o',
            stream: false
        });

        // Print the AI's response
        console.log(response.content);
    }

    main();
    ```
  </Tab>
  <Tab title="TypeScript">
    ```typescript
    // Install: npm install backboard-sdk
    import { BackboardClient } from 'backboard-sdk';

    async function main(): Promise<void> {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Create an assistant
        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'A helpful assistant'
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message and get the complete response
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: 'Hello! Tell me a fun fact about space.',
            llmProvider: 'openai',
            modelName: 'gpt-4o',
            stream: false
        });

        // Print the AI's response
        console.log(response.content);
    }

    main();
    ```
  </Tab>
</Tabs>

## Streaming

<Tabs>
  <Tab title="Python">
    ```python
    # Install: pip install backboard-sdk
    import asyncio
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")

        # Create an assistant
        assistant = await client.create_assistant(
            name="My First Assistant",
            system_prompt="A helpful assistant"
        )

        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)

        # Send a message and stream the response
        async for chunk in client.add_message(
            thread_id=thread.thread_id,
            content="Hello! Tell me a fun fact about space.",
            llm_provider="openai",
            model_name="gpt-4o",
            stream=True
        ):
            # Print each chunk as it arrives
            if chunk.content:
                print(chunk.content, end="", flush=True)
        
        print()  # New line after streaming completes

    if __name__ == "__main__":
        asyncio.run(main())
    ```
  </Tab>
  <Tab title="JavaScript">
    ```javascript
    // Install: npm install backboard-sdk
    const { BackboardClient } = require('backboard-sdk');

    async function main() {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Create an assistant
        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'A helpful assistant'
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message and stream the response
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: 'Hello! Tell me a fun fact about space.',
            llmProvider: 'openai',
            modelName: 'gpt-4o',
            stream: true
        });

        // Process each chunk as it arrives
        for await (const chunk of stream) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        
        console.log(); // New line after streaming completes
    }

    main();
    ```
  </Tab>
  <Tab title="TypeScript">
    ```typescript
    // Install: npm install backboard-sdk
    import { BackboardClient } from 'backboard-sdk';

    async function main(): Promise<void> {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Create an assistant
        const assistant = await client.createAssistant({
            name: 'My First Assistant',
            systemPrompt: 'A helpful assistant'
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message and stream the response
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: 'Hello! Tell me a fun fact about space.',
            llmProvider: 'openai',
            modelName: 'gpt-4o',
            stream: true
        });

        // Process each chunk as it arrives
        for await (const chunk of stream) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        
        console.log(); // New line after streaming completes
    }

    main();
    ```
  </Tab>
</Tabs>
