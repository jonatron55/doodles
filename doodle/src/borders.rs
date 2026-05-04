// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

/// A style of character border.
pub enum BorderStyle {
    /// Single-line border: `┌─┐`.
    Single,

    /// Single-line border with curved corners: `╭─╮`.
    Curved,

    /// Bold single-line border: `┏━┓`.
    Bold,

    /// Double-line border: `╔═╗`.
    Double,
}

/// Single-line border characters for all combinations of connections.
pub const BORDERS_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '└', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '┌', // 0b0110 (SOUTH | EAST)
    '├', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '┘', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┴', // 0b1011 (NORTH | EAST | WEST)
    '┐', // 0b1100 (SOUTH | WEST)
    '┤', // 0b1101 (NORTH | SOUTH | WEST)
    '┬', // 0b1110 (SOUTH | EAST | WEST)
    '┼', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Curved border characters for all combinations of connections.
pub const BORDERS_CURVED: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '╰', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '╭', // 0b0110 (SOUTH | EAST)
    '├', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '╯', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┴', // 0b1011 (NORTH | EAST | WEST)
    '╮', // 0b1100 (SOUTH | WEST)
    '┤', // 0b1101 (NORTH | SOUTH | WEST)
    '┬', // 0b1110 (SOUTH | EAST | WEST)
    '┼', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Double-line border characters for all combinations of connections.
pub const BORDERS_DOUBLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╨', // 0b0001 (NORTH)
    '╞', // 0b0010 (EAST)
    '╚', // 0b0011 (NORTH | EAST)
    '╥', // 0b0100 (SOUTH)
    '║', // 0b0101 (NORTH | SOUTH)
    '╔', // 0b0110 (SOUTH | EAST)
    '╠', // 0b0111 (NORTH | EAST | SOUTH)
    '╡', // 0b1000 (WEST)
    '╝', // 0b1001 (NORTH | WEST)
    '═', // 0b1010 (EAST | WEST)
    '╩', // 0b1011 (NORTH | EAST | WEST)
    '╗', // 0b1100 (SOUTH | WEST)
    '╣', // 0b1101 (NORTH | SOUTH | WEST)
    '╦', // 0b1110 (SOUTH | EAST | WEST)
    '╬', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Border characters for double vertical and single horizontal lines.
pub const BORDERS_DOUBLE_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╨', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '╙', // 0b0011 (NORTH | EAST)
    '╥', // 0b0100 (SOUTH)
    '║', // 0b0101 (NORTH | SOUTH)
    '╓', // 0b0110 (SOUTH | EAST)
    '╟', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '╜', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '╨', // 0b1011 (NORTH | EAST | WEST)
    '╖', // 0b1100 (SOUTH | WEST)
    '╢', // 0b1101 (NORTH | SOUTH | WEST)
    '╥', // 0b1110 (SOUTH | EAST | WEST)
    '╫', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Border characters for single vertical and double horizontal lines.
pub const BORDERS_SINGLE_DOUBLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╞', // 0b0010 (EAST)
    '╘', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '╒', // 0b0110 (SOUTH | EAST)
    '╞', // 0b0111 (NORTH | EAST | SOUTH)
    '╡', // 0b1000 (WEST)
    '╛', // 0b1001 (NORTH | WEST)
    '═', // 0b1010 (EAST | WEST)
    '╧', // 0b1011 (NORTH | EAST | WEST)
    '╕', // 0b1100 (SOUTH | WEST)
    '╡', // 0b1101 (NORTH | SOUTH | WEST)
    '╤', // 0b1110 (SOUTH | EAST | WEST)
    '╪', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Bold border characters for all combinations of connections.
pub const BORDERS_BOLD: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╹', // 0b0001 (NORTH)
    '╺', // 0b0010 (EAST)
    '┗', // 0b0011 (NORTH | EAST)
    '╻', // 0b0100 (SOUTH)
    '┃', // 0b0101 (NORTH | SOUTH)
    '┏', // 0b0110 (SOUTH | EAST)
    '┣', // 0b0111 (NORTH | EAST | SOUTH)
    '╸', // 0b1000 (WEST)
    '┛', // 0b1001 (NORTH | WEST)
    '━', // 0b1010 (EAST | WEST)
    '┻', // 0b1011 (NORTH | EAST | WEST)
    '┓', // 0b1100 (SOUTH | WEST)
    '┫', // 0b1101 (NORTH | SOUTH | WEST)
    '┳', // 0b1110 (SOUTH | EAST | WEST)
    '╋', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Border characters for bold vertical and single horizontal lines.
pub const BORDERS_BOLD_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╹', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '┖', // 0b0011 (NORTH | EAST)
    '╻', // 0b0100 (SOUTH)
    '┃', // 0b0101 (NORTH | SOUTH)
    '┎', // 0b0110 (SOUTH | EAST)
    '┠', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '┚', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┸', // 0b1011 (NORTH | EAST | WEST)
    '┒', // 0b1100 (SOUTH | WEST)
    '┨', // 0b1101 (NORTH | SOUTH | WEST)
    '┰', // 0b1110 (SOUTH | EAST | WEST)
    '╂', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

/// Border characters for single vertical and bold horizontal lines.
pub const BORDERS_SINGLE_BOLD: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╺', // 0b0010 (EAST)
    '┕', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '┍', // 0b0110 (SOUTH | EAST)
    '┝', // 0b0111 (NORTH | EAST | SOUTH)
    '╸', // 0b1000 (WEST)
    '┙', // 0b1001 (NORTH | WEST)
    '━', // 0b1010 (EAST | WEST)
    '┷', // 0b1011 (NORTH | EAST | WEST)
    '┑', // 0b1100 (SOUTH | WEST)
    '┥', // 0b1101 (NORTH | SOUTH | WEST)
    '┯', // 0b1110 (SOUTH | EAST | WEST)
    '┿', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];
