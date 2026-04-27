# Implementation Plan: Rust Terminal Desk Clock

## 1. Project Initialization & Dependencies
- **Project Setup**: Initialize a new Rust binary project.
- **Dependencies**:
    - `ratatui`: For the TUI framework and layout.
    - `crossterm`: For terminal backend, raw mode, and event handling (resize, quit).
    - `chrono`: For retrieving and formatting local system time.
    - `tokio` or `std::time`: For the event loop ticker.

## 2. The "Large Number" Rendering Engine
Since terminals are grid-based, "large numbers" must be constructed from multiple cells.
- **Font Map**: Define a constant map (e.g., a 2D array or a series of strings) that represents each digit (0-9) and symbols (`:`, ` `) as a grid of blocks (e.g., 5x3 or 8x5 cells).
- **Scaling Logic**: 
    - Implement a scaling algorithm that calculates the available terminal area.
    - Determine the maximum integer multiplier for the base font map that fits within the current terminal width and height while maintaining the aspect ratio.
    - Select the appropriate "font size" tier based on the current dimensions.
- **Buffer Drawing**: Create a function that translates the font map into `ratatui::widgets::Paragraph` or direct buffer modifications using block characters (e.g., `█`).

## 3. Layout & Positioning
- **Centering**: 
    - Use `ratatui::layout::Layout` and `Constraint::Percentage(50)` or calculate offsets manually to ensure the clock is centered perfectly.
    - Calculate a `Rect` for the "Time" area and a separate `Rect` for the "Date" area below it.
- **Dynamic Adjustment**: The layout logic will be executed inside the `draw` loop, meaning every frame (or every resize event) will re-calculate the available space and adjust the scale of the numbers automatically.

## 4. Application State & Event Loop
- **State Management**:
    - A simple struct to hold the current time and terminal dimensions.
- **The Main Loop**:
    - **Tick Rate**: Set a refresh interval of 1 second (or faster for smooth updates) to update the seconds display.
    - **Event Handling**:
        - `Event::Key`: Handle `q` or `Ctrl+C` to exit gracefully.
        - `Event::Resize`: Trigger a redraw to recalculate scaling and centering.
- **Graceful Exit**: Implement a cleanup routine to disable raw mode and show the cursor again upon exit.

## 5. Display Components
- **Primary Display (Time)**: 
    - Format: `HH:MM:SS AM/PM`.
    - Rendered using the Large Number engine.
- **Secondary Display (Date)**:
    - Format: `Weekday, Month Day, Year` (e.g., "Monday, April 27, 2026").
    - Rendered using standard Ratatui `Paragraph` centered below the time.

## 6. Execution Roadmap
1. **Skeleton**: Set up `crossterm` terminal raw mode and the basic `ratatui` loop.
2. **Time Logic**: Integrate `chrono` to print the time in a standard format.
3. **Font Engine**: Build the block-character map and the logic to render a single large digit.
4. **scaling & Layout**: Implement the logic that adjusts font size based on `frame.size()`.
5. **Final Polish**: Add the date display and refine centering/padding.
