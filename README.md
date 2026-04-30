# 🕒 Terminal Desk Clock

A high-visibility, scaling digital clock and countdown timer for your terminal. Designed to turn your terminal into a functional desk clock, this application uses `ratatui` and `crossterm` to render large, responsive glyphs that adapt to your window size.

## ✨ Features

- **Adaptive Scaling**: The clock and date automatically scale their size to fill the available terminal area.
- **Dual Modes**:
  - **Clock Mode**: Displays the current time (`HH:MM:SS AM/PM` or `HH:MM:SS`) and date.
  - **Countdown Timer**: A fully featured timer for productivity (e.g., Pomodoro) with custom durations.
  - **Stopwatch**: A precise timing tool with start/pause/reset functionality and centisecond subsecond precision (under 1 hour).
- **Visual Feedback**:
  - **Blinking**: The timer blinks when paused to notify you that it is not actively counting down.
  - **Color Shifting**: 
    - The Countdown Timer shifts to **Light Blue** when active.
    - The Stopwatch shifts to **Pink** when running.
  - **Termination Alert**: The terminal window flashes red when the countdown reaches zero.
  - **Desktop Notification**: A native OS desktop notification is sent with title "Countdown Timer Complete" when the timer finishes.
- **Responsive Design**: Layout and scaling are re-calculated on every frame, making it perfectly responsive to terminal resize events.

## 🚀 Usage

### ⌨️ General Controls
| Key | Action |
| :--- | :--- |
| `q` | Quit application |
| `t` | Switch to **Clock Mode** |
| `c` | Switch to **Countdown Mode** |
| `s` | Switch to **Stopwatch Mode** |
| `h` | Toggle 12h/24h clock format |

### ⏲️ Countdown Timer Controls
When in Countdown Mode, you have additional controls to manage your session:

| Key | Action |
| :--- | :--- |
| `Space` | Start / Pause the timer |
| `r` | Reset timer to the duration it had when last started |
| `↑` / `↓` | Increase / Decrease minutes |
| `←` / `→` | Increase / Decrease seconds |

#### Timer States:
- **Running**: Display is Light Blue.
- **Paused**: Display is Light Blue and blinks.
- **Stopped/Reset**: Display is White.

### ⏱️ Stopwatch Controls
When in Stopwatch Mode, you have the following controls:

| Key | Action |
| :--- | :--- |
| `Space` | Start / Pause the stopwatch |
| `r` | Reset stopwatch to zero and clear lap |
| `l` | Record lap time (displayed in the secondary display) |

### Stopwatch Display Formats
- **Under 1 hour**: `MM:SS.cs` (minutes, seconds, centiseconds)
- **1 hour or more**: `HH:MM:SS` (hours, minutes, seconds) — the dot separator cannot fit all six values in the terminal width

### Lap Times
The current lap time is shown in the secondary (middle) display in the configured lap color (default blue). Pressing `l` overwrites the previous lap. When no lap is set, the secondary display shows "Stopwatch".

## 🛠️ Installation & Building

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) and `cargo` installed on your system.

### Build and Run
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd deskclock
   ```

2. Run the application directly:
   ```bash
   cargo run
   ```

3. To build a release version for maximum performance:
   ```bash
   cargo build --release
   ./target/release/deskclock
   ```

## 🏗️ Architecture
- **Scaling Engine**: Uses a custom 5x5 grid-based font system. It calculates a scale factor based on the terminal's `Rect` dimensions to repeat base glyph characters horizontally and vertically.
- **State Management**: Maintains the timer state in the background, allowing you to switch between the clock and timer modes without interrupting a running countdown.

## 🧪 Testing

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
- **Notification** (11 tests): Validates `Notifier` trait implementations, `MockNotifier` behavior, and notification message formatting.
- **Config** (7 tests): Validates default values, TOML parsing, custom values, and `color_from_str` parsing (named colors, hex, and rgb formats).

**Total: 107 unit tests**