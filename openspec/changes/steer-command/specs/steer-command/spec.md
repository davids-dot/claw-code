## ADDED Requirements

### Requirement: Steer Queue
The system SHALL maintain a thread-safe queue (`SteerQueue`) that holds pending steer texts. The queue MUST support concurrent push from the input thread and pop from the turn loop.

#### Scenario: Push and pop steer text
- **WHEN** a user enters `/steer focus on tests` during AI output
- **THEN** the text `"focus on tests"` is pushed to `SteerQueue`
- **AND** the next iteration of `run_turn` loop pops and injects it

#### Scenario: Multiple steer texts are queued in order
- **WHEN** a user enters `/steer A` then `/steer B` before the next API call
- **THEN** both texts are injected in order: first `"A"`, then `"B"`

### Requirement: Steer Slash Command
The system SHALL recognize `/steer <text>` as a valid slash command in the REPL. The command MUST be available both during AI output (via non-blocking stdin polling) and during idle prompt (via rustyline).

#### Scenario: Steer during AI output
- **WHEN** AI is streaming output and the user types `/steer use simpler variable names`
- **THEN** the text `"use simpler variable names"` is enqueued to `SteerQueue`
- **AND** the AI continues its current response without interruption

#### Scenario: Steer during idle prompt
- **WHEN** the user is at the REPL prompt and enters `/steer check error handling next`
- **THEN** the text `"check error handling next"` is enqueued to `SteerQueue`
- **AND** the next `run_turn` call injects it before the first API request

### Requirement: Steer Message Injection
The system SHALL inject queued steer texts as user messages into the session between conversation loop iterations. Each steer text MUST be prefixed with `[steer]` to distinguish it from regular user messages.

#### Scenario: Steer injected before next API call
- **WHEN** `run_turn` loop starts a new iteration and `SteerQueue` contains `"focus on error paths"`
- **THEN** a user message `"[steer] focus on error paths"` is appended to session messages
- **AND** the next API request includes this message in its context

#### Scenario: No steer text queued
- **WHEN** `run_turn` loop starts a new iteration and `SteerQueue` is empty
- **THEN** no additional user message is appended
- **AND** the iteration proceeds normally

### Requirement: Steer Help and Completion
The system SHALL include `/steer` in the slash command completion candidates and help output. The help text MUST describe the command as: "Inject guidance during AI output without interrupting the conversation".

#### Scenario: Steer appears in completion
- **WHEN** the user types `/st` and requests completion
- **THEN** `/steer` appears in the completion candidates

#### Scenario: Steer appears in help
- **WHEN** the user runs `/help`
- **THEN** the help output includes `/steer <text>` with its description

### Requirement: Steer Input Hint
The system SHALL display a hint during AI output indicating that `/steer` is available. The hint MUST be shown in the spinner area and MUST NOT interfere with the AI output rendering.

#### Scenario: Hint displayed during AI thinking
- **WHEN** AI is in the "thinking" state (spinner active)
- **THEN** the spinner area shows a hint like `Type /steer <text> to guide...` in dim text
