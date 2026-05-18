# Ulti

Ulti is a terminal emulator for agent-focused terminal workspaces. It balances a minimal everyday terminal surface with built-in workspace concepts that reduce the need for external multiplexers.

## Language

**Terminal Emulator**:
A graphical application that runs shells and terminal programs directly inside its own window.
_Avoid_: Terminal wrapper, tmux replacement

**Bare Bones Terminal**:
A usable first terminal surface with an interactive local shell and basic ANSI terminal behavior.
_Avoid_: Text-only shell, full TUI-compatible terminal

**Workspace**:
A named area for grouping related terminal work.
_Avoid_: Project, session group

**Tab**:
A switchable view inside a workspace that contains one pane layout.
_Avoid_: Window, workspace

**Pane**:
A rectangular terminal area inside a tab.
_Avoid_: Split, terminal

**Terminal Session**:
A running shell or terminal program attached to exactly one pane.
_Avoid_: Pane, process, tab

**Attention State**:
The visible status of a pane that helps a user monitor long-running terminal work.
_Avoid_: AI status, notification, job state

## Relationships

- A **Terminal Emulator** starts with a **Bare Bones Terminal** before adding workspace surfaces.
- A **Workspace** contains one or more **Tabs**.
- A **Tab** contains one or more **Panes**.
- A **Pane** owns exactly one **Terminal Session**.
- A **Terminal Session** exists only while the application process is running.
- A **Pane** exposes an **Attention State** for agent-focused monitoring.

## Example dialogue

> **Dev:** "Are we building a wrapper around tmux first?"
> **Domain expert:** "No, Ulti is a **Terminal Emulator** first; workspace features come later inside that emulator."

> **Dev:** "Does **Bare Bones Terminal** mean we support vim immediately?"
> **Domain expert:** "No, it means interactive shell I/O and basic ANSI behavior first; full-screen TUIs can come later."

> **Dev:** "When an agent runs in the bottom-right split, is that a workspace?"
> **Domain expert:** "No, that is a **Terminal Session** inside a **Pane**; the **Workspace** groups the broader task."

> **Dev:** "Does agent-focused mean Ulti talks to AI APIs?"
> **Domain expert:** "Not first; it means panes expose an **Attention State** so long-running agent work is easier to monitor."

## Flagged ambiguities

- "terminal" can mean an emulator, shell session, or workspace tool; resolved: this project starts as a **Terminal Emulator**.
