> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Overview

Enable memory to make your assistant remember information across conversations. Set `memory="Auto"` to automatically save and retrieve relevant context. This allows your assistant to recall user preferences, previous interactions, and other important details even in new conversation threads.

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
            name="Memory Assistant",
            system_prompt="You are a helpful assistant with persistent memory"
        )
        
        # Create first thread and share information
        thread1 = await client.create_thread(assistant.assistant_id)
        
        # Share information with memory enabled
        response1 = await client.add_message(
            thread_id=thread1.thread_id,
            content="My name is Sarah and I work as a software engineer at Google.",
            memory="Auto",  # Enable memory - automatically saves relevant info
            stream=False
        )
        print(f"AI: {response1.content}")
        
        # Optional: Poll for memory operation completion
        # memory_op_id = response1.memory_operation_id
        # if memory_op_id:
        #     import time
        #     while True:
        #         status_response = requests.get(
        #             f"{base_url}/assistants/memories/operations/{memory_op_id}",
        #             headers={"X-API-Key": api_key}
        #         )
        #         if status_response.status_code == 200:
        #             data = status_response.json()
        #             if data.get("status") in ("COMPLETED", "ERROR"):
        #                 print(f"Memory operation: {data.get('status')}")
        #                 break
        #         time.sleep(1)
        
        # Create a new thread to test memory recall
        thread2 = await client.create_thread(assistant.assistant_id)
        
        # Ask what the assistant remembers (in a completely new thread!)
        response3 = await client.add_message(
            thread_id=thread2.thread_id,
            content="What do you remember about me?",
            memory="Auto",  # Searches and retrieves saved memories
            stream=False
        )
        print(f"AI: {response3.content}")
        # Should mention: Sarah, Google, software engineer

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
            name: 'Memory Assistant',
            systemPrompt: 'You are a helpful assistant with persistent memory'
        });
        
        // Create first thread and share information
        const thread1 = await client.createThread(assistant.assistantId);
        
        // Share information with memory enabled
        const response1 = await client.addMessage({
            threadId: thread1.threadId,
            content: 'My name is Sarah and I work as a software engineer at Google.',
            memory: 'Auto',  // Enable memory - automatically saves relevant info
            stream: false
        });
        console.log(`AI: ${response1.content}`);
        
        // Optional: Poll for memory operation completion
        // const memoryOpId = response1.memoryOperationId;
        // if (memoryOpId) {
        //     while (true) {
        //         const statusResponse = await fetch(
        //             `${baseUrl}/assistants/memories/operations/${memoryOpId}`,
        //             { headers: { 'X-API-Key': apiKey } }
        //         );
        //         if (statusResponse.ok) {
        //             const data = await statusResponse.json();
        //             if (['COMPLETED', 'ERROR'].includes(data.status)) {
        //                 console.log(`Memory operation: ${data.status}`);
        //                 break;
        //             }
        //         }
        //         await new Promise(resolve => setTimeout(resolve, 1000));
        //     }
        // }
        
        // Create a new thread to test memory recall
        const thread2 = await client.createThread(assistant.assistantId);
        
        // Ask what the assistant remembers (in a completely new thread!)
        const response3 = await client.addMessage({
            threadId: thread2.threadId,
            content: 'What do you remember about me?',
            memory: 'Auto',  // Searches and retrieves saved memories
            stream: false
        });
        console.log(`AI: ${response3.content}`);
        // Should mention: Sarah, Google, software engineer
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
            name: 'Memory Assistant',
            systemPrompt: 'You are a helpful assistant with persistent memory'
        });
        
        // Create first thread and share information
        const thread1 = await client.createThread(assistant.assistantId);
        
        // Share information with memory enabled
        const response1 = await client.addMessage({
            threadId: thread1.threadId,
            content: 'My name is Sarah and I work as a software engineer at Google.',
            memory: 'Auto',  // Enable memory - automatically saves relevant info
            stream: false
        });
        console.log(`AI: ${response1.content}`);
        
        // Optional: Poll for memory operation completion
        // const memoryOpId = response1.memoryOperationId;
        // if (memoryOpId) {
        //     while (true) {
        //         const statusResponse = await fetch(
        //             `${baseUrl}/assistants/memories/operations/${memoryOpId}`,
        //             { headers: { 'X-API-Key': apiKey } }
        //         );
        //         if (statusResponse.ok) {
        //             const data = await statusResponse.json();
        //             if (['COMPLETED', 'ERROR'].includes(data.status)) {
        //                 console.log(`Memory operation: ${data.status}`);
        //                 break;
        //             }
        //         }
        //         await new Promise(resolve => setTimeout(resolve, 1000));
        //     }
        // }
        
        // Create a new thread to test memory recall
        const thread2 = await client.createThread(assistant.assistantId);
        
        // Ask what the assistant remembers (in a completely new thread!)
        const response3 = await client.addMessage({
            threadId: thread2.threadId,
            content: 'What do you remember about me?',
            memory: 'Auto',  // Searches and retrieves saved memories
            stream: false
        });
        console.log(`AI: ${response3.content}`);
        // Should mention: Sarah, Google, software engineer
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
            name="Memory Assistant",
            system_prompt="You are a helpful assistant with persistent memory"
        )
        
        # Create first thread and share information
        thread1 = await client.create_thread(assistant.assistant_id)
        
        # Share information with memory enabled (streaming)
        print("AI: ", end="")
        async for chunk in client.add_message(
            thread_id=thread1.thread_id,
            content="My name is Sarah and I work as a software engineer at Google.",
            memory="Auto",  # Enable memory - automatically saves relevant info
            stream=True
        ):
            if chunk.content:
                print(chunk.content, end="", flush=True)
        print()
        
        # Wait a moment for memory to be processed
        await asyncio.sleep(2)
        
        # Create a new thread to test memory recall
        thread2 = await client.create_thread(assistant.assistant_id)
        
        # Ask what the assistant remembers (in a completely new thread!)
        print("AI: ", end="")
        async for chunk in client.add_message(
            thread_id=thread2.thread_id,
            content="What do you remember about me?",
            memory="Auto",  # Searches and retrieves saved memories
            stream=True
        ):
            if chunk.content:
                print(chunk.content, end="", flush=True)
        print()
        # Should mention: Sarah, Google, software engineer

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
            name: 'Memory Assistant',
            systemPrompt: 'You are a helpful assistant with persistent memory'
        });
        
        // Create first thread and share information
        const thread1 = await client.createThread(assistant.assistantId);
        
        // Share information with memory enabled (streaming)
        process.stdout.write('AI: ');
        const stream1 = await client.addMessage({
            threadId: thread1.threadId,
            content: 'My name is Sarah and I work as a software engineer at Google.',
            memory: 'Auto',  // Enable memory - automatically saves relevant info
            stream: true
        });

        for await (const chunk of stream1) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        console.log();
        
        // Wait a moment for memory to be processed
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Create a new thread to test memory recall
        const thread2 = await client.createThread(assistant.assistantId);
        
        // Ask what the assistant remembers (in a completely new thread!)
        process.stdout.write('AI: ');
        const stream2 = await client.addMessage({
            threadId: thread2.threadId,
            content: 'What do you remember about me?',
            memory: 'Auto',  // Searches and retrieves saved memories
            stream: true
        });

        for await (const chunk of stream2) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        console.log();
        // Should mention: Sarah, Google, software engineer
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
            name: 'Memory Assistant',
            systemPrompt: 'You are a helpful assistant with persistent memory'
        });
        
        // Create first thread and share information
        const thread1 = await client.createThread(assistant.assistantId);
        
        // Share information with memory enabled (streaming)
        process.stdout.write('AI: ');
        const stream1 = await client.addMessage({
            threadId: thread1.threadId,
            content: 'My name is Sarah and I work as a software engineer at Google.',
            memory: 'Auto',  // Enable memory - automatically saves relevant info
            stream: true
        });

        for await (const chunk of stream1) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        console.log();
        
        // Wait a moment for memory to be processed
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Create a new thread to test memory recall
        const thread2 = await client.createThread(assistant.assistantId);
        
        // Ask what the assistant remembers (in a completely new thread!)
        process.stdout.write('AI: ');
        const stream2 = await client.addMessage({
            threadId: thread2.threadId,
            content: 'What do you remember about me?',
            memory: 'Auto',  // Searches and retrieves saved memories
            stream: true
        });

        for await (const chunk of stream2) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        console.log();
        // Should mention: Sarah, Google, software engineer
    }

    main();
    ```
  </Tab>
</Tabs>
