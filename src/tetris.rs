use crate::game::Game;

const HEIGHT: usize = 20;
const WIDTH: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Figure {
    pub relative_points: [Point; 4],
    pub pivot: Point,
    pub rotation: f32,
}

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
                pivot: Point { x: 1.5, y: 0.0 },
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

    pub fn applied_rotation_and_position(&self, position: Point) -> [Point; 4] {
        let mut points = self.relative_points;
        for point in points.iter_mut() {
            let x = point.x - self.pivot.x;
            let y = point.y - self.pivot.y;
            let x_new = x * self.rotation.cos() - y * self.rotation.sin();
            let y_new = x * self.rotation.sin() + y * self.rotation.cos();
            point.x = x_new + self.pivot.x + position.x;
            point.y = y_new + self.pivot.y + position.y;
        }
        points
    }
}

pub struct TetrisGame {
    pub board: [[bool; 10]; 20],
    pub current_figure: Figure,
    pub current_figure_position: Point,
    pub next_figure: Figure,
    pub score: u32,
}

impl TetrisGame {
    pub fn new() -> Self {
        Self {
            board: [[false; WIDTH]; HEIGHT],
            current_figure: Figure::new(FigureType::Square, 0.0),
            current_figure_position: Point { x: 0.0, y: 0.0 },
            next_figure: Figure::new(FigureType::Square, 0.0),
            score: 0,
        }
    }
}

impl Game for TetrisGame {
    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        _delta_time: &std::time::Duration,
    ) -> bool {
        // Input handling
        let new_position = {
            let mut new_position = self.current_figure_position;
            if let Some(input) = input {
                match input.code {
                    crossterm::event::KeyCode::Left => {
                        new_position.x -= 1.0;
                    }
                    crossterm::event::KeyCode::Right => {
                        new_position.x += 1.0;
                    }
                    crossterm::event::KeyCode::Up => {
                        self.current_figure.rotation += std::f32::consts::PI / 2.0;
                    }
                    crossterm::event::KeyCode::Down => {
                        new_position.y += 1.0;
                    }
                    _ => {}
                }
            }
            new_position
        };

        // Check if the figure can be moved to the new position
        let can_move = {
            let mut can_move = true;
            for point in self
                .current_figure
                .applied_rotation_and_position(new_position)
                .iter()
            {
                if point.x < 0.0
                    || point.x >= WIDTH as f32
                    || point.y < 0.0
                    || point.y >= HEIGHT as f32
                    || self.board[point.y as usize][point.x as usize]
                {
                    can_move = false;
                }
            }
            can_move
        };

        // Move the figure if possible
        if can_move {
            self.current_figure_position = new_position;
        }

        true
    }

    fn draw(
        &self,
        out: &mut std::io::Stdout,
        _delta_time: &std::time::Duration,
    ) -> crossterm::Result<()> {
        use crossterm::{cursor::MoveTo, execute};
        use std::io::Write;

        const BOARDER_WIDTH: u16 = 2;

        // Draw the board
        {
            // Draw cells
            {
                for (y, row) in self.board.iter().enumerate() {
                    execute!(out, MoveTo(0, y as u16))?;
                    write!(out, "║ ")?;
                    for &cell in row.iter() {
                        if cell {
                            write!(out, "██")?;
                        } else {
                            write!(out, "  ")?;
                        }
                    }
                    write!(out, " ║")?;
                }
            }
            // Draw border
            {
                execute!(out, MoveTo(0, HEIGHT as u16))?;
                write!(out, "╚═")?;
                for _ in 0..WIDTH {
                    write!(out, "══")?;
                }
                write!(out, "═╝")?;
            }
        }

        // Draw the current figure
        {
            for point in self
                .current_figure
                .applied_rotation_and_position(self.current_figure_position)
                .iter()
            {
                execute!(
                    out,
                    MoveTo(BOARDER_WIDTH + point.x as u16 * 2, point.y as u16)
                )?;
                write!(out, "██")?;
            }
        }

        out.flush()
    }

    fn get_score(&self) -> u32 {
        self.score
    }
}
