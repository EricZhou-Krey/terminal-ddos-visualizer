**A terminal-based visualizer for global DDoS attack traffic, powered by the Cloudflare API and built with Rust using the Ratatui TUI framework.**

## ✨ Features
* **Request Queue:** Displays a feed of recent "pings" or attack requests, including origin and destination regions (e.g., `Ping from: Vietnam -> Vietnam`).
* **Regional View:** Allows navigation between different regional map views (e.g., World, Europe, Asia) for focused inspection.
* **TUI Interface:** Uses the **Ratatui** library for a responsive, interactive Terminal User Interface.

## 💻 Screenshots

| World View 1 | World View 2 | Asia View |
| :---: | :---: | :---: |
| ![Global View1](img/Screenshot1.png) | ![Global View2](img/Screenshot2.png) | ![Zoomed in view of Asia](img/Screenshot3.png) |
| Shows global traffic overview. | Zooms in on Asia region. |


## ⚙️ Configuration
| `CLOUDFLARE_API_KEY` | (Required) | Your Cloudflare API key for data fetching. 

## ⌨️ Controls

The TUI is designed for simple keyboard navigation.

| Key | Action |
| :--- | :--- |
| **`Any Button other than left or right`** | Quit the application. |
| **`Left Arrow`** | Navigate to the previous regional view (e.g., from Asia to World). |
| **`Right Arrow`** | Navigate to the next regional view (e.g., from World to Europe). |
