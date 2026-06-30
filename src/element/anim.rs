//! Move animation planning and interpolation. Port of chessground `anim.ts`.

use std::collections::HashMap;
use std::time::Instant;

use crate::state::HeadlessState;
use crate::types::{Key, Piece, Pieces, Pos};
use crate::util::{all_keys, distance_sq, key_to_pos, same_piece};

/// `[goal_file, goal_rank, current_file, current_rank]` in grid units.
pub type AnimVector = [f32; 4];

#[derive(Clone, Debug, Default)]
pub struct AnimPlan {
    pub anims: HashMap<Key, AnimVector>,
    pub fadings: HashMap<Key, Piece>,
}

#[derive(Clone, Debug)]
pub struct AnimCurrent {
    pub start: Instant,
    pub plan: AnimPlan,
}

#[derive(Clone)]
struct AnimPiece {
    key: Key,
    pos: Pos,
    piece: Piece,
}

fn make_piece(key: Key, piece: Piece) -> AnimPiece {
    let pos = key_to_pos(&key);
    AnimPiece { key, pos, piece }
}

fn closer<'a>(piece: &AnimPiece, candidates: &'a [AnimPiece]) -> Option<&'a AnimPiece> {
    candidates
        .iter()
        .min_by_key(|p| distance_sq(piece.pos, p.pos))
}

fn compute_plan(prev_pieces: &Pieces, current: &HeadlessState) -> AnimPlan {
    let mut anims = HashMap::new();
    let mut animed_origs = Vec::new();
    let mut fadings = HashMap::new();
    let mut missings = Vec::new();
    let mut news = Vec::new();
    let mut pre_pieces = HashMap::new();

    for (k, p) in prev_pieces {
        pre_pieces.insert(k.clone(), make_piece(k.clone(), *p));
    }

    for key in all_keys() {
        let cur = current.pieces.get(&key).copied();
        let pre = pre_pieces.get(&key);
        match (cur, pre) {
            (Some(cur_p), Some(pre_p)) if !same_piece(&cur_p, &pre_p.piece) => {
                missings.push(pre_p.clone());
                news.push(make_piece(key.clone(), cur_p));
            }
            (Some(cur_p), None) => news.push(make_piece(key.clone(), cur_p)),
            (None, Some(pre_p)) => missings.push(pre_p.clone()),
            _ => {}
        }
    }

    for new_p in &news {
        let candidates: Vec<AnimPiece> = missings
            .iter()
            .filter(|p| same_piece(&new_p.piece, &p.piece))
            .cloned()
            .collect();
        if let Some(pre_p) = closer(new_p, &candidates) {
            let vector = [
                pre_p.pos.file as f32 - new_p.pos.file as f32,
                pre_p.pos.rank as f32 - new_p.pos.rank as f32,
                0.,
                0.,
            ];
            let mut anim = vector;
            anim[2] = anim[0];
            anim[3] = anim[1];
            anims.insert(new_p.key.clone(), anim);
            animed_origs.push(pre_p.key.clone());
        }
    }

    for p in missings {
        if !animed_origs.contains(&p.key) && !anims.contains_key(&p.key) {
            fadings.insert(p.key, p.piece);
        }
    }

    AnimPlan { anims, fadings }
}

fn easing(t: f32) -> f32 {
    if t < 0.5 {
        4. * t * t * t
    } else {
        (t - 1.) * (2. * t - 2.) * (2. * t - 2.) + 1.
    }
}

/// Run `mutation` with animation when enabled.
pub fn anim<F, R>(mutation: F, state: &mut HeadlessState) -> R
where
    F: FnOnce(&mut HeadlessState) -> R,
{
    if state.animation.enabled {
        animate(mutation, state)
    } else {
        render(mutation, state)
    }
}

/// Run `mutation` and clear any in-flight animation plan.
pub fn render<F, R>(mutation: F, state: &mut HeadlessState) -> R
where
    F: FnOnce(&mut HeadlessState) -> R,
{
    state.animation.current = None;
    mutation(state)
}

fn animate<F, R>(mutation: F, state: &mut HeadlessState) -> R
where
    F: FnOnce(&mut HeadlessState) -> R,
{
    let prev_pieces = state.pieces.clone();
    let result = mutation(state);
    let plan = compute_plan(&prev_pieces, state);
    if plan.anims.is_empty() && plan.fadings.is_empty() {
        state.animation.current = None;
    } else {
        state.animation.current = Some(AnimCurrent {
            start: Instant::now(),
            plan,
        });
    }
    result
}

/// Advance the animation clock. Returns `true` while frames are still needed.
pub fn step(state: &mut HeadlessState) -> bool {
    let Some(cur) = state.animation.current.as_mut() else {
        return false;
    };
    let duration_ms = state.animation.duration.max(1) as f32;
    let elapsed = cur.start.elapsed().as_secs_f32() * 1000.;
    let rest = 1. - elapsed / duration_ms;
    if rest <= 0. {
        state.animation.current = None;
        return false;
    }
    let ease = easing(rest);
    for anim in cur.plan.anims.values_mut() {
        anim[2] = anim[0] * ease;
        anim[3] = anim[1] * ease;
    }
    true
}

/// Grid-unit offset applied when painting `key` during animation.
pub fn offset_for(state: &HeadlessState, key: &Key) -> Option<(f32, f32)> {
    let cur = state.animation.current.as_ref()?;
    let anim = cur.plan.anims.get(key)?;
    Some((anim[2], anim[3]))
}

pub fn is_fading(state: &HeadlessState, key: &Key) -> bool {
    let Some(cur) = state.animation.current.as_ref() else {
        return false;
    };
    let Some(fading_piece) = cur.plan.fadings.get(key) else {
        return false;
    };
    state.pieces.get(key) == Some(fading_piece)
}

pub fn cancel_for_key(state: &mut HeadlessState, key: &Key) {
    if let Some(cur) = state.animation.current.as_mut() {
        cur.plan.anims.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::HeadlessState;
    use crate::types::{Color, Key, Piece, Role};

    #[test]
    fn anim_plan_links_moved_piece() {
        let mut state = HeadlessState::defaults();
        let e2 = Key::new("e2").unwrap();
        let e4 = Key::new("e4").unwrap();
        let prev = state.pieces.clone();
        state.pieces.remove(&e2);
        state.pieces.insert(e4.clone(), Piece {
            role: crate::types::Role::Pawn,
            color: Color::White,
            promoted: false,
        });
        let plan = compute_plan(&prev, &state);
        assert!(plan.anims.contains_key(&e4));
    }

    #[test]
    fn capture_does_not_fade_dest_square() {
        let mut state = HeadlessState::defaults();
        let f3 = Key::new("f3").unwrap();
        let e5 = Key::new("e5").unwrap();
        let knight = Piece {
            role: Role::Knight,
            color: Color::White,
            promoted: false,
        };
        let pawn = Piece {
            role: Role::Pawn,
            color: Color::Black,
            promoted: false,
        };
        state.pieces.clear();
        state.pieces.insert(f3.clone(), knight);
        state.pieces.insert(e5.clone(), pawn);
        let prev = state.pieces.clone();
        state.pieces.remove(&f3);
        state.pieces.insert(e5.clone(), knight);
        let plan = compute_plan(&prev, &state);
        assert!(plan.anims.contains_key(&e5));
        assert!(!plan.fadings.contains_key(&e5));
    }

    #[test]
    fn is_fading_does_not_hide_replacing_piece() {
        let mut state = HeadlessState::defaults();
        let e5 = Key::new("e5").unwrap();
        let knight = Piece {
            role: Role::Knight,
            color: Color::White,
            promoted: false,
        };
        let pawn = Piece {
            role: Role::Pawn,
            color: Color::Black,
            promoted: false,
        };
        state.pieces.insert(e5.clone(), knight);
        let mut plan = AnimPlan::default();
        plan.fadings.insert(e5.clone(), pawn);
        state.animation.current = Some(AnimCurrent {
            start: Instant::now(),
            plan,
        });
        assert!(!is_fading(&state, &e5));
    }
}
