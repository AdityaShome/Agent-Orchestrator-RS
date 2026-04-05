# Agent Orchestrator Build Steps

## Phase 1: LLM Provider Integration
### Step 1.1: Create LLM client
- File: `src/llm/provider.rs`
- Content:
  - Uses `reqwest::Client` and `serde_json::json`
  - `call_llm(prompt: &str) -> String`
  - POST to provider generate endpoint (configured for Gemini)
  - Query param: `key` from `GEMINI_API_KEY` (loaded via `.env`)
  - JSON body:
    - `contents` -> `parts` -> `{ "text": prompt }`

### Step 1.2: Test it in `main.rs`
- Print response only. Nothing fancy.

## Phase 2: Structured Output (Critical)
### Step 2.1: Force JSON output
- Planner should return:
  - `{ "steps": ["step1", "step2"] }`

### Step 2.2: Create model
- File: `src/models/task.rs`
- Content:
  - `use serde::Deserialize;`
  - `#[derive(Deserialize)] pub struct Plan { pub steps: Vec<String>, }`

### Step 2.3: Parse response
- Must validate JSON. No shortcuts.

## Phase 3: Build Agents
### Step 3.1: Planner agent
- Function:
  - `pub fn planner_prompt(task: &str) -> String {`
  - `format!("Break task into steps. Return JSON:\nTask: {}\nFormat: {{\"steps\":[]}}", task)`
  - `}`

### Step 3.2: Worker agent
- Function:
  - `pub fn worker_prompt(step: &str) -> String {`
  - `format!("Execute this step:\n{}", step)`
  - `}`

### Step 3.3: Aggregator agent
- Function:
  - `pub fn aggregator_prompt(results: &Vec<String>) -> String {`
  - `format!("Combine results into final answer:\n{:?}", results)`
  - `}`

## Phase 4: Build Orchestrator (First Real System)
### Step 4.1: Core flow
- File: `src/orchestrator/mod.rs`
- Flow:
  - Call planner (`call_gemini(planner_prompt(task))`)
  - Steps placeholder: `let steps = vec!["dummy step"]; // replace with parsed JSON`
  - For each step: call worker, collect results
  - Call aggregator on results
  - `println!("{}", final_output);`

## Phase 5: Make It Real
### Step 5.1: Proper JSON parsing
- Extract only JSON
- Handle invalid outputs
- Add fallback

### Step 5.2: Logging
- Print:
  - plan
  - each step
  - results
- Note: Debugging agents = 50% of the work

## Phase 6: Parallel Execution
- Replace worker loop with parallel calls
- Use `futures::future::join_all`

## Phase 7: Add Router Agent
- Router decides which agent to call
- Example output:
  - `{ "agent": "research", "input": "..." }`

## Phase 8: Add Memory
- Create `memory/`
  - `store.rs`
- Store:
  - previous outputs
  - context

## Phase 9: Add Critic Loop (Game Changer)
- Flow:
  - Worker -> Critic -> Improve -> Final
- Note: Improves quality

## Phase 10: Add Tool System (MCP-lite)
### Step 10.1: Tool schema
- `enum Tool { Search, FileRead }`

### Step 10.2: Agent outputs
- `{ "tool": "search", "query": "multi agent systems" }`

### Step 10.3: Rust executes tool
- System interacts with real world

## Phase 11: Production Features
- Add:
  - retries
  - timeout handling
  - error recovery
  - rate limiting

## Phase 12: Advanced Systems
- Add:
  - hierarchical agents
  - voting system
  - cost tracking
  - dynamic planning
