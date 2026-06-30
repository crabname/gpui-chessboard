//! Premove heuristics. Port of chessground `premove.ts`.

use crate::state::HeadlessState;
use crate::types::{Color, Key, MobilityContext, Pieces, PosAndKey, Role};
use crate::util::{
    bishop_dir, diff, key_to_pos, knight_dir, king_dir_non_castling, pawn_dir_advance, rook_dir,
};

fn pawn_mobility(ctx: &MobilityContext) -> bool {
    let ox = ctx.orig.pos.file as i32;
    let oy = ctx.orig.pos.rank as i32;
    let dx = ctx.dest.pos.file as i32;
    let dy = ctx.dest.pos.rank as i32;
    if diff(ox, dx) > 1 {
        return false;
    }
    if diff(ox, dx) == 1 {
        let step = if ctx.color == Color::White { 1 } else { -1 };
        dy == oy + step
    } else {
        pawn_dir_advance(ox, oy, dx, dy, ctx.color == Color::White)
    }
}

fn knight_mobility(ctx: &MobilityContext) -> bool {
    knight_dir(
        ctx.orig.pos.file as i32,
        ctx.orig.pos.rank as i32,
        ctx.dest.pos.file as i32,
        ctx.dest.pos.rank as i32,
    )
}

fn bishop_mobility(ctx: &MobilityContext) -> bool {
    bishop_dir(
        ctx.orig.pos.file as i32,
        ctx.orig.pos.rank as i32,
        ctx.dest.pos.file as i32,
        ctx.dest.pos.rank as i32,
    )
}

fn rook_mobility(ctx: &MobilityContext) -> bool {
    rook_dir(
        ctx.orig.pos.file as i32,
        ctx.orig.pos.rank as i32,
        ctx.dest.pos.file as i32,
        ctx.dest.pos.rank as i32,
    )
}

fn queen_mobility(ctx: &MobilityContext) -> bool {
    bishop_mobility(ctx) || rook_mobility(ctx)
}

fn king_mobility(ctx: &MobilityContext) -> bool {
    let ox = ctx.orig.pos.file as i32;
    let oy = ctx.orig.pos.rank as i32;
    let dx = ctx.dest.pos.file as i32;
    let dy = ctx.dest.pos.rank as i32;
    king_dir_non_castling(ox, oy, dx, dy)
        || (oy == dy
            && oy == if ctx.color == Color::White { 0 } else { 7 }
            && ((ox == 4
                && ((dx == 2 && ctx.rook_files_friendlies.contains(&0))
                    || (dx == 6 && ctx.rook_files_friendlies.contains(&7))))
                || ctx.rook_files_friendlies.contains(&(dx as u8))))
}

fn mobility_by_role(role: Role, ctx: &MobilityContext) -> bool {
    match role {
        Role::Pawn => pawn_mobility(ctx),
        Role::Knight => knight_mobility(ctx),
        Role::Bishop => bishop_mobility(ctx),
        Role::Rook => rook_mobility(ctx),
        Role::Queen => queen_mobility(ctx),
        Role::King => king_mobility(ctx),
    }
}

fn filter_pieces(pieces: &Pieces, color: Color) -> Pieces {
    pieces
        .iter()
        .filter(|(_, p)| p.color == color)
        .map(|(k, p)| (k.clone(), *p))
        .collect()
}

fn rook_files_friendlies(pieces: &Pieces, color: Color) -> Vec<u8> {
    let rank = if color == Color::White { '1' } else { '8' };
    pieces
        .iter()
        .filter(|(k, p)| k.as_str().ends_with(rank) && p.color == color && p.role == Role::Rook)
        .map(|(k, _)| key_to_pos(k).file)
        .collect()
}

/// Compute heuristic premove destinations for a piece (no legality engine).
pub fn premove(state: &HeadlessState, key: &Key) -> Vec<Key> {
    let pieces = &state.pieces;
    let Some(piece) = pieces.get(key) else {
        return Vec::new();
    };
    if piece.color == state.turn_color {
        return Vec::new();
    }
    let color = piece.color;
    let friendlies = filter_pieces(pieces, color);
    let enemies = filter_pieces(pieces, crate::types::opposite(color));
    let orig = PosAndKey {
        key: key.clone(),
        pos: key_to_pos(key),
    };
    let rook_files = rook_files_friendlies(pieces, color);
    let last_move = state.last_move.clone();

    crate::util::all_pos_and_key()
        .into_iter()
        .filter(|dest| {
            let ctx = MobilityContext {
                orig: orig.clone(),
                dest: dest.clone(),
                role: piece.role,
                all_pieces: pieces.clone(),
                friendlies: friendlies.clone(),
                enemies: enemies.clone(),
                color,
                rook_files_friendlies: rook_files.clone(),
                last_move: last_move.clone(),
            };
            mobility_by_role(piece.role, &ctx)
        })
        .map(|pk| pk.key)
        .collect()
}
