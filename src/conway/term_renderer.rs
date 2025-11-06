use std::{
    hash::{BuildHasher, Hash, Hasher, RandomState},
    io::{Result as IoResult, Write as _, stdout},
};

use crate::board::Board;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{ContentStyle, PrintStyledContent},
};
use doodles::common::term::{BOLD_STYLES, DIM_STYLES};

/// Glyphs used to represent cells.
///
/// Each row corresponds to a different random variation, and each column
/// corresponds to a different age, with the first column used for living cells.
/// When a cell is dead, it ages through the columns from left to right with
/// each generation. Once it reaches the last column, it will remain there.
const CELL_GLYPHS: [[char; 12]; 8] = [
    ['█', '▓', '▒', '░', '⣿', '⡿', '⡾', '⡶', '⠶', '⠦', '⠢', '⠠'],
    ['█', '▓', '▒', '░', '⣿', '⣾', '⣺', '⡺', '⡪', '⢊', '⢈', '⠈'],
    ['█', '▓', '▒', '░', '⣿', '⢿', '⢟', '⢝', '⢜', '⢘', '⢈', '⢀'],
    ['█', '▓', '▒', '░', '⣿', '⣻', '⣛', '⣚', '⣒', '⢒', '⢂', '⢀'],
    ['█', '▓', '▒', '░', '⣿', '⣯', '⣫', '⢫', '⢪', '⢨', '⠨', '⠈'],
    ['█', '▓', '▒', '░', '⣿', '⣟', '⣗', '⣓', '⣃', '⢃', '⢁', '⠁'],
    ['█', '▓', '▒', '░', '⣿', '⡿', '⠿', '⠯', '⠭', '⠍', '⠅', '⠄'],
    ['█', '▓', '▒', '░', '⣿', '⣽', '⣼', '⡼', '⡬', '⠬', '⠌', '⠄'],
];

/// Renders the given board to the terminal.
///
/// A border is drawn around the board, and each cell is rendered using colored
/// glyphs that indicate different ages. These glyphs are selected randomly from
/// a predefined set to add visual variety.
///
/// Although the simulation supports an arbitrary number of colors, only six
/// distinct terminal colors are available. Colors will repeat if more than six
/// are used.
///
/// This uses low-level terminal commands to render the board at a fixed
/// position and size. It should render without flickering on most terminals.
/// The caller is responsible for clearing the terminal and hiding the cursor
/// before the first call to this function and restoring them afterwards.
///
/// Arguments
/// =========
///
/// - `board` - The board to render.
/// - `random_state` - A random state used to generate consistent random values
///   between frames (e.g., for selecting glyph variations).
///
/// Returns
/// =======
///
/// `Ok(())` if the rendering was successful, or a [`std::io::Error`] if any
/// problems occurred during terminal output.
pub fn render(board: &Board, random_state: &RandomState) -> IoResult<()> {
    let (width, height) = board.size();
    let mut stdout = stdout();

    for y in 0..height {
        queue!(stdout, MoveTo(0, y as u16),)?;

        for x in 0..width {
            let cell = board.cell(x, y);

            if cell.is_empty() {
                queue!(
                    stdout,
                    PrintStyledContent(ContentStyle::default().apply(" "))
                )?;
                continue;
            }

            let color = cell.color.map(|color| (color as usize) % BOLD_STYLES.len());
            let style = if cell.is_alive() {
                color.map(|i| BOLD_STYLES[i]).unwrap()
            } else {
                color.map(|i| DIM_STYLES[i]).unwrap()
            };

            let col = if cell.is_alive() {
                0
            } else {
                ((cell.age - 1) as usize).min(CELL_GLYPHS[0].len() - 1)
            };

            let mut hasher = random_state.build_hasher();
            x.hash(&mut hasher);
            y.hash(&mut hasher);
            let row = hasher.finish() as usize % CELL_GLYPHS.len();

            let glyph = CELL_GLYPHS[row][col];

            queue!(stdout, PrintStyledContent(style.apply(glyph)))?;
        }
    }

    stdout.flush()?;

    Ok(())
}
