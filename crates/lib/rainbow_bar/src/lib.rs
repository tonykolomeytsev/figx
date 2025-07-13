#![warn(missing_docs)]
//! # Rainbow Progress Bar
//!
//! A customizable terminal progress bar that automatically selects the most colorful rendering
//! based on terminal capabilities (monochrome, ANSI, or Xterm 256-color). It can also be
//! manually configured via [`ProgressBarOptions`].
//!
//! ## Usage
//!
//! Create a new progress bar with [`ProgressBar::new()`], then update its `current` and `max` values
//! and display it using its [`Display`] implementation:
//! ```
//! use lib_rainbow_bar::{ProgressBar, ProgressBarOptions};
//!
//! let mut pb = ProgressBar::new(ProgressBarOptions::default());
//! pb.max = 100;
//! pb.current = 42;
//! println!("{pb}");
//! ```
//!
//! For animated bars (Xterm only), call [`ProgressBar::update_anim_state()`] between frames.
//!
//! ## Palette Auto-Detection
//!
//! The bar automatically detects the best color mode using `supports-color`.
//!
//! Supported palettes:
//! - Monochrome (fallback)
//! - ANSI (8-bit color)
//! - Xterm (256-color with rainbow animation)
//!
//! ## Entry Point
//! - [`ProgressBar::new()`] is the main constructor.

use std::fmt::Display;
use supports_color::ColorLevel;

/// A terminal progress bar that adapts to the terminal's color capabilities.
///
/// Use the `max` and `current` fields to track progress.
/// Format the bar using [`std::fmt::Display`].
pub struct ProgressBar {
    /// The maximum progress value
    pub max: usize,
    /// The current progress value
    pub current: usize,
    width: usize,
    palette: Palette,
    ansi_colors: (u8, u8),
    anim_state: usize,
}

/// Configuration options for [`ProgressBar`].
///
/// Use this struct to override default bar width, color palette, or ANSI color codes.
pub struct ProgressBarOptions {
    /// Width of the bar in terminal columns (chars)
    pub bar_width: usize,
    /// Override the automatically detected palette
    pub override_palette: Option<Palette>,
    /// Override ANSI foreground colors: (bar, track)
    pub override_ansi_colors: Option<(u8, u8)>,
}

/// Available color palettes for rendering the progress bar.
///
/// This determines how the progress bar will be styled based on terminal capabilities:
/// - [`Palette::Monochrome`] – No colors; uses plain Unicode characters.
/// - [`Palette::Ansi`] – Basic 8-color ANSI escape codes.
/// - [`Palette::Xterm`] – Full 256-color Xterm palette with animated rainbow effects.
///
/// This is usually auto-selected based on the terminal’s color support, but can be overridden
/// manually via [`ProgressBarOptions`].
pub enum Palette {
    /// Renders the bar using only text characters, with no color
    Monochrome,
    /// Renders the bar using ANSI escape codes (standard 8-color support)
    Ansi,
    /// Renders the bar using 256-color Xterm codes and a rainbow animation
    Xterm,
}

impl Default for ProgressBarOptions {
    fn default() -> Self {
        Self {
            bar_width: 40,
            override_palette: None,
            override_ansi_colors: None,
        }
    }
}

impl ProgressBar {
    /// Default bar ansi color: Green color
    const ANSI_COLOR_BAR: u8 = 32;
    /// Default track ansi color: Bright black /or/ Grey
    const ANSI_COLOR_TRACK: u8 = 90;
    /// Looped sequence of the rainbow colors from here:
    /// https://www.hackitu.de/termcolor256/
    const XTERM_COLORS_BAR: &'static [u8; 30] = &[
        196, 202, 208, 214, 220, 226, 190, 154, 118, 82, 46, 47, 48, 49, 50, 51, 45, 39, 33, 27,
        21, 57, 93, 129, 165, 201, 200, 199, 198, 197,
    ];
    /// Looped sequence of the shades of grey
    const XTERM_COLORS_TRACK: &'static [u8; 10] =
        &[235, 236, 237, 238, 239, 240, 239, 238, 237, 236];
    const RESET_STYLE: u8 = 0;

    /// Creates a new [`ProgressBar`] using the given options.
    ///
    /// Automatically detects terminal capabilities unless overridden.
    pub fn new(opts: ProgressBarOptions) -> Self {
        Self {
            max: 0,
            current: 0,
            width: opts.bar_width,
            palette: opts.override_palette.unwrap_or_else(|| {
                match supports_color::on_cached(supports_color::Stream::Stderr) {
                    None => Palette::Monochrome,
                    Some(l) => match l {
                        ColorLevel { has_256: true, .. } => Palette::Xterm,
                        ColorLevel {
                            has_basic: true, ..
                        } => Palette::Ansi,
                        _ => Palette::Monochrome,
                    },
                }
            }),
            ansi_colors: opts
                .override_ansi_colors
                .unwrap_or_else(|| (Self::ANSI_COLOR_BAR, Self::ANSI_COLOR_TRACK)),
            anim_state: 0,
        }
    }

    /// Updates the internal animation state.
    ///
    /// Call this in a render loop to animate the bar in Xterm mode.
    pub fn update_anim_state(&mut self) {
        self.anim_state = self.anim_state.wrapping_add(1);
    }

    /// Returns the total width of the rendered progress bar string.
    ///
    /// This includes the progress fraction (e.g. `" 42/100"`).
    /// This not includes the ansi escape codes or any control symbols.
    pub fn len(&self) -> usize {
        let number1_len = self.max.checked_ilog10().unwrap_or(0) + 1;
        let number2_len = self.current.checked_ilog10().unwrap_or(0) + 1;
        // +2 because of space ' ' and '/' delimeter
        self.width + number1_len as usize + number2_len as usize + 2
    }

    #[inline]
    fn fmt_monochrome(&self, f: &mut std::fmt::Formatter<'_>, percent: f32) -> std::fmt::Result {
        let ProgressBar {
            max,
            current,
            width,
            palette: _,
            ansi_colors: _,
            anim_state: _,
        } = *self;
        let f2x = match (width as f32 * percent * 2.0) as usize {
            0 if percent > 0.0 => 1,
            val => val,
        };

        if f2x % 2 == 0 {
            let f1x = f2x / 2;
            let w = width.saturating_sub(f1x);
            write!(f, "{0:━>f1x$}{0: >w$}", "")?;
        }
        if f2x % 2 == 1 {
            let f1x = f2x / 2;
            let w = width.saturating_sub(f1x + 1);
            write!(f, "{0:━>f1x$}╸{0: >w$}", "")?;
        }
        write!(f, " {current}/{max}")
    }

    #[inline]
    fn fmt_ansi(&self, f: &mut std::fmt::Formatter<'_>, percent: f32) -> std::fmt::Result {
        let ProgressBar {
            max,
            current,
            width,
            palette: _,
            ansi_colors: (bar_color, track_color),
            anim_state: _,
        } = *self;
        let f2x = match (width as f32 * percent * 2.0) as usize {
            0 if percent > 0.0 => 1,
            val => val,
        };
        let f1x = f2x / 2;

        let w = width.saturating_sub(f1x + 1);
        write!(
            f,
            "\x1b[{bar_color}m{0:━>f1x$}{h1}\x1b[{track_color}m{h2}{0:━>w$}\x1b[{reset_color}m",
            "",
            reset_color = Self::RESET_STYLE,
            h1 = if f2x % 2 == 1 { "╸" } else { "" },
            h2 = if f2x % 2 == 0 && f1x < width {
                "╺"
            } else {
                ""
            },
        )?;

        write!(f, " {current}/{max}")
    }

    #[inline]
    fn fmt_xterm(&self, f: &mut std::fmt::Formatter<'_>, percent: f32) -> std::fmt::Result {
        let ProgressBar {
            max,
            current,
            width,
            palette: _,
            ansi_colors: _,
            anim_state,
        } = *self;
        let f2x = match (width as f32 * percent * 2.0) as usize {
            0 if percent > 0.0 => 1,
            val => val,
        };
        let f1x = f2x / 2;

        let track_color = if percent == 0.0 {
            Self::XTERM_COLORS_TRACK[anim_state % 10]
        } else {
            Self::XTERM_COLORS_TRACK[2]
        };

        let rev_anim_state = usize::MAX / 2 - anim_state;
        for i in 0..f1x {
            let color = Self::XTERM_COLORS_BAR[(i + rev_anim_state) % 30];
            write!(f, "\x1b[38;5;{color}m━")?;
        }
        if f2x % 2 == 1 {
            let color = Self::XTERM_COLORS_BAR[(f1x + rev_anim_state) % 30];
            write!(f, "\x1b[38;5;{color}m╸")?;
        } else if f1x < width {
            write!(f, "\x1b[38;5;{track_color}m╺")?;
        }
        for _ in f1x..width.saturating_sub(1) {
            write!(f, "\x1b[38;5;{track_color}m━")?;
        }

        write!(f, "\x1b[{}m {current}/{max}", Self::RESET_STYLE)
    }
}

impl Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let percent = match (self.current, self.max) {
            (0, _) | (_, 0) => 0.0,
            (c, m) => c as f32 / m as f32,
        };

        match self.palette {
            Palette::Monochrome => self.fmt_monochrome(f, percent),
            Palette::Ansi => self.fmt_ansi(f, percent),
            Palette::Xterm => self.fmt_xterm(f, percent),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn test_monochrome_in_action() {
        let mut pb = ProgressBar::new(ProgressBarOptions {
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 146;
        for i in 0..=146 {
            pb.current = i;
            eprint!("\r{pb} ");
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn test_ansi_in_action() {
        let mut pb = ProgressBar::new(ProgressBarOptions {
            override_palette: Some(Palette::Ansi),
            ..Default::default()
        });
        pb.max = 146;
        for i in 0..=146 {
            pb.current = i;
            eprint!("\r{pb} ");
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn test_xterm_in_action() {
        let mut pb = ProgressBar::new(ProgressBarOptions {
            override_palette: Some(Palette::Xterm),
            ..Default::default()
        });
        pb.max = 146;
        for _ in 0..10 {
            pb.update_anim_state();
            eprint!("\r{pb} ");
            thread::sleep(Duration::from_millis(50));
        }
        for i in 0..=146 {
            pb.current = i;
            pb.update_anim_state();
            eprint!("\r{pb} ");
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn test_monochrome_progress_0of0() {
        // Given
        let mut pb = ProgressBar::new(ProgressBarOptions {
            bar_width: 10,
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 0;
        pb.current = 0;

        // When
        let output = pb.to_string();
        let length = pb.len();

        // Then
        assert_eq!("           0/0", output);
        assert_eq!(length, output.chars().count());
    }

    #[test]
    fn test_monochrome_progress_0of100() {
        // Given
        let mut pb = ProgressBar::new(ProgressBarOptions {
            bar_width: 10,
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 100;
        pb.current = 0;

        // When
        let output = pb.to_string();
        let length = pb.len();

        // Then
        assert_eq!("           0/100", output);
        assert_eq!(length, output.chars().count());
    }

    #[test]
    fn test_monochrome_progress_1of100() {
        // Given
        let mut pb = ProgressBar::new(ProgressBarOptions {
            bar_width: 10,
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 100;
        pb.current = 1;

        // When
        let output = pb.to_string();
        let length = pb.len();

        // Then
        assert_eq!("╸          1/100", output);
        assert_eq!(length, output.chars().count());
    }

    #[test]
    fn test_monochrome_progress_50of100() {
        // Given
        let mut pb = ProgressBar::new(ProgressBarOptions {
            bar_width: 10,
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 100;
        pb.current = 50;

        // When
        let output = pb.to_string();
        let length = pb.len();

        // Then
        assert_eq!("━━━━━      50/100", output);
        assert_eq!(length, output.chars().count());
    }

    #[test]
    fn test_monochrome_progress_100of100() {
        // Given
        let mut pb = ProgressBar::new(ProgressBarOptions {
            bar_width: 10,
            override_palette: Some(Palette::Monochrome),
            ..Default::default()
        });
        pb.max = 100;
        pb.current = 100;

        // When
        let output = pb.to_string();
        let length = pb.len();

        // Then
        assert_eq!("━━━━━━━━━━ 100/100", output);
        assert_eq!(length, output.chars().count());
    }
}
