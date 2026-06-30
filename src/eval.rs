//! Optional evaluation bar display (host-provided scores).

use crate::types::Color;

/// Centipawn / mate evaluation from white's perspective, supplied by the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvalDisplay {
    /// Centipawns from white's perspective (`+` = white better).
    pub cp: Option<i32>,
    /// Mate in full moves; positive = white delivers mate.
    pub mate: Option<i8>,
}

impl EvalDisplay {
    pub fn cp(cp: i32) -> Self {
        Self {
            cp: Some(cp),
            mate: None,
        }
    }

    pub fn mate(mate: i8) -> Self {
        Self {
            cp: None,
            mate: Some(mate),
        }
    }
}

/// Which side of the board square the gauge is drawn on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EvalBarPosition {
    #[default]
    Left,
    Right,
}

/// Runtime eval bar settings (mirrors [`crate::config::EvalConfigPatch`]).
#[derive(Clone, Debug, PartialEq)]
pub struct EvalBarConfig {
    pub enabled: bool,
    pub position: EvalBarPosition,
    /// `None` while the engine is thinking or before the first score arrives.
    pub display: Option<EvalDisplay>,
}

impl Default for EvalBarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            position: EvalBarPosition::Left,
            display: None,
        }
    }
}

/// White's winning chances in `[-1, 1]` (lichess `winningChances` formula).
pub fn white_winning_chance(eval: EvalDisplay) -> f64 {
    if let Some(mate) = eval.mate {
        mate_winning_chance(mate)
    } else if let Some(cp) = eval.cp {
        cp_winning_chance(cp)
    } else {
        0.0
    }
}

/// Black section height fraction for the gauge (`0` = all white, `1` = all black).
pub fn black_height_fraction(eval: Option<EvalDisplay>, orientation: Color) -> f64 {
    let ev = eval.map(white_winning_chance).unwrap_or(0.0);
    let black = (1.0 - ev) / 2.0;
    match orientation {
        Color::White => black.clamp(0.0, 1.0),
        Color::Black => (1.0 - black).clamp(0.0, 1.0),
    }
}

fn cp_winning_chance(cp: i32) -> f64 {
    raw_winning_chance(cp.clamp(-1000, 1000) as f64)
}

fn mate_winning_chance(mate: i8) -> f64 {
    let cp = (21 - i32::from(mate.unsigned_abs().min(10))) * 100;
    let signed = f64::from(cp) * if mate > 0 { 1.0 } else { -1.0 };
    raw_winning_chance(signed)
}

fn raw_winning_chance(cp: f64) -> f64 {
    2.0 / (1.0 + (-0.004 * cp).exp()) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_position_is_half_black() {
        let f = black_height_fraction(Some(EvalDisplay::cp(0)), Color::White);
        assert!((f - 0.5).abs() < 1e-6);
    }

    #[test]
    fn white_winning_shrinks_black_bar() {
        let f = black_height_fraction(Some(EvalDisplay::cp(500)), Color::White);
        assert!(f < 0.5);
    }

    #[test]
    fn searching_is_neutral() {
        let f = black_height_fraction(None, Color::White);
        assert!((f - 0.5).abs() < 1e-6);
    }

    #[test]
    fn orientation_flips_gauge() {
        let eval = EvalDisplay::cp(200);
        let white = black_height_fraction(Some(eval), Color::White);
        let black = black_height_fraction(Some(eval), Color::Black);
        assert!((white + black - 1.0).abs() < 1e-6);
    }
}
