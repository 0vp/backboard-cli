> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## Overview

Define custom functions (tools) for your assistant to call. When the assistant determines it needs to use a tool, it will return a `REQUIRES_ACTION` status with the tool calls. You then execute the function and submit the outputs back to continue the conversation.

## Non-Streaming

<Tabs>
  <Tab title="Python">
    ```python
    # Install: pip install backboard-sdk
    import asyncio
    import json
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")

        # Define a tool (function) for the assistant to call
        tools = [{
            "type": "function",
            "function": {
                "name": "get_current_weather",
                "description": "Get current weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string", "description": "City name"}
                    },
                    "required": ["location"]
                }
            }
        }]

        # Create an assistant with the tool
        assistant = await client.create_assistant(
            name="Weather Assistant",
            system_prompt="You are a helpful weather assistant",
            tools=tools
        )

        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)

        # Send a message that triggers the tool call
        response = await client.add_message(
            thread_id=thread.thread_id,
            content="What's the weather in San Francisco?",
            stream=False
        )

        # Check if the assistant requires action (tool call)
        if response.status == "REQUIRES_ACTION" and response.tool_calls:
            tool_outputs = []
            
            # Process each tool call
            for tc in response.tool_calls:
                if tc.function.name == "get_current_weather":
                    # Get parsed arguments (required parameters are guaranteed by API)
                    args = tc.function.parsed_arguments
                    location = args["location"]
                    
                    # Execute your function and format the output
                    weather_data = {
                        "temperature": "68°F",
                        "condition": "Sunny",
                        "location": location
                    }
                    
                    tool_outputs.append({
                        "tool_call_id": tc.id,
                        "output": json.dumps(weather_data)
                    })

            # Submit the tool outputs back to continue the conversation
            final_response = await client.submit_tool_outputs(
                thread_id=thread.thread_id,
                run_id=response.run_id,
                tool_outputs=tool_outputs
            )

            print(final_response.content)

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

        // Define a tool (function) for the assistant to call
        const tools = [{
            type: 'function',
            function: {
                name: 'get_current_weather',
                description: 'Get current weather for a location',
                parameters: {
                    type: 'object',
                    properties: {
                        location: { type: 'string', description: 'City name' }
                    },
                    required: ['location']
                }
            }
        }];

        // Create an assistant with the tool
        const assistant = await client.createAssistant({
            name: 'Weather Assistant',
            systemPrompt: 'You are a helpful weather assistant',
            tools: tools
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message that triggers the tool call
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: "What's the weather in San Francisco?",
            stream: false
        });

        // Check if the assistant requires action (tool call)
        if (response.status === 'REQUIRES_ACTION' && response.toolCalls) {
            const toolOutputs = [];
            
            // Process each tool call
            for (const tc of response.toolCalls) {
                if (tc.function.name === 'get_current_weather') {
                    // Get parsed arguments
                    const args = tc.function.parsedArguments;
                    const location = args.location;
                    
                    // Execute your function and format the output
                    const weatherData = {
                        temperature: '68°F',
                        condition: 'Sunny',
                        location: location
                    };
                    
                    toolOutputs.push({
                        toolCallId: tc.id,
                        output: JSON.stringify(weatherData)
                    });
                }
            }

            // Submit the tool outputs back to continue the conversation
            const finalResponse = await client.submitToolOutputs({
                threadId: thread.threadId,
                runId: response.runId,
                toolOutputs: toolOutputs
            });

            console.log(finalResponse.content);
        }
    }

    main();
    ```
  </Tab>
  <Tab title="TypeScript">
    ```typescript
    // Install: npm install backboard-sdk
    import { BackboardClient, Tool, ToolOutput } from 'backboard-sdk';

    async function main(): Promise<void> {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Define a tool (function) for the assistant to call
        const tools: Tool[] = [{
            type: 'function',
            function: {
                name: 'get_current_weather',
                description: 'Get current weather for a location',
                parameters: {
                    type: 'object',
                    properties: {
                        location: { type: 'string', description: 'City name' }
                    },
                    required: ['location']
                }
            }
        }];

        // Create an assistant with the tool
        const assistant = await client.createAssistant({
            name: 'Weather Assistant',
            systemPrompt: 'You are a helpful weather assistant',
            tools: tools
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message that triggers the tool call
        const response = await client.addMessage({
            threadId: thread.threadId,
            content: "What's the weather in San Francisco?",
            stream: false
        });

        // Check if the assistant requires action (tool call)
        if (response.status === 'REQUIRES_ACTION' && response.toolCalls) {
            const toolOutputs: ToolOutput[] = [];
            
            // Process each tool call
            for (const tc of response.toolCalls) {
                if (tc.function.name === 'get_current_weather') {
                    // Get parsed arguments
                    const args = tc.function.parsedArguments as { location: string };
                    const location = args.location;
                    
                    // Execute your function and format the output
                    const weatherData = {
                        temperature: '68°F',
                        condition: 'Sunny',
                        location: location
                    };
                    
                    toolOutputs.push({
                        toolCallId: tc.id,
                        output: JSON.stringify(weatherData)
                    });
                }
            }

            // Submit the tool outputs back to continue the conversation
            const finalResponse = await client.submitToolOutputs({
                threadId: thread.threadId,
                runId: response.runId,
                toolOutputs: toolOutputs
            });

            console.log(finalResponse.content);
        }
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
    import json
    from backboard import BackboardClient

    async def main():
        # Initialize the Backboard client
        client = BackboardClient(api_key="YOUR_API_KEY")

        # Define a tool (function) for the assistant to call
        tools = [{
            "type": "function",
            "function": {
                "name": "get_current_weather",
                "description": "Get current weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string", "description": "City name"}
                    },
                    "required": ["location"]
                }
            }
        }]

        # Create an assistant with the tool
        assistant = await client.create_assistant(
            name="Weather Assistant",
            system_prompt="You are a helpful weather assistant",
            tools=tools
        )

        # Create a thread
        thread = await client.create_thread(assistant.assistant_id)

        # Send a message that triggers the tool call (streaming)
        response = None
        async for chunk in client.add_message(
            thread_id=thread.thread_id,
            content="What's the weather in San Francisco?",
            stream=True
        ):
            response = chunk
            if chunk.content:
                print(chunk.content, end="", flush=True)

        # Check if the assistant requires action (tool call)
        if response and response.status == "REQUIRES_ACTION" and response.tool_calls:
            tool_outputs = []
            
            # Process each tool call
            for tc in response.tool_calls:
                if tc.function.name == "get_current_weather":
                    args = tc.function.parsed_arguments
                    location = args["location"]
                    
                    weather_data = {
                        "temperature": "68°F",
                        "condition": "Sunny",
                        "location": location
                    }
                    
                    tool_outputs.append({
                        "tool_call_id": tc.id,
                        "output": json.dumps(weather_data)
                    })

            # Submit the tool outputs and stream the final response
            async for chunk in client.submit_tool_outputs(
                thread_id=thread.thread_id,
                run_id=response.run_id,
                tool_outputs=tool_outputs,
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

        // Define a tool (function) for the assistant to call
        const tools = [{
            type: 'function',
            function: {
                name: 'get_current_weather',
                description: 'Get current weather for a location',
                parameters: {
                    type: 'object',
                    properties: {
                        location: { type: 'string', description: 'City name' }
                    },
                    required: ['location']
                }
            }
        }];

        // Create an assistant with the tool
        const assistant = await client.createAssistant({
            name: 'Weather Assistant',
            systemPrompt: 'You are a helpful weather assistant',
            tools: tools
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message that triggers the tool call (streaming)
        let response = null;
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: "What's the weather in San Francisco?",
            stream: true
        });

        for await (const chunk of stream) {
            response = chunk;
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }

        // Check if the assistant requires action (tool call)
        if (response?.status === 'REQUIRES_ACTION' && response?.toolCalls) {
            const toolOutputs = [];
            
            // Process each tool call
            for (const tc of response.toolCalls) {
                if (tc.function.name === 'get_current_weather') {
                    const args = tc.function.parsedArguments;
                    const location = args.location;
                    
                    const weatherData = {
                        temperature: '68°F',
                        condition: 'Sunny',
                        location: location
                    };
                    
                    toolOutputs.push({
                        toolCallId: tc.id,
                        output: JSON.stringify(weatherData)
                    });
                }
            }

            // Submit the tool outputs and stream the final response
            const finalStream = await client.submitToolOutputs({
                threadId: thread.threadId,
                runId: response.runId,
                toolOutputs: toolOutputs,
                stream: true
            });

            for await (const chunk of finalStream) {
                if (chunk.content) {
                    process.stdout.write(chunk.content);
                }
            }
            
            console.log();
        }
    }

    main();
    ```
  </Tab>
  <Tab title="TypeScript">
    ```typescript
    // Install: npm install backboard-sdk
    import { BackboardClient, Tool, ToolOutput, MessageResponse } from 'backboard-sdk';

    async function main(): Promise<void> {
        // Initialize the Backboard client
        const client = new BackboardClient({ apiKey: 'YOUR_API_KEY' });

        // Define a tool (function) for the assistant to call
        const tools: Tool[] = [{
            type: 'function',
            function: {
                name: 'get_current_weather',
                description: 'Get current weather for a location',
                parameters: {
                    type: 'object',
                    properties: {
                        location: { type: 'string', description: 'City name' }
                    },
                    required: ['location']
                }
            }
        }];

        // Create an assistant with the tool
        const assistant = await client.createAssistant({
            name: 'Weather Assistant',
            systemPrompt: 'You are a helpful weather assistant',
            tools: tools
        });

        // Create a thread
        const thread = await client.createThread(assistant.assistantId);

        // Send a message that triggers the tool call (streaming)
        let response: MessageResponse | null = null;
        const stream = await client.addMessage({
            threadId: thread.threadId,
            content: "What's the weather in San Francisco?",
            stream: true
        });

        for await (const chunk of stream) {
            response = chunk;
            if (chunk.content) {
                process.stdout.write(chunk.content);
            }
        }

        // Check if the assistant requires action (tool call)
        if (response?.status === 'REQUIRES_ACTION' && response?.toolCalls) {
            const toolOutputs: ToolOutput[] = [];
            
            // Process each tool call
            for (const tc of response.toolCalls) {
                if (tc.function.name === 'get_current_weather') {
                    const args = tc.function.parsedArguments as { location: string };
                    const location = args.location;
                    
                    const weatherData = {
                        temperature: '68°F',
                        condition: 'Sunny',
                        location: location
                    };
                    
                    toolOutputs.push({
                        toolCallId: tc.id,
                        output: JSON.stringify(weatherData)
                    });
                }
            }

            // Submit the tool outputs and stream the final response
            const finalStream = await client.submitToolOutputs({
                threadId: thread.threadId,
                runId: response.runId,
                toolOutputs: toolOutputs,
                stream: true
            });

            for await (const chunk of finalStream) {
                if (chunk.content) {
                    process.stdout.write(chunk.content);
                }
            }
            
            console.log();
        }
    }

    main();
    ```
  </Tab>
</Tabs>
