use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, atomic::AtomicUsize};

static PROGRESS_BAR: LazyLock<ProgressBar> = LazyLock::new(ProgressBar::default);
const PROGRESS_BAR_WIDTH: usize = 30;

#[derive(Default)]
pub struct ProgressBar {
    max: AtomicUsize,
    current: AtomicUsize,
    visible: AtomicBool,
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
        let percent = if current == 0 || max == 0 {
            0.0
        } else {
            current as f32 / max as f32
        };
        let filled_area = (PROGRESS_BAR_WIDTH as f32 * percent) as usize;
        write!(f, "[")?;
        if filled_area > 0 {
            write!(f, "{}>", "=".repeat(filled_area - 1))?;
        }
        if filled_area < PROGRESS_BAR_WIDTH {
            write!(f, "{}", " ".repeat(PROGRESS_BAR_WIDTH - filled_area))?;
        }
        write!(f, "] {current}/{max}")?;
        Ok(())
    }
}

pub fn set_progress_bar_maximum(max: usize) {
    PROGRESS_BAR.init(max);
}

pub fn set_progress_bar_progress(current: usize) {
    PROGRESS_BAR.set_progress(current);
}

pub fn is_progress_bar_visible() -> bool {
    PROGRESS_BAR.visible.load(Ordering::Relaxed)
}

pub fn set_progress_bar_visible(visible: bool) {
    PROGRESS_BAR.visible.store(visible, Ordering::Relaxed);
}

pub fn get_progress_bar_display() -> String {
    PROGRESS_BAR.to_string()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use std::{thread, time::Duration};

    use crate::{ProgressBar, set_progress_bar_maximum, set_progress_bar_progress};

    #[test]
    fn test_empty_progress_bar() {
        // Given
        let pb = ProgressBar::default();
        pb.init(146);
        pb.set_progress(0);
        let expected = "[                              ] 0/146";

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
        let expected = "[======>                       ] 36/146";

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
        let expected = "[=============================>] 146/146";

        // When
        let actual = pb.to_string();

        // Then
        assert_eq!(expected, actual);
    }

    #[test]
    #[ignore = "only for manual testing"]
    fn test_in_action() {
        set_progress_bar_maximum(146);
        for i in 0..146 {
            set_progress_bar_progress(i);
            thread::sleep(Duration::from_millis(100));
        }
    }
}
