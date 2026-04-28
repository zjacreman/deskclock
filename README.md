# 🕒 Terminal Desk Clock

A high-visibility, scaling digital clock and countdown timer for your terminal. Designed to turn your terminal into a functional desk clock, this application uses `ratatui` and `crossterm` to render large, responsive glyphs that adapt to your window size.

## ✨ Features

- **Adaptive Scaling**: The clock and date automatically scale their size to fill the available terminal area.
- **Dual Modes**:
  - **Clock Mode**: Displays the current time (`HH:MM:SS AM/PM`) and date.
  - **Countdown Timer**: A fully featured timer for productivity (e.g., Pomodoro) with custom durations.
- **Visual Feedback**:
  - **Blinking**: The timer blinks when paused to notify you that it is not actively counting down.
  - **Color Shifting**: The display shifts to Light Blue when the timer is active.
  - **Termination Alert**: The terminal window flashes red when the countdown reaches zero.
- **Responsive Design**: Layout and scaling are re-calculated on every frame, making it perfectly responsive to terminal resize events.

## 🚀 Usage

### ⌨️ General Controls
| Key | Action |
| :--- | :--- |
| `q` | Quit application |
| `t` | Switch to **Clock Mode** |
| `c` | Switch to **Countdown Mode** |

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