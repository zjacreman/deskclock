# Implementation Plan: Stopwatch Feature

This document outlines the detailed technical steps required to implement the Stopwatch functionality in the Terminal Desk Clock application.

## 1. Objectives
- Add a new `Stopwatch` mode accessible via the 's' key.
- Use the existing large-text rendering engine.
- Provide controls for Start/Pause (Space) and Reset ('r').
- Ensure stopwatch state persists across mode switches.
- Ensure stopwatch operations do not interfere with the countdown timer.
- Visuals: Render in Pink (`Color::Magenta`), blink when paused.

## 2. Technical Specifications

### 2.1 Data Model
Create a `Stopwatch` struct in `src/main.rs` to encapsulate the stopwatch state.

```rust
struct Stopwatch {
    elapsed_time: Duration,
    last_start_time: Option<Instant>,
    is_running: bool,
}
```

**Required Methods for `Stopwatch`:**
- `new()`: Initialize with `elapsed_time: Duration::ZERO`, `last_start_time: None`, and `is_running: false`.
- `start()`: 
    - Set `is_running = true`.
    - Set `last_start_time = Some(Instant::now())`.
- `pause()`: 
    - If `is_running` is true:
        - Calculate duration since `last_start_time`.
        - Add that duration to `elapsed_time`.
        - Set `is_running = false`.
        - Set `last_start_time = None`.
- `reset()`: 
    - Set `elapsed_time = Duration::ZERO`.
    - Set `is_running = false`.
    - Set `last_start_time = None`.
- `current_elapsed()`: 
    - Return `elapsed_time`.
    - If `is_running` is true, add the duration since `last_start_time` to the returned value.

### 2.2 Application State Integration
- **`AppMode` Enum**: Add a `Stopwatch` variant to `AppMode`.
- **`App` Struct**: Add a field `stopwatch: Stopwatch`.
- **`App::new()`**: Initialize the `stopwatch` field.

## 3. Logic and Event Handling

### 3.1 Mode Switching
Update the event loop to handle the `'s'` key:
- `KeyCode::Char('s')` $\rightarrow$ `self.mode = AppMode::Stopwatch`.

### 3.2 Context-Aware Controls
Modify the input handling logic to ensure controls are scoped to the current mode.

- **Space Bar (`KeyCode::Char(' ')`)**:
    - If `self.mode == AppMode::Stopwatch`:
        - If `stopwatch.is_running` $\rightarrow$ `stopwatch.pause()`.
        - Else $\rightarrow$ `stopwatch.start()`.
    - If `self.mode == AppMode::Countdown`: (Existing logic).

- **Reset (`KeyCode::Char('r')`)**:
    - If `self.mode == AppMode::Stopwatch`:
        - `stopwatch.reset()`.
    - If `self.mode == AppMode::Countdown`: (Existing logic).

## 4. Rendering Pipeline

### 4.1 Time Formatting
The stopwatch should be rendered in the top section (`chunks[0]`).
- Convert `stopwatch.current_elapsed()` into a string format: `HH:MM:SS`.
- If hours are 0, `MM:SS` is acceptable, or keep `HH:MM:SS` for consistency.

### 4.2 Visual Styling
- **Color**: Use `Color::Magenta` (Pink).
- **Blinking Logic**:
    - If `self.mode == AppMode::Stopwatch` and `!stopwatch.is_running` and `stopwatch.elapsed_time > Duration::ZERO`:
        - Apply visibility toggle based on `(Local::now().timestamp_millis() / 500) % 2 == 0`.
    - Else: Visible.

### 4.3 UI Menu
Update the bottom menu string based on the mode:
- If `AppMode::Stopwatch`: 
    - If running: `"q: Quit | t: Time | c: Countdown | Space: Pause | r: Reset"`
    - If paused/stopped: `"q: Quit | t: Time | c: Countdown | Space: Start | r: Reset"`

## 5. Documentation Update
Update `AGENTS.md` to reflect the new functionality:
- **UI Layout**: Add "or Stopwatch" to the Top Section description.
- **Event Loop**: Add `s: Switch to Stopwatch mode` to the list of input commands.
- **Development Notes**: Add a section describing the `Stopwatch` struct and its behavior (Pink, blinking when paused).

## 6. Verification Checklist
- [ ] Pressing 's' switches to stopwatch mode.
- [ ] Space starts and pauses the stopwatch.
- [ ] 'r' resets stopwatch to 0:00.
- [ ] Switching from Stopwatch $\rightarrow$ Time $\rightarrow$ Stopwatch maintains the timer progress.
- [ ] Resetting the stopwatch does NOT reset a running countdown timer.
- [ ] Stopwatch is rendered in Pink.
- [ ] Stopwatch blinks when paused (and elapsed > 0).
- [ ] The layout remains responsive to terminal resize.