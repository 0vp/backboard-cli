> ## Documentation Index
> Fetch the complete documentation index at: https://backboard-docs.docsalot.dev/llms.txt
> Use this file to discover all available pages before exploring further.


## General

<AccordionGroup>
  <Accordion title="What is Backboard?">
    Backboard is a unified AI infrastructure platform that provides access to 2,200+ LLMs, built-in RAG, and the world's smartest memory system—all through a single API. It enables developers to build stateful, context-aware AI applications without managing complex infrastructure.
  </Accordion>

  <Accordion title="How does Backboard's memory work?">
    Backboard uses a sophisticated hybrid storage system to manage and retrieve memories for AI assistants. When you send a message with `memory="Auto"`, the system automatically extracts relevant facts and preferences, storing them across optimized data stores.

    When your assistant needs to access memories, Backboard conducts a comprehensive search and retrieves relevant information that can be seamlessly integrated into responses—enabling personalized, context-aware interactions across conversations.
  </Accordion>

  <Accordion title="How is Backboard different from traditional RAG?">
    Backboard's memory implementation offers several advantages over traditional Retrieval-Augmented Generation:

    - **Entity Relationships**: Understands and relates entities across different interactions, not just static documents
    - **Contextual Continuity**: Retains information across sessions, maintaining continuity in long-term engagements
    - **Adaptive Learning**: Improves personalization based on user interactions and feedback
    - **Dynamic Updates**: Updates memory in real-time with new information, unlike static RAG systems
  </Accordion>

  <Accordion title="What are common use cases for Backboard?">
    - **Personalized Learning Assistants**: Remember user preferences, strengths, weaknesses, and progress
    - **Customer Support AI Agents**: Retain context from previous interactions for better assistance
    - **Healthcare Assistants**: Track patient history, medication schedules, and treatment plans
    - **Virtual Companions**: Build deeper relationships by remembering personal details and past conversations
    - **Productivity Tools**: Remember user habits, frequently used documents, and task history
    - **Gaming AI**: Create immersive experiences by remembering player choices and progress
  </Accordion>
</AccordionGroup>

## Getting Started

<AccordionGroup>
  <Accordion title="How do I get started with Backboard?">
    1. Sign up at [app.backboard.io](https://app.backboard.io/signup)
    2. Navigate to Settings → API Keys to create your API key
    3. Install the SDK (`pip install backboard-sdk` or `npm install backboard-sdk`)
    4. Follow our [Quickstart Guide](/quickstart) to send your first message
  </Accordion>

  <Accordion title="Do I need a credit card to get started?">
    No. New accounts receive **$10 in free credits** to explore the platform. You can start building immediately without entering payment information.
  </Accordion>

  <Accordion title="What SDKs are available?">
    Backboard provides official SDKs for:
    - **Python**: `pip install backboard-sdk`
    - **JavaScript/TypeScript**: `npm install backboard-sdk`

    You can also integrate directly via our REST API using any HTTP client.
  </Accordion>
</AccordionGroup>

## Memory

<AccordionGroup>
  <Accordion title="How do I enable memory for my assistant?">
    Simply set `memory="Auto"` when sending a message. Backboard automatically extracts and stores relevant information, then retrieves it when needed in future conversations.

    ```python
    response = await client.add_message(
        thread_id=thread.thread_id,
        content="My name is Sarah and I work at Google.",
        memory="Auto"
    )
    ```
  </Accordion>

  <Accordion title="Why aren't my memories being created?">
    Backboard uses a classification system to determine which content should be stored as memories. Not all text will generate memories. Memories are typically not created for:

    - Definitional questions (e.g., "What is machine learning?")
    - General concept explanations without personal context
    - Technical definitions and theoretical explanations
    - Abstract or theoretical content

    **Best practices for memory extraction:**
    - Include temporal markers (when events occurred)
    - Add personal context or experiences
    - Frame information in terms of real-world applications
    - Include specific examples rather than general definitions
  </Accordion>

  <Accordion title="Can I manage memories manually?">
    Yes. You can view, enable, disable, or delete individual memories through the dashboard or via the Memory API endpoints. This gives you granular control over what your assistant remembers.
  </Accordion>

  <Accordion title="Do memories persist across threads?">
    Yes. Memories are stored at the assistant level, not the thread level. This means information saved in one conversation thread is accessible in all other threads for the same assistant.
  </Accordion>
</AccordionGroup>

## Models and Providers

<AccordionGroup>
  <Accordion title="What models does Backboard support?">
    Backboard provides access to over **2,200 models** from leading providers including:
    - OpenAI (GPT-4o, GPT-4, GPT-3.5)
    - Anthropic (Claude 3.5, Claude 3)
    - Google (Gemini Pro, Gemini Ultra)
    - Meta (Llama 3)
    - X.ai (Grok)
    - OpenRouter (frontier and free models)
    - Cerebras (optimized for speed)
  </Accordion>

  <Accordion title="Can I switch models mid-conversation?">
    Yes. Backboard's stateful thread management supports on-demand model switching across all 2,200+ models. You can specify different models for different messages within the same thread.
  </Accordion>

  <Accordion title="Do I need separate API keys for each provider?">
    No. Backboard provides unified access to all models through a single API key. You don't need to manage separate credentials for each provider.
  </Accordion>
</AccordionGroup>

## Documents and RAG

<AccordionGroup>
  <Accordion title="What document formats are supported?">
    Backboard supports common document formats including PDF, TXT, and other text-based files. Documents are automatically indexed and made searchable for RAG queries.
  </Accordion>

  <Accordion title="How does the RAG system work?">
    Backboard's agentic RAG system uses hybrid search combining BM25 and vector retrieval. It automatically chunks documents based on the model's context window, achieving low-latency retrieval while seamlessly scaling to accommodate additional documents.
  </Accordion>

  <Accordion title="Can I use my own vector database?">
    Yes. While Backboard provides built-in RAG capabilities, you can integrate your own vector database and retrieval system if needed.
  </Accordion>
</AccordionGroup>

## Pricing and Billing

<AccordionGroup>
  <Accordion title="How does pricing work?">
    Backboard uses a credit-based system. You pay for the tokens and resources you consume. New accounts start with $10 in free credits. Visit [app.backboard.io](https://app.backboard.io) for current pricing details.
  </Accordion>

  <Accordion title="Can I share credits across a team?">
    Yes. Backboard supports multi-user accounts, allowing you to share credits across multiple users under one billing profile.
  </Accordion>
</AccordionGroup>

## Support

<AccordionGroup>
  <Accordion title="How can I get help?">
    - **Documentation**: Browse our [docs](/) for guides and API references
    - **Status Page**: Check [status.backboard.io](https://backboard.statusgator.app/) for service status
    - **Contact**: Reach out via the dashboard for support inquiries
  </Accordion>

  <Accordion title="What happens if Backboard is down?">
    You can monitor real-time service status at [backboard.statusgator.app](https://backboard.statusgator.app/). We recommend implementing appropriate error handling and retry logic in your applications.
  </Accordion>
</AccordionGroup>
