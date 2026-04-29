# Project Overview: Terminal Desk Clock

This is a Rust-based terminal application that displays a large, scaling digital clock and date. It is built using `ratatui` and `crossterm`.

## Architecture & Layout

### 1. File Structure
- `src/main.rs`: The main entry point and application logic. It handles the event loop, TUI layout, and the scaling rendering engine.
- `src/font.rs`: Defines the `LargeFont` system. It contains a map of characters (0-9, :, A, P, M, etc.) represented as 5x5 grids of block characters (`█`).
- `PLAN.md`: The original implementation plan used to develop the project.
- `Cargo.toml`: Project configuration and dependencies.

### 2. Core Components

#### Scaling Rendering Engine (`src/main.rs` -> `render_large_text`)
The most critical part of the application is how it handles "large" numbers in a grid-based terminal:
- **Base Glyphs**: Every character is defined as a 5x5 grid in `src/font.rs`.
- **Scaling Logic**: The engine calculates a `scale` factor by dividing the available terminal area by the total base width/height of the string to be rendered.
- **Drawing**:
    - It repeats each row of the base glyph `scale` times vertically.
    - It repeats each character in the row string `scale` times horizontally.
    - A single-cell spacer is added between glyphs to prevent blending.
- **Fallback**: If the terminal is too small to display even the base 1x scale, it falls back to a standard `ratatui::widgets::Paragraph`.
### 3. Core Components

#### UI Layout
The screen is divided vertically using `ratatui::layout::Layout`:
- **Top Section (70%)**: Reserved for the scaled Time display (`HH:MM:SS AM/PM`), Countdown Timer, or Stopwatch.
- **Middle Section (20%)**: Reserved for the Date display (`Weekday, Month Day, Year`) or the Countdown Timer's end time.
- **Bottom Section (10%)**: Reserved for a dynamic command menu that displays available keys based on the current mode and state.

### 3. Event Loop
- **Tick Rate**: Sets a refresh interval (approx 200ms) to keep the clock seconds accurate and handle UI animations (blinking, flashing).
- **Input**: Available commands are displayed in the bottom menu and vary by mode:
    - `q`: Graceful shutdown.
    - `t`: Switch to Time mode.
    - `c`: Switch to Countdown mode.
    - `s`: Switch to Stopwatch mode.
    - `h`: Toggle 12h/24h clock format (Time mode only).
    - `Space`: Start/Pause timer.
    - `r`: Reset timer to session start value.
    - `Up`/`Down`: Adjust timer minutes (Countdown mode only).
    - `Left`/`Right`: Adjust timer seconds (Countdown mode only).
- **Responsiveness**: Layout and scaling are re-calculated on every frame, making the app naturally responsive to terminal resize events.

## Testing

This project includes a comprehensive unit test suite covering the core components: LargeFont, Stopwatch, CountdownTimer, and App state management.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test category
cargo test font
cargo test stopwatch
cargo test countdown
```

### Test Coverage

- **LargeFont** (29 tests): Validates all 36 glyphs (digits, letters, punctuation), character dimensions (5x5), UTF-8 character counts, case-insensitive mapping, and unknown character handling.
- **Stopwatch** (10 tests): Tests start/pause/reset lifecycle, idempotency, time accumulation across pause cycles, and running/paused accuracy.
- **CountdownTimer** (22 tests): Covers initialization, start/pause/reset flow, duration adjustments, boundary conditions (zero duration), remaining time calculation, and timer state persistence.
- **App State** (19 tests): Validates default state, mode transitions (Time/Countdown/Stopwatch), font integration, and arrow key event gating.

**Total: 80 unit tests**

## Development Notes for Future Agents

### Adding Glyphs
To add new characters (e.g., for a 24h clock or different symbols), add them to the `HashMap` in `src/font.rs` as `Vec<String>` with exactly 5 elements of 5 characters each. Each row must contain only valid block characters (`█`) and spaces.

### Testing Guidelines
- **New glyphs**: Add corresponding glyph content tests to verify the 5x5 structure and character composition.
- **New timer features**: Add tests that verify state transitions and edge cases (e.g., zero durations, negative adjustments).
- **Stopwatch changes**: Always test time accumulation across multiple pause/resume cycles.
- **Font changes**: Verify both byte length (`len()`) and character count (`chars().count()`) when dealing with UTF-8 block characters.

### Layout Changes
The layout constraints are located in the `terminal.draw` closure in `src/main.rs`. The three vertical sections use percentage-based constraints (70%, 20%, 10%).

### Scaling Improvements
Current scaling is integer-based. For smoother transitions, consider implementing sub-pixel-like approximations or different font tiers.

### Countdown Timer Implementation Notes
Implemented in `src/main.rs` via `CountdownTimer` struct. Note the distinction between 'paused' (Light Blue, blinking) and 'stopped' (White, static) states. The timer uses `initial_duration` to allow resetting to the last started value.

**Important**: Calling `start()` updates `initial_duration` to the current duration. If you need to preserve the original initial duration, call `start()` only once or manually save it before calling `start()`.

### Stopwatch Implementation Notes
Implemented in `src/main.rs` via `Stopwatch` struct. The stopwatch is rendered in Pink (`Color::Magenta`) and blinks when paused (with non-zero elapsed time). The timer state persists across mode switches.

## Dependencies
- `ratatui`: TUI framework.
- `crossterm`: Terminal backend and event handling.
- `chrono`: Time and date formatting.