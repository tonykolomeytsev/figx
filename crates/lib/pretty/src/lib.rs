use crossterm::{
    cursor::{Hide, MoveToNextLine, MoveToPreviousLine, Show},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
    terminal::{Clear, ClearType},
};
use ordermap::OrderMap;
use std::{
    hash::Hash,
    io::{Write, stderr},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize},
    },
    thread::{self},
    time::{Duration, UNIX_EPOCH},
};

/// A renderer for visualizing long-running parallel processes in CLI applications.
///
/// Maintains an ordered collection of states and renders them to stderr at 60 FPS.
/// Intended to be alive during heavy operations, not needed for instant tasks.
///
/// Output is rendered only in interactive (`tty`) terminals. In non-interactive terminals,
/// logging is preferred (not yet implemented).
pub struct StateRenderer {
    states: Arc<Mutex<OrderMap<usize, State>>>,
    is_active: Arc<AtomicBool>,
    keys: Arc<AtomicUsize>,
}

/// Represents the state of a long-running operation.
///
/// Variants correspond to different operation phases with human-readable labels.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum State {
    /// Operation is blocked/waiting for resources or queue position
    Pending(String),
    /// Performing network I/O operations
    Fetching(String),
    Exporting(String),
    Downloading(String),
    /// Performing heavy local computations
    Transforming(String),
}

impl Default for StateRenderer {
    fn default() -> Self {
        let states: Arc<Mutex<OrderMap<usize, State>>> = Default::default();
        let is_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let keys: Arc<AtomicUsize> = Default::default();
        {
            let cloned_states = states.clone();
            let cloned_is_active = is_active.clone();
            thread::spawn(move || render_infinitely(cloned_states, cloned_is_active));
        }
        Self {
            states,
            is_active,
            keys,
        }
    }
}

impl Drop for StateRenderer {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering::*;
        self.is_active.store(false, Relaxed);
        thread::sleep(Duration::from_millis(16));
        let _ = execute!(stderr().lock(), Clear(ClearType::FromCursorDown), Show);
    }
}

impl StateRenderer {
    /// Creates a new handle to manage a single operation's state.
    ///
    /// Each long-running operation should get its own handle.
    /// The state will be automatically removed when handle is dropped.
    /// The order of lines is stable and reflects the order of handle creation.
    pub fn get_handle(&self) -> StateHandle {
        use std::sync::atomic::Ordering::*;
        StateHandle {
            key: self.keys.fetch_add(1, Relaxed),
            states: self.states.clone(),
        }
    }
}

/// Internal rendering loop running at ~60 FPS (16ms interval).
///
/// Uses stderr for output to avoid interfering with stdout logging.
/// Workarounds exist for crossterm quirks (see links in source).
pub fn render_infinitely(states: Arc<Mutex<OrderMap<usize, State>>>, is_active: Arc<AtomicBool>) {
    use std::sync::atomic::Ordering::*;
    while is_active.load(Relaxed) {
        render(&states).unwrap();
        thread::sleep(Duration::from_millis(16));
    }
}

/// Renders all current states to terminal with appropriate spinners and colors.
///
/// Note: Errors are intentionally ignored as rendering is non-critical.
/// TODO: Add TTY detection to skip rendering for non-interactive terminals.
fn render(states: &Arc<Mutex<OrderMap<usize, State>>>) -> std::io::Result<()> {
    let mut stdout = stderr().lock();
    let current_states = states.lock().unwrap();
    let current_time = (std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 100
        % 8) as usize;

    queue!(stdout, Clear(ClearType::FromCursorDown))?;

    // region: Workarounds
    // That's why:
    // - https://github.com/crossterm-rs/crossterm/issues/550
    // - https://github.com/crossterm-rs/crossterm/issues/673
    let lines_count = current_states.len();
    for _ in 0..lines_count {
        queue!(stdout, Print("\n"))?;
    }
    if lines_count > 0 {
        queue!(stdout, Hide, MoveToPreviousLine(lines_count as u16))?;
    } else {
        queue!(stdout, Show)?;
    }
    // endregion: Workarounds

    for (_, state) in current_states.iter() {
        match state {
            State::Pending(label) => queue!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(light_spinner(current_time)),
                Print(" Pending ".bold()),
                ResetColor,
                Print(&label),
                MoveToNextLine(1),
            )?,
            State::Fetching(label) => queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(medium_spinner(current_time)),
                Print(" Fetching ".bold()),
                ResetColor,
                Print(&label),
                MoveToNextLine(1),
            )?,
            State::Exporting(label) => queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(medium_spinner(current_time)),
                Print(" Exporting ".bold()),
                ResetColor,
                Print(&label),
                MoveToNextLine(1),
            )?,
            State::Downloading(label) => queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(medium_spinner(current_time)),
                Print(" Downloading ".bold()),
                ResetColor,
                Print(&label),
                MoveToNextLine(1),
            )?,
            State::Transforming(label) => queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(heavy_spinner(current_time)),
                Print(" Transforming ".bold()),
                ResetColor,
                Print(&label),
                MoveToNextLine(1),
            )?,
        }
    }

    if lines_count > 0 {
        queue!(stdout, MoveToPreviousLine(lines_count as u16))?;
    }

    stdout.flush()
}

/// Handle for managing an individual operation's rendered state.
///
/// Dropping the handle removes its associated line from the renderer.
pub struct StateHandle {
    key: usize,
    states: Arc<Mutex<OrderMap<usize, State>>>,
}

impl StateHandle {
    /// Sets the current state for this process.
    ///
    /// This updates the displayed text and spinner. Overwrites any previous state.
    pub fn set_state(&self, state: State) {
        let mut states = self.states.lock().unwrap();
        states.insert(self.key, state);
    }

    /// Removes this handle from the active set.
    ///
    /// This will cause its line to disappear in the next frame.
    pub fn remove_state(&self) {
        let mut states = self.states.lock().unwrap();
        states.remove(&self.key);
    }
}

/// Spinner characters for Pending state (light animation)
fn light_spinner(i: usize) -> char {
    let arr = ['⠈', '⠐', '⠠', '⢀', '⡀', '⠄', '⠂', '⠁'];
    arr[i % 8]
}

/// Spinner characters for Fetching state (medium animation)
fn medium_spinner(i: usize) -> char {
    let arr = ['⣶', '⣧', '⣏', '⡟', '⠿', '⢻', '⣹', '⣼'];
    arr[i % 8]
}

/// Spinner characters for Transforming state (heavy animation)
fn heavy_spinner(i: usize) -> char {
    let arr = ['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'];
    arr[i % 8]
}

impl Drop for StateHandle {
    fn drop(&mut self) {
        if let Ok(states) = self.states.lock().as_deref_mut() {
            states.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    #[ignore = "for manual local testing only"]
    pub fn test_animation() {
        let pretty = StateRenderer::default();

        let h1 = pretty.get_handle();
        let h2 = pretty.get_handle();
        let h3 = pretty.get_handle();
        let h4 = pretty.get_handle();

        h1.set_state(State::Fetching("@ui-kit".to_string()));
        h2.set_state(State::Pending("//feature/app/settings:Gear".to_string()));
        h3.set_state(State::Pending("//feature/app/settings:Home".to_string()));
        h4.set_state(State::Pending("//feature/app/settings:Toggle".to_string()));
        thread::sleep(Duration::from_secs(2));
        h1.set_state(State::Fetching("@ui-kit".to_string()));
        thread::sleep(Duration::from_secs(2));
        h1.remove_state();
        h1.set_state(State::Fetching("//feature/app/settings:Save".to_string()));
        h2.set_state(State::Fetching("//feature/app/settings:Gear".to_string()));
        thread::sleep(Duration::from_secs(1));
        h3.set_state(State::Fetching("//feature/app/settings:Home".to_string()));
        thread::sleep(Duration::from_secs(1));
        h4.set_state(State::Fetching("//feature/app/settings:Toggle".to_string()));
        thread::sleep(Duration::from_secs(2));
        h2.set_state(State::Transforming(
            "//feature/app/settings:Gear".to_string(),
        ));
        thread::sleep(Duration::from_secs(1));
        h3.set_state(State::Transforming(
            "//feature/app/settings:Home".to_string(),
        ));
        thread::sleep(Duration::from_secs(1));
        h4.set_state(State::Transforming(
            "//feature/app/settings:Toggle".to_string(),
        ));
        h1.set_state(State::Transforming(
            "//feature/app/settings:Gear".to_string(),
        ));
        thread::sleep(Duration::from_secs(1));
        h2.remove_state();
        thread::sleep(Duration::from_secs(1));
        h3.remove_state();
        thread::sleep(Duration::from_secs(1));
        h4.remove_state();
        thread::sleep(Duration::from_secs(1));
        h1.remove_state();
        drop(pretty);
    }
}
