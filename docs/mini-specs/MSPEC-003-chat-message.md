# Mini-Spec 003: ChatMessage Component

**Task:** Create the ChatMessage component with 5 variants for displaying user, agent, system, tool call, and error messages in the chat thread.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. All 5 variants render with correct visual styling:
   | Variant | Visual | Left Accent |
   |---------|--------|-------------|
   | **User** | Glass panel, no glow | Gray left border `--border-glass` |
   | **Agent** | Glass panel with purple glow | Purple left border + streaming cursor |
   | **System** | Muted, compact | No border, smaller `--text-muted` text |
   | **Tool Call** | Expandable accordion embedded in chat (see ToolCallAccordion) | Yellow left border `--status-warning` |
   | **Error** | Red-tinted glass border | Red left border `--status-error` |

2. Agent message supports streaming mode with animated breathing cursor (`▎` with purple glow per design guide §6.3)

3. Message timestamp shown as secondary text (`--text-secondary`, 12px)

4. Agent avatar (status dot + initials/icon) shown left of agent messages

5. User avatar (initials) shown right of user messages

6. Copy-to-clipboard button on hover for code-containing messages

7. CSS Module scoped — uses `design-tokens.css` tokens

8. Chat area uses `role="log"` + `aria-live="polite"`

9. Streaming text uses `aria-busy="true"` while streaming, `aria-busy="false"` when complete

10. Supports `children` slots for nested CodeBlock and ToolCallAccordion components

11. Typescript props: `variant`, `content`, `timestamp`, `isStreaming`, `avatar`, `agentName`, `children`

## Out of Scope

- Inline markdown rendering (assumes markdown parsed upstream)
- Edit/delete message actions

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
