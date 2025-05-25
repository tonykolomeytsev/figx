use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::{LazyLock, atomic::AtomicUsize};

use ordermap::OrderSet;

static PROGRESS_BAR: LazyLock<ProgressBar> = LazyLock::new(ProgressBar::default);
const PROGRESS_BAR_WIDTH: usize = 30;

#[derive(Default)]
pub struct ProgressBar {
    max: AtomicUsize,
    current: AtomicUsize,
    visible: AtomicBool,
    in_progress_items: Arc<Mutex<OrderSet<String>>>,
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
        let mut length = 0;
        for (idx, item) in self
            .in_progress_items
            .as_ref()
            .lock()
            .unwrap()
            .iter()
            .enumerate()
        {
            if idx == 0 {
                write!(f, ": ")?;
            }
            length += item.len();
            if length > 60 {
                write!(f, ", ...")?;
                break;
            }
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }

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

pub fn create_in_progress_item<'a>(name: &'a str) -> InProgressGuard<'a> {
    let guard = InProgressGuard(name);
    PROGRESS_BAR
        .in_progress_items
        .lock()
        .unwrap()
        .insert(name.to_owned());
    return guard;
}

pub struct InProgressGuard<'a>(&'a str);

impl<'a> Drop for InProgressGuard<'a> {
    fn drop(&mut self) {
        if let Ok(mut items) = PROGRESS_BAR.in_progress_items.lock() {
            items.remove(self.0);
        }
    }
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
