use std::fmt::Display;
use std::io::{Write, stderr};
use std::sync::atomic::Ordering;
use std::sync::{LazyLock, atomic::AtomicUsize};

use owo_colors::OwoColorize;

static HANDLE: LazyLock<ProgressBar> = LazyLock::new(|| ProgressBar::default());
const PROGRESS_BAR_WIDTH: usize = 30;

#[derive(Default)]
pub struct ProgressBar {
    max: AtomicUsize,
    current: AtomicUsize,
}

impl ProgressBar {
    fn init(&self, max: usize) {
        self.max.store(max, Ordering::Relaxed);
    }

    fn set_progress(&self, current: usize) {
        self.current.store(current, Ordering::Relaxed);
    }
}

impl Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = self.current.load(Ordering::Relaxed);
        let max = self.max.load(Ordering::Relaxed);
        let percent = current as f32 / max as f32;
        let filled_area = (PROGRESS_BAR_WIDTH as f32 * percent) as usize;
        write!(f, "[")?;
        if filled_area > 0 {
            write!(f, "{}>", "=".repeat(filled_area - 1))?;
        }
        if filled_area < PROGRESS_BAR_WIDTH {
            write!(f, "{}", " ".repeat(PROGRESS_BAR_WIDTH - filled_area))?;
        }
        let percent = percent * 100.0;
        write!(f, "] {percent:.0}%")?;
        Ok(())
    }
}

pub fn init(max: usize) {
    HANDLE.init(max);
}

pub fn set_progress(current: usize) {
    HANDLE.set_progress(current);
}

pub fn display_progress_bar() -> std::io::Result<()> {
    let mut out = stderr().lock();
    // TODO: no allocations
    // TODO: clear line before print
    write!(
        out,
        "{label: >12} {bar}\r",
        label = "Progress".bold().green(),
        bar = HANDLE.to_string()
    )
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use std::{thread, time::Duration};

    use crate::{ProgressBar, display_progress_bar, init, set_progress};

    #[test]
    fn test_empty_progress_bar() {
        // Given
        let pb = ProgressBar::default();
        pb.init(146);
        pb.set_progress(0);
        let expected = "[                              ] 0%";

        // When
        let actual = pb.to_string();

        // Then
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_25p_progress_bar() {
        // Given
        let pb = ProgressBar::default();
        pb.init(146);
        pb.set_progress(36);
        let expected = "[======>                       ] 25%";

        // When
        let actual = pb.to_string();

        // Then
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_full_progress_bar() {
        // Given
        let pb = ProgressBar::default();
        pb.init(146);
        pb.set_progress(146);
        let expected = "[=============================>] 100%";

        // When
        let actual = pb.to_string();

        // Then
        assert_eq!(expected, actual);
    }

    #[test]
    #[ignore = "only for manual testing"]
    fn test_in_action() {
        init(146);
        for i in 0..146 {
            set_progress(i);
            let _ = display_progress_bar();
            thread::sleep(Duration::from_millis(100));
        }
    }
}
