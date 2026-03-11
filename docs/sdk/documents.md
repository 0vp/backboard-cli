> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Overview

Upload documents to your assistant and query their contents. Documents are automatically indexed and made searchable, enabling your assistant to answer questions based on the uploaded content.

## Non-Streaming

<Tabs>
  <Tab title="Python">
    ```python
    # Install: pip install backboard-sdk
    import asyncio
    import time
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")
        
        # Create an assistant
        assistant = await client.create_assistant(
            name="Document Assistant",
            system_prompt="You are a helpful document analysis assistant"
        )
        
        # Upload a document to the assistant
        document = await client.upload_document_to_assistant(
            assistant.assistant_id,
            "my_document.pdf"
        )
        
        # Wait for the document to be indexed
        print("Waiting for document to be indexed...")
        while True:
            status = await client.get_document_status(document.document_id)
            if status.status == "indexed":
                print("Document indexed successfully!")
                break
            elif status.status == "failed":
                print(f"Document indexing failed: {status.status_message}")
                return
            time.sleep(2)
        
        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)
        
        # Ask a question about the document
        response = await client.add_message(
            thread_id=thread.thread_id,
            content="What are the key points in the uploaded document?",
            stream=False
        )
        
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
            name: 'Document Assistant',
            systemPrompt: 'You are a helpful document analysis assistant'
        });
        
        // Upload a document to the assistant
        const document = await client.uploadDocumentToAssistant(
            assistant.assistantId,
            'my_document.pdf'
        );
        
        // Wait for the document to be indexed
        console.log('Waiting for document to be indexed...');
        while (true) {
            const status = await client.getDocumentStatus(document.documentId);
            if (status.status === 'indexed') {
                console.log('Document indexed successfully!');
                break;
            } else if (status.status === 'failed') {
                console.log(`Document indexing failed: ${status.statusMessage}`);
                return;
            }
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Create a thread
        const thread = await client.createThread(assistant.assistantId);
        
        // Ask a question about the document
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: 'What are the key points in the uploaded document?',
            stream: false
        });
        
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
            name: 'Document Assistant',
            systemPrompt: 'You are a helpful document analysis assistant'
        });
        
        // Upload a document to the assistant
        const document = await client.uploadDocumentToAssistant(
            assistant.assistantId,
            'my_document.pdf'
        );
        
        // Wait for the document to be indexed
        console.log('Waiting for document to be indexed...');
        while (true) {
            const status = await client.getDocumentStatus(document.documentId);
            if (status.status === 'indexed') {
                console.log('Document indexed successfully!');
                break;
            } else if (status.status === 'failed') {
                console.log(`Document indexing failed: ${status.statusMessage}`);
                return;
            }
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Create a thread
        const thread = await client.createThread(assistant.assistantId);
        
        // Ask a question about the document
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: 'What are the key points in the uploaded document?',
            stream: false
        });
        
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
    import time
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")
        
        # Create an assistant
        assistant = await client.create_assistant(
            name="Document Assistant",
            system_prompt="You are a helpful document analysis assistant"
        )
        
        # Upload a document to the assistant
        document = await client.upload_document_to_assistant(
            assistant.assistant_id,
            "my_document.pdf"
        )
        
        # Wait for the document to be indexed
        print("Waiting for document to be indexed...")
        while True:
            status = await client.get_document_status(document.document_id)
            if status.status == "indexed":
                print("Document indexed successfully!")
                break
            elif status.status == "failed":
                print(f"Document indexing failed: {status.status_message}")
                return
            time.sleep(2)
        
        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)
        
        # Ask a question about the document with streaming
        async for chunk in client.add_message(
            thread_id=thread.thread_id,
            content="What are the key points in the uploaded document?",
            stream=True
        ):
            if chunk.content:
                print(chunk.content, end="", flush=True)
        
        print()

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
            name: 'Document Assistant',
            systemPrompt: 'You are a helpful document analysis assistant'
        });
        
        // Upload a document to the assistant
        const document = await client.uploadDocumentToAssistant(
            assistant.assistantId,
            'my_document.pdf'
        );
        
        // Wait for the document to be indexed
        console.log('Waiting for document to be indexed...');
        while (true) {
            const status = await client.getDocumentStatus(document.documentId);
            if (status.status === 'indexed') {
                console.log('Document indexed successfully!');
                break;
            } else if (status.status === 'failed') {
                console.log(`Document indexing failed: ${status.statusMessage}`);
                return;
            }
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Create a thread
        const thread = await client.createThread(assistant.assistantId);
        
        // Ask a question about the document with streaming
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: 'What are the key points in the uploaded document?',
            stream: true
        });

        for await (const chunk of stream) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        
        console.log();
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
            name: 'Document Assistant',
            systemPrompt: 'You are a helpful document analysis assistant'
        });
        
        // Upload a document to the assistant
        const document = await client.uploadDocumentToAssistant(
            assistant.assistantId,
            'my_document.pdf'
        );
        
        // Wait for the document to be indexed
        console.log('Waiting for document to be indexed...');
        while (true) {
            const status = await client.getDocumentStatus(document.documentId);
            if (status.status === 'indexed') {
                console.log('Document indexed successfully!');
                break;
            } else if (status.status === 'failed') {
                console.log(`Document indexing failed: ${status.statusMessage}`);
                return;
            }
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Create a thread
        const thread = await client.createThread(assistant.assistantId);
        
        // Ask a question about the document with streaming
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: 'What are the key points in the uploaded document?',
            stream: true
        });

        for await (const chunk of stream) {
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }
        
        console.log();
    }

    main();
    ```
  </Tab>
</Tabs>
