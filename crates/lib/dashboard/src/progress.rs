use crossterm::style::{Color, Stylize};
use std::{cmp::min, fmt::Display};

pub(crate) struct ProgressBar {
    max: usize,
    current: usize,
    anim_state: usize,
    palette: Palette,
    width: usize,
}

impl Default for ProgressBar {
    fn default() -> Self {
        let term_width = if let Some((w, _)) = term_size::dimensions_stderr() {
            min(40, w)
        } else {
            40
        };
        Self {
            max: 0,
            current: 0,
            anim_state: 0,
            palette: Default::default(),
            width: term_width,
        }
    }
}

pub(crate) enum Palette {
    Mono,
    Ansi,
}

impl Default for Palette {
    fn default() -> Self {
        if let Some(_) = supports_color::on_cached(supports_color::Stream::Stderr) {
            Self::Ansi
        } else {
            Self::Mono
        }
    }
}

impl ProgressBar {
    // From https://www.hackitu.de/termcolor256/
    const ANSI_PRIMARY: [u8; 30] = [
        196, 202, 208, 214, 220, 226, 190, 154, 118, 82, 46, 47, 48, 49, 50, 51, 45, 39, 33, 27,
        21, 57, 93, 129, 165, 201, 200, 199, 198, 197,
    ];
    const ANSI_SECONDARY: [u8; 10] = [235, 236, 237, 238, 239, 240, 239, 238, 237, 236];

    pub fn update_anim_state(&mut self) {
        self.anim_state = self.anim_state.wrapping_add(1);
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = max;
    }

    pub fn set_current(&mut self, current: usize) {
        self.current = current;
    }
}

impl Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = self.max;
        let current = self.current;
        let percent = if current == 0 || max == 0 {
            0.0
        } else {
            current as f32 / max as f32
        };
        let filled_area = (self.width as f32 * percent * 3.0) as usize / 3;
        match self.palette {
            Palette::Mono => {
                if filled_area > 0 {
                    // Available blocks: '╸', '━', '╺'
                    if filled_area < self.width {
                        write!(f, "{}{}", "━".repeat(filled_area - 1).green(), "╸".green())?;
                    } else {
                        write!(f, "{}", "━".repeat(filled_area).green())?;
                    }
                }
                if filled_area < self.width {
                    write!(f, "{}", "━".repeat(self.width - filled_area).dark_grey())?;
                }
            }
            Palette::Ansi => {
                // Available blocks: '╸', '━', '╺'
                let state = self.anim_state;
                if filled_area > 0 {
                    for i in 0..(filled_area - 1) {
                        let reverse_index = filled_area - 1 - i;
                        let color_index =
                            (state as usize + reverse_index) % Self::ANSI_PRIMARY.len();
                        let ansi_color = Self::ANSI_PRIMARY[color_index];
                        write!(f, "{}", "━".with(Color::AnsiValue(ansi_color)))?;
                    }
                    let color_index = state as usize % 30;
                    let ansi_color = Self::ANSI_PRIMARY[color_index];

                    if filled_area < self.width {
                        write!(f, "{}", "╸".with(Color::AnsiValue(ansi_color)))?;
                    }
                }
                if filled_area < self.width {
                    let ansi_color = if filled_area == 0 {
                        let color_index = (state / 2) as usize % Self::ANSI_SECONDARY.len();
                        Self::ANSI_SECONDARY[color_index]
                    } else {
                        Self::ANSI_SECONDARY[0]
                    };
                    write!(
                        f,
                        "{}",
                        "━"
                            .repeat(self.width - filled_area - 1)
                            .with(Color::AnsiValue(ansi_color))
                    )?;
                }
            }
        }
        if max > 0 {
            write!(f, " {current}/{max}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    // #[ignore = "only for manual testing"]
    fn test_in_action() {
        let mut pb = ProgressBar::default();
        pb.set_max(146);
        eprint!("\r{pb} ");
        for i in 0..=(10 + 146) {
            pb.set_current(if i > 10 { i - 10 } else { 0 });
            pb.update_anim_state();
            eprint!("\r{pb} ");
            thread::sleep(Duration::from_millis(50));
        }
        println!();
    }
}
