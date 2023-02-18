use crate::game::{Game, UpdateEvent};
use crossterm::style::Stylize;
use rand::Rng;
use std::time::Duration;
use strum::EnumCount;
use strum_macros::{EnumCount, FromRepr};

const HEIGHT: usize = 20;
const WIDTH: usize = 10;
const TO_DESCEND_SLOW: Duration = Duration::from_millis(200);
const TO_DESCEND_FAST: Duration = Duration::from_millis(50);
const MINIMUM_USER_INPUT_DISTANCE: Duration = Duration::from_millis(125);
const INIT_FIGURE_POS: Point = Point { x: 3.0, y: 0.0 };
const LOSE_LINE: f32 = 1.0;

enum UserInput {
    Left,
    Right,
    Rotate,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Figure {
    pub relative_points: [Point; 4],
    pub pivot: Point,
    pub rotation: f32, // in radians
}

#[derive(FromRepr, EnumCount)]
pub enum FigureType {
    Square,
    Line,
    L,
    LMirrored,
    Z,
    ZMirrored,
    T,
}

impl Figure {
    pub fn new(figure_type: FigureType, rotation: f32) -> Self {
        match figure_type {
            FigureType::Square => Self {
                relative_points: [
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 0.0, y: 1.0 },
                    Point { x: 1.0, y: 1.0 },
                ],
                pivot: Point { x: 0.5, y: 0.5 },
                rotation,
            },
            FigureType::Line => Self {
                relative_points: [
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 2.0, y: 0.0 },
                    Point { x: 3.0, y: 0.0 },
                ],
                pivot: Point { x: 1.5, y: 0.5 },
                rotation,
            },
            FigureType::L => Self {
                relative_points: [
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 0.0, y: 1.0 },
                    Point { x: 1.0, y: 1.0 },
                    Point { x: 2.0, y: 1.0 },
                ],
                pivot: Point { x: 1.0, y: 1.0 },
                rotation,
            },
            FigureType::LMirrored => Self {
                relative_points: [
                    Point { x: 0.0, y: 1.0 },
                    Point { x: 1.0, y: 1.0 },
                    Point { x: 2.0, y: 1.0 },
                    Point { x: 2.0, y: 0.0 },
                ],
                pivot: Point { x: 1.0, y: 1.0 },
                rotation,
            },
            FigureType::Z => Self {
                relative_points: [
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 },
                    Point { x: 2.0, y: 1.0 },
                ],
                pivot: Point { x: 1.0, y: 0.0 },
                rotation,
            },
            FigureType::ZMirrored => Self {
                relative_points: [
                    Point { x: 0.0, y: 1.0 },
                    Point { x: 1.0, y: 1.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 2.0, y: 0.0 },
                ],
                pivot: Point { x: 1.0, y: 0.0 },
                rotation,
            },
            FigureType::T => Self {
                relative_points: [
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 2.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 },
                ],
                pivot: Point { x: 1.0, y: 0.0 },
                rotation,
            },
        }
    }

    pub fn applied_rotation_and_position(&self, rotation: f32, position: Point) -> [Point; 4] {
        let mut points = self.relative_points;
        for point in points.iter_mut() {
            let x = point.x - self.pivot.x;
            let y = point.y - self.pivot.y;
            let x_new = x * rotation.cos() - y * rotation.sin();
            let y_new = x * rotation.sin() + y * rotation.cos();
            point.x = x_new + self.pivot.x + position.x;
            point.y = y_new + self.pivot.y + position.y;
        }
        points
    }
}

pub struct TetrisGame {
    pub board: [[bool; WIDTH]; HEIGHT],
    pub current_figure: Figure,
    pub current_figure_position: Point,
    pub next_figure: Figure,
    pub score: usize,
    pub to_descend: Duration,
    pub from_prev_descend: Duration,
    pub is_tetris_was_last: bool,

    last_user_input: UserInput,
    from_last_user_input: Duration,
}

impl TetrisGame {
    pub fn new() -> Self {
        Self {
            board: [[false; WIDTH]; HEIGHT],
            current_figure: Self::gen_figure(),
            current_figure_position: INIT_FIGURE_POS,
            next_figure: Self::gen_figure(),
            score: 0,
            to_descend: TO_DESCEND_SLOW,
            from_prev_descend: Duration::new(0, 0),
            is_tetris_was_last: false,

            last_user_input: UserInput::None,
            from_last_user_input: Duration::new(0, 0),
        }
    }

    pub fn gen_figure() -> Figure {
        Figure::new(
            FigureType::from_repr(rand::thread_rng().gen_range(0..FigureType::COUNT))
                .unwrap_or(FigureType::Square),
            0.0,
        )
    }

    fn is_line_ready(&self, row_num: usize) -> bool {
        self.board[row_num].iter().all(|&c| c)
    }
}

impl Game for TetrisGame {
    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        delta_time: &std::time::Duration,
    ) -> UpdateEvent {
        self.from_prev_descend += *delta_time;
        self.from_last_user_input += *delta_time;

        // Game over handle (TODO rework all update function, because it looks strange)
        {
            if self
                .current_figure
                .applied_rotation_and_position(
                    self.current_figure.rotation,
                    self.current_figure_position,
                )
                .iter()
                .any(|p| self.board[p.y.round() as usize][p.x.round() as usize])
            {
                return UpdateEvent::GameOver;
            }
        }

        // Input handling
        let (mut new_position, new_rotation) = {
            use crossterm::event::KeyCode;

            let mut new_rotation = self.current_figure.rotation;
            let mut new_position = self.current_figure_position;

            if let Some(input) = input {
                // Rotate and move
                if self.from_last_user_input > MINIMUM_USER_INPUT_DISTANCE {
                    match input.code {
                        KeyCode::Left => {
                            new_position.x -= 1.0;
                            self.last_user_input = UserInput::Left;
                        }
                        KeyCode::Right => {
                            new_position.x += 1.0;
                            self.last_user_input = UserInput::Right;
                        }
                        KeyCode::Up => {
                            new_rotation += std::f32::consts::PI / 2.0;
                            self.last_user_input = UserInput::Rotate;
                        }
                        _ => {}
                    }
                    self.from_last_user_input = Duration::new(0, 0);
                }
                // Descend faster
                if input.code == KeyCode::Down {
                    self.to_descend = TO_DESCEND_FAST
                } else {
                    self.to_descend = TO_DESCEND_SLOW
                }
            }

            (new_position, new_rotation)
        };

        // Apply descend (modifies new_position)
        if self.from_prev_descend > self.to_descend {
            new_position.y += 1.0;
            self.from_prev_descend = Duration::new(0, 0);
        }

        // Check if the figure can be moved to the new position
        let can_move = {
            let mut can_move = true;
            for point in self
                .current_figure
                .applied_rotation_and_position(new_rotation, new_position)
                .iter()
            {
                if point.x.round() < 0.0
                    || point.x.round() >= WIDTH as f32
                    || point.y.round() >= HEIGHT as f32
                    || self.board[point.y.round() as usize][point.x.round() as usize]
                {
                    can_move = false;
                }
            }
            can_move
        };

        // Move the figure if possible
        if can_move {
            self.current_figure_position = new_position;
            self.current_figure.rotation = new_rotation;
        }

        // Bake figure to self.board
        let is_figure_placed = if self
            .current_figure
            .applied_rotation_and_position(
                self.current_figure.rotation,
                self.current_figure_position,
            )
            .iter()
            .any(|p| {
                p.y.round() as usize >= HEIGHT - 1
                    || self.board[p.y.round() as usize + 1][p.x.round() as usize]
            }) {
            for p in self
                .current_figure
                .applied_rotation_and_position(
                    self.current_figure.rotation,
                    self.current_figure_position,
                )
                .iter()
            {
                self.board[p.y.round() as usize][p.x.round() as usize] = true
            }

            self.current_figure = self.next_figure;
            self.current_figure_position = INIT_FIGURE_POS;
            self.next_figure = Self::gen_figure();
            self.from_prev_descend = Duration::new(0, 0);
            self.to_descend = TO_DESCEND_SLOW;

            true
        } else {
            false
        };

        // Check for cleared lines
        {
            let mut curr_base_line = HEIGHT - 1 as usize;

            while curr_base_line > LOSE_LINE.round() as usize {
                let mut lines_in_row = 0;

                if !self.is_line_ready(curr_base_line) {
                    curr_base_line -= 1;
                    continue;
                }

                while self.is_line_ready(curr_base_line - lines_in_row) {
                    lines_in_row += 1;
                }

                self.score += if lines_in_row >= 4 {
                    if self.is_tetris_was_last {
                        300 * lines_in_row
                    } else {
                        self.is_tetris_was_last = true;
                        200 * lines_in_row
                    }
                } else {
                    self.is_tetris_was_last = false;
                    100 * lines_in_row
                };

                for col in 0..WIDTH {
                    for row in (0..=curr_base_line - lines_in_row).rev() {
                        self.board[row + lines_in_row][col] = self.board[row][col];
                    }
                }

                curr_base_line -= 1;
            }
        }

        if !is_figure_placed
            && !can_move
            && self
                .current_figure
                .applied_rotation_and_position(
                    self.current_figure.rotation,
                    self.current_figure_position,
                )
                .iter()
                .all(|p| p.y < LOSE_LINE)
        {
            print!("{:?}", self.current_figure_position);
            print!(
                "{:?}",
                self.current_figure.applied_rotation_and_position(
                    self.current_figure.rotation,
                    self.current_figure_position
                )
            );
            UpdateEvent::GameOver
        } else {
            UpdateEvent::GameContinue
        }
    }

    fn draw(
        &self,
        out: &mut std::io::Stdout,
        _delta_time: &std::time::Duration,
    ) -> crossterm::Result<()> {
        use crossterm::{cursor::MoveTo, execute};
        use std::io::Write;

        const BORDER_WIDTH: u16 = 2;

        // Draw the board
        {
            // Draw cells
            {
                for (y, row) in self.board.iter().enumerate() {
                    execute!(out, MoveTo(0, y as u16))?;
                    write!(out, " ║")?;
                    for &cell in row.iter() {
                        if cell {
                            write!(out, "██")?;
                        } else {
                            write!(out, "  ")?;
                        }
                    }
                    write!(out, "║ ")?;
                }
            }
            // Draw border
            {
                execute!(out, MoveTo(0, HEIGHT as u16))?;
                write!(out, " ╚")?;
                for _ in 0..WIDTH {
                    write!(out, "══")?;
                }
                write!(out, "╝ ")?;
            }
        }

        // Draw the current figure
        {
            for point in self
                .current_figure
                .applied_rotation_and_position(
                    self.current_figure.rotation,
                    self.current_figure_position,
                )
                .iter()
            {
                execute!(
                    out,
                    MoveTo(
                        BORDER_WIDTH + point.x.round() as u16 * 2,
                        point.y.round() as u16
                    )
                )?;
                write!(out, "██")?;
            }
        }

        // Draw score
        {
            fn digits_num(num: usize) -> u16 {
                if num == 0 {
                    1
                } else {
                    f32::floor(f32::log10(num as f32) + 1.0) as u16
                }
            }

            let score_hint = "Score: ";
            execute!(
                out,
                MoveTo(
                    (WIDTH as u16 + BORDER_WIDTH * 2
                        - score_hint.len() as u16
                        - digits_num(self.score))
                        / 2,
                    HEIGHT as u16 + 2
                )
            )?;

            let score = format!("{}", self.score);
            write!(
                out,
                "Score: {}",
                if self.score < 1_000 {
                    score.white()
                } else if self.score < 10_000 {
                    score.green()
                } else if self.score < 50_000 {
                    score.yellow()
                } else {
                    score.red()
                }
            )?;
        }

        execute!(out, MoveTo(0, 0))
    }

    fn get_score(&self) -> usize {
        self.score
    }
}
