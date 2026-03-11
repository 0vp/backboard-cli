we will be using /Users/q/Desktop/projects/backboard-cli/docs as the LLM endpoint, please understand it well.

the backboard api key is already in the .env as BACKBOARD_API_KEY

we are building a coding agent
- root project that we open the project in is the project we want to code in.
- allow tools ls,glob,ripgrep (as grep), execute, tool calls, webfetch, websearch, message, todo_create, todo_complete, etc.
- how the agent flow works. user enters prompt -> agent runs a bunch tool calls -> sends message to user to update on progress -> tool calls get a response based on the tool response, which has a id. the backboard docs has a example of how to do the tools etc, ONLY return the tool response, for non response tools, such as the message or todo create, then those will just return a 'keep alive' to keep the agent going. a example of a agent using backboard can be found here: /Users/q/Desktop/projects/backboard-swarm

for websearch, we want to use jina.ai, here are the example calls of how this can be done.

web fetch
curl "https://r.jina.ai/https://www.example.com" \
  -H "Authorization: Bearer JINA_API_KEY"

web search
curl "https://s.jina.ai/?q=Jina+AI" \
  -H "Authorization: Bearer JINA_API_KEY" \
  -H "X-Respond-With: no-content"

use the .env in the root directory, the env key is already there: JINA_API_KEY

for the todo:
we want to have a clear list, and once a todo is done, then we should have it crossed out. this will help keep the agent on track.

the system prompt and tools should be in seperate folders and stuff, only 1 root file.

tech stack
- RUST
- think like a senior level code reviewer & architect
- files should be max 250-300 lines, seperated by concerns (configs, constants, lists, etc all seperate)
- follow RUST style guide

ideal projects to mimick, if possible, get the system prompts for factory and stuff.
- claude code
- factory ai

the ideal system prompt will automatically run ls, and automatically search for the tools and their versions, so it run ls or tree with a certain depth, so the agent has a good starting point.

then it can run stuff based on the available tools, such as python3 --version, python --version and stuff so there is a better idea of the development env.

for the ui, it should be a streaming tui (Readline / REPL-style TUI) refer to image 1 for a example of how it should look like. however, the color should be a light laventar purple or something.