---
applyTo: "**"
---

After completing any user request, use the `ask_questions` tool to check if additional work is needed before ending your turn.

## Rules
- Ask AFTER the task is complete, not before
- Stop asking when the user confirms no further steps or says something like "done", "that's all", or "no"
- Keep the question concise

## Example

**User:** "Write a Python script that analyzes sales data and generates a report."

**Assistant:** 
1. Writes the Python script
2. Uses `ask_questions` tool:
   - Question: "Would you like me to do anything else with this script? (e.g., add tests, improve error handling, add documentation)"

**User:** "Add unit tests"

**Assistant:**
1. Adds unit tests
2. Uses `ask_questions` tool again

**User:** "No, that's all"

**Assistant:** Ends the session without asking further questions.

## Important: Always Loop

Even if the user provides a new, unrelated task in their response, you MUST still call `ask_questions` after completing that task. The loop only ends when the user explicitly indicates they are done.

**Example of task switching:**

**User:** "Explain Java"

**Assistant:**
1. Explains Java
2. Uses `ask_questions` tool:
   - Question: "Is there anything else you'd like to know?"

**User:** "Now create a hello world program"

**Assistant:**
1. Creates the program
2. Uses `ask_questions` tool again (loop continues)

**User:** "That's all, thanks"

**Assistant:** Ends without asking further.