use crate::progress::ProgressBar;
use crossbeam_channel::{Receiver, Sender, unbounded};
use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use slab::Slab;
use std::{
    collections::HashSet,
    io::{stderr, IsTerminal, Write},
    sync::{
        atomic::{AtomicUsize, Ordering}, Arc, LazyLock, Mutex, OnceLock
    },
    thread::{self},
    time::Duration,
};
use terminal_size::Width;

mod logger;
pub use logger::*;
mod progress;

static INSTANCE: LazyLock<Dashboard> = LazyLock::new(|| Dashboard::new());

pub struct Dashboard {
    start_trigger: Sender<()>,
    is_interactive: bool,
    max_targets: Arc<AtomicUsize>,
    current_targets: Arc<AtomicUsize>,
    requested_remotes: Arc<AtomicUsize>,
    loaded_packages: Arc<AtomicUsize>,
    in_progress_targets: Arc<Mutex<Slab<String>>>,
    process_name: OnceLock<String>,
    progress_bar: Arc<Mutex<ProgressBar>>,
}

impl Dashboard {
    fn new() -> Self {
        let (start_trigger, start_receiver) = unbounded();
        thread::spawn(move || lifecycle_loop(start_receiver));
        Self {
            start_trigger,
            is_interactive: stderr().is_terminal() && !is_ci::cached(),
            max_targets: Default::default(),
            current_targets: Default::default(),
            requested_remotes: Default::default(),
            loaded_packages: Default::default(),
            in_progress_targets: Default::default(),
            process_name: OnceLock::new(),
            progress_bar: Default::default(),
        }
    }
}

fn lifecycle_loop(start_receiver: Receiver<()>) {
    if let Err(_) = start_receiver.recv() {
        return;
    }
    while let Err(_) = start_receiver.try_recv() {
        INSTANCE.progress_bar.lock().unwrap().update_anim_state();
        lifecycle!(target: "@", "");
        thread::sleep(Duration::from_millis(50));
    }
}

pub(crate) fn render_progress_bar(pb: &mut ProgressBar) -> std::io::Result<()> {
    if !INSTANCE.is_interactive {
        return Ok(());
    }
    let mut stderr = stderr().lock();
    let max = INSTANCE.max_targets.load(Ordering::Relaxed);
    let process_name = match INSTANCE.process_name.get() {
        Some(name) => name.to_owned(),
        None => "Executing".to_owned(),
    };

    // first line: progress bar
    pb.set_max(max);
    pb.set_current(INSTANCE.current_targets.load(Ordering::Relaxed));
    queue!(
        stderr,
        Print(format!("{: >12} ", process_name).cyan().bold()),
        Print(pb),
    )?;
    let _ = stderr.flush()?;

    // second line
    let in_progress_line = {
        let slab = INSTANCE.in_progress_targets.lock().unwrap();
        if slab.is_empty() {
            queue!(stderr, Clear(ClearType::UntilNewLine), MoveToColumn(0))?;
            return Ok(());
        }
        let mut unique_items = HashSet::with_capacity(slab.len());
        slab.iter()
            .map(|(_, v)| v.as_str())
            .filter(|it| {
                if !unique_items.contains(it) {
                    unique_items.insert(*it);
                    true
                } else {
                    false
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let max_length = if let Some((Width(w), _)) = terminal_size::terminal_size_of(&stderr) {
        (w as usize).saturating_sub(13).saturating_sub(60)
    } else {
        30
    };
    let in_progress_line = if in_progress_line.len() > max_length {
        format!("{}...", &in_progress_line[..(max_length.saturating_sub(3))])
    } else {
        in_progress_line
    };

    queue!(
        stderr,
        Print(": "),
        Print(in_progress_line),
        Clear(ClearType::UntilNewLine),
        MoveToColumn(0),
    )?;
    Ok(())
}

pub fn init_dashboard(params: InitDashboardParams) {
    INSTANCE
        .max_targets
        .store(params.requested_targets, Ordering::Relaxed);
    INSTANCE
        .requested_remotes
        .store(params.requested_remotes, Ordering::Relaxed);
    INSTANCE
        .loaded_packages
        .store(params.loaded_packages, Ordering::Relaxed);
    let _ = INSTANCE.process_name.set(params.process_name.to_string());
    let _ = INSTANCE.start_trigger.send(());
}

pub struct InitDashboardParams {
    pub requested_targets: usize,
    pub requested_remotes: usize,
    pub loaded_packages: usize,
    pub process_name: &'static str,
}

pub fn shutdown_dashboard() {
    let _ = INSTANCE.start_trigger.send(());
}

pub fn track_progress(name: String) -> InProgressItem {
    InProgressItem {
        id: INSTANCE.in_progress_targets.lock().unwrap().insert(name),
    }
}

pub struct InProgressItem {
    id: usize,
}

impl InProgressItem {
    pub fn mark_as_done(self) {
        INSTANCE.current_targets.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for InProgressItem {
    fn drop(&mut self) {
        if let Ok(mut targets) = INSTANCE.in_progress_targets.lock() {
            targets.remove(self.id);
        }
    }
}
