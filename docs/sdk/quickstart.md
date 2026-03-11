> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Getting Started

For a complete step-by-step guide on setting up your API key and sending your first message, see the [Quickstart Guide](/quickstart).

## Installation

<Tabs>
  <Tab title="Python">
    ```bash
    pip install backboard-sdk
    ```
  </Tab>
  <Tab title="JavaScript/TypeScript">
    ```bash
    npm install backboard-sdk
    ```
  </Tab>
</Tabs>

## SDK Guides

Explore detailed examples for each SDK feature:

<CardGroup cols={2}>
  <Card title="First Message" icon="message" href="/sdk/first-message">
    Create an assistant, initialize a thread, and send your first message to get a response.
  </Card>
  <Card title="Tool Calls" icon="wrench" href="/sdk/tool-calls">
    Define custom functions for your assistant to call and handle tool call requests.
  </Card>
  <Card title="Documents" icon="file-lines" href="/sdk/documents">
    Upload documents to your assistant and query their contents.
  </Card>
  <Card title="Memory" icon="brain" href="/sdk/memory">
    Enable persistent memory for context that spans across conversations.
  </Card>
</CardGroup>

## Streaming vs Non-Streaming

Each guide provides examples for both streaming and non-streaming modes:

- **Non-Streaming**: Wait for the complete response before processing. Best for simple use cases where you need the full response at once.
- **Streaming**: Process response chunks as they arrive. Best for real-time applications and better user experience with long responses.

## Language Support

All examples are available in three languages:

- **Python** - Async/await pattern with `BackboardClient`
- **JavaScript** - CommonJS with Promise-based API
- **TypeScript** - Full type safety with interfaces and type definitions
