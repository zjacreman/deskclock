# Project Overview: Terminal Desk Clock

This is a Rust-based terminal application that displays a large, scaling digital clock and date. It is built using `ratatui` and `crossterm`.

## Architecture & Layout

### 1. File Structure
- `src/main.rs`: The main entry point and application logic. It handles the event loop, mode switching, UI layout, and keyboard input coordination. Contains only App-level integration tests (mode transitions, arrow key handling, notifier setup).
- `src/font.rs`: Defines the `LargeFont` system. It contains a map of characters (0-9, :, A, P, M, etc.) represented as 5x5 grids of block characters (`█`). Contains 33 glyph validation tests.
- `src/notification.rs`: Defines the `Notifier` trait and concrete implementations (`SystemNotifier` — on macOS: prefers `terminal-notifier` CLI, falls back to `osascript`; on Linux/Windows: uses `notify-rust`; test: `MockNotifier`) that send desktop notifications when the countdown timer completes. Contains 15 notification tests.
- `src/timer.rs`: Contains the `CountdownTimer` logic and state management. Contains 23 timer tests.
- `src/stopwatch.rs`: Contains the `Stopwatch` logic and state management. Contains 17 stopwatch tests.
- `src/ui.rs`: Contains the scaling rendering engine and layout definitions.
- `src/config.rs`: Contains configuration loading (`AppConfig`, `ColorConfig`, `DefaultMode`) and the `color_from_str` parser. Contains 7 config tests.
- `PLAN.md`: The original implementation plan used to develop the project.
- `Cargo.toml`: Project configuration and dependencies.

### 2. Core Components

#### Scaling Rendering Engine (`src/ui.rs` -> `render_large_text`)
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
    - `l`: Record lap time (Stopwatch mode only).
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

- **LargeFont** (33 tests): Validates all 37 glyphs (digits, letters, punctuation including `.`, all glyph dimensions (5x5), UTF-8 character counts, case-insensitive mapping, glyph consistency, and mixed case input.
- **Stopwatch** (17 tests): Tests start/pause/reset lifecycle, idempotency, time accumulation across pause cycles, lap recording, lap reset behavior, and ensuring laps don't affect running state.
- **CountdownTimer** (23 tests): Covers initialization, start/pause/reset flow, duration adjustments, boundary conditions (zero duration), remaining time calculation, and timer state persistence.
- **App State** (16 tests): Validates default state, mode transitions (Time/Countdown/Stopwatch), arrow key event gating, and AppMode derivation.
- **Notification** (15 tests): Validates `Notifier` trait implementations, `MockNotifier` behavior, `MockNotifier` notification message formatting, `terminal-notifier` availability detection, cross-platform notification path selection, and enabled/disabled notifier behavior.
- **Config** (7 tests): Validates default values, TOML parsing, custom values, and `color_from_str` parsing (named colors, hex, and rgb formats).

**Total: 111 unit tests**

## Development Notes for Future Agents

### Adding Glyphs
To add new characters (e.g., for a 24h clock or different symbols), add them to the `HashMap` in `src/font.rs` as `Vec<String>` with exactly 5 elements of 5 characters each. Each row must contain only valid block characters (`█`) and spaces.

### Testing Guidelines
- **New glyphs**: Add corresponding glyph content tests to verify the 5x5 structure and character composition.
- **New timer features**: Add tests that verify state transitions and edge cases (e.g., zero durations, negative adjustments).
- **Stopwatch changes**: Always test time accumulation across multiple pause/resume cycles.
- **New lap feature**: Test that `add_lap()` saves the lap, overwrites any previous lap, and `reset()` clears it — verify lap recording doesn't affect running state or elapsed time.
- **Font changes**: Verify both byte length (`len()`) and character count (`chars().count()`) when dealing with UTF-8 block characters.
- **Notification features**: Use `MockNotifier` to verify notifications without triggering actual OS calls.

### Testing Locations
Tests are organized by module in their respective files:
- `src/font.rs#tests`: 33 LargeFont glyph validation tests
- `src/stopwatch.rs#tests`: 17 Stopwatch timer tests
- `src/timer.rs#tests`: 23 CountdownTimer tests
- `src/notification.rs#tests`: 15 MockNotifier/SystemNotifier/terminal-notifier tests
- `src/config.rs#tests`: 7 config parsing tests
- `src/main.rs#tests`: 16 App integration tests (mode transitions, arrow keys, notifier injection)

### Layout Changes
The layout constraints are defined in `src/ui.rs` via the `create_main_layout` function. The three vertical sections use percentage-based constraints (70%, 20%, 10%).

### Scaling Improvements
Current scaling is integer-based. For smoother transitions, consider implementing sub-pixel-like approximations or different font tiers.

### Countdown Timer Implementation Notes
Implemented in `src/timer.rs` via the `CountdownTimer` struct. Note the distinction between 'paused' (Light Blue, blinking) and 'stopped' (White, static) states. The timer uses `initial_duration` to allow resetting to the last started value.

**Important**: Calling `start()` updates `initial_duration` to the current duration. If you need to preserve the original initial duration, call `start()` only once or manually save it before calling `start()`.

When the countdown timer finishes, it triggers two notifications:
1. A **red screen flash** (terminal-only visual alert lasting ~1.25 seconds)
2. A **native OS desktop notification** — on macOS, prefers `terminal-notifier` CLI (falls back to `osascript` if not installed); on Linux/Windows, uses `notify-rust` — with title "Countdown Timer Complete" and body "00:00 - Timer has finished"

To inject a `MockNotifier` for testing, use `app.with_notifier(Box::new(MockNotifier::new()))`.

### Stopwatch Implementation Notes
Implemented in `src/stopwatch.rs` via the `Stopwatch` struct. The stopwatch is rendered in Pink (`Color::Magenta`) and blinks when paused (with non-zero elapsed time). The timer state persists across mode switches.

The stopwatch displays subsecond (centisecond) precision. When the elapsed time is under 1 hour, the format is `MM:SS.cs` (minutes, seconds, centiseconds). When at or over 1 hour, it switches to `HH:MM:SS` (hours, minutes, seconds) since the terminal width cannot fit all six values plus the dot separator.

The `.` (dot) glyph was added to `src/font.rs` to serve as the subsecond separator, with a corresponding `test_dot_glyph_content` test.

#### Lap Feature
The stopwatch supports a single lap via pressing `l` in Stopwatch mode. The lap time is saved as `Duration` and displayed in the secondary display (middle section, scaled with the same large font) using the configurable `stopwatch_lap_color` (default `Blue`). Pressing `l` again overwrites the previous lap. Reset (`r`) clears the lap. No lap set shows "Stopwatch" text in the secondary display instead.

The lap key press is gated to Stopwatch mode — pressing `l` in any other mode does nothing.

## Configuration
Lap color can be customized in your config file:

```toml
stopwatch_lap_color = "Cyan"
```

## Dependencies
- `ratatui`: TUI framework.
- `crossterm`: Terminal backend and event handling.
- `chrono`: Time and date formatting.
- `notify-rust` (Linux/Windows only): Native OS desktop notifications for timer completion alerts.
- **macOS**: Uses `terminal-notifier` CLI if available (`terminal-notifier -title "..." -message "..."`), with `osascript display notification` as a fallback if `terminal-notifier` is not installed. (`which terminal-notifier` is used to detect availability.)