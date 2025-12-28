# Plan Generation Prompt

You are a senior software architect. Analyze the following contract and codebase context, then create a detailed implementation plan.

## Contract
{{contract}}

## Codebase Context
{{context}}

## Instructions
1. Break down the work into clear, actionable tasks.
2. Order tasks by dependency (do A before B if B depends on A).
3. Estimate complexity for each task (1-5 scale).
4. Identify potential risks or blockers.

## Output Format
Provide a numbered list of tasks with:
- Task name
- Description
- Estimated complexity
- Dependencies (if any)
