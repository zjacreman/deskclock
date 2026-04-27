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

#### UI Layout
The screen is divided vertically using `ratatui::layout::Layout`:
- **Top Section (80%)**: Reserved for the scaled Time display (`HH:MM:SS AM/PM`).
- **Bottom Section (20%)**: Reserved for the Date display (`Weekday, Month Day, Year`).

### 3. Event Loop
- **Tick Rate**: Sets a refresh interval (approx 200ms) to keep the clock seconds accurate.
- **Input**: Listens for the `q` key to trigger a graceful shutdown.
- **Responsiveness**: Layout and scaling are re-calculated on every frame, making the app naturally responsive to terminal resize events.

## Development Notes for Future Agents
- **Adding Glyphs**: To add new characters (e.g., for a 24h clock or different symbols), add them to the `HashMap` in `src/font.rs` as `Vec<String>` with exactly 5 elements of 5 characters each.
- **Changing Layout**: The layout constraints are located in the `terminal.draw` closure in `src/main.rs`.
- **Scaling Improvements**: Current scaling is integer-based. For smoother transitions, consider implementing sub-pixel-like approximations or different font tiers.

## Dependencies
- `ratatui`: TUI framework.
- `crossterm`: Terminal backend and event handling.
- `chrono`: Time and date formatting.
