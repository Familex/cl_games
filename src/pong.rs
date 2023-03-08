use crate::game::{Game, Score, UpdateEvent};
use crossterm::{cursor::MoveTo, event::KeyEvent, execute, style::Print, terminal};
use rand::Rng;

mod planks {
    pub const FROM_BOUNDS_INDENT: u16 = 5;
    pub const DEFAULT_LENGTH: u16 = 5;
    pub const SPEED: f32 = 2.0;
}
mod ball {
    use super::Point;
    pub const MAX_INITIAL_SPEED: Point = Point { x: 10.0, y: 10.0 };
    pub const MIN_INITIAL_SPEED: Point = Point { x: 5.0, y: 5.0 };
}
const VELOCITY_X_SCALE: f32 = 3.0;
const VELOCITY_Y_SCALE: f32 = 1.1;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Point {
    x: f32,
    y: f32,
}

pub struct Plank {
    position: Point,
    length: u16,
}

impl Plank {
    fn new(w: u16, y: u16) -> Self {
        Self {
            position: Point {
                x: w as f32 / 2.0 / 2.0,
                y: y as f32,
            },
            length: planks::DEFAULT_LENGTH,
        }
    }

    fn draw(&self, out: &mut std::io::Stdout) -> crossterm::Result<()> {
        let bx = self.position.x.round() as u16 * 2 - self.length;

        for dx in 0..self.length {
            execute!(
                out,
                MoveTo(bx + dx * 2, self.position.y.round() as u16),
                Print("==")
            )?;
        }

        Ok(())
    }
}

pub struct Ball {
    position: Point,
    velocity: Point,
}

impl Ball {
    fn new(w: u16, h: u16) -> Self {
        let mut rng = rand::thread_rng();
        let mut velocity = Point {
            x: rng.gen::<i32>() as f32 % ball::MAX_INITIAL_SPEED.x,
            y: rng.gen::<i32>() as f32 % ball::MAX_INITIAL_SPEED.y,
        };
        // Make sure that ball will move
        if velocity.y.abs() < ball::MIN_INITIAL_SPEED.y {
            velocity.y = ball::MIN_INITIAL_SPEED.y * velocity.y.signum();
        }
        if velocity.x.abs() < ball::MIN_INITIAL_SPEED.x {
            velocity.x = ball::MIN_INITIAL_SPEED.x * velocity.x.signum();
        }

        Self {
            position: Point {
                x: w as f32 / 2.0 / 2.0,
                y: h as f32 / 2.0,
            },
            velocity,
        }
    }
}

pub struct PongGame {
    enemy: Plank,
    player: Plank,
    ball: Ball,
    score: i64,
}

impl PongGame {
    pub fn new() -> Self {
        let (width, height) = terminal::size().expect("Failed to get terminal size");

        Self {
            enemy: Plank::new(width, planks::FROM_BOUNDS_INDENT),
            player: Plank::new(width, height - planks::FROM_BOUNDS_INDENT - 1),
            ball: Ball::new(width, height),
            score: 0,
        }
    }

    fn reset_positions(&mut self) {
        let (width, height) = terminal::size().expect("Failed to get terminal size");

        self.enemy = Plank::new(width, planks::FROM_BOUNDS_INDENT);
        self.player = Plank::new(width, height - planks::FROM_BOUNDS_INDENT - 1);
        self.ball = Ball::new(width, height);
    }
}

impl Default for PongGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for PongGame {
    fn get_score(&self) -> Score {
        Score { value: self.score }
    }

    fn draw(
        &self,
        out: &mut std::io::Stdout,
        _delta_time: &std::time::Duration,
    ) -> crossterm::Result<()> {
        use crossterm::style::Stylize;
        use std::io::Write;

        let (width, height) = terminal::size()?;

        // draw planks
        {
            self.player.draw(out)?;
            self.enemy.draw(out)?;
        }

        // draw ball
        {
            execute!(
                out,
                MoveTo(
                    self.ball.position.x.round() as u16 * 2,
                    self.ball.position.y.round() as u16
                ),
                Print("()")
            )?;
        }

        // score
        {
            fn digits_num(num: i64) -> u16 {
                let num = num.abs();
                if num == 0 {
                    1
                } else {
                    f32::floor(f32::log10(num as f32) + 1.0) as u16
                }
            }

            let score_hint = "Score: ";
            let score = format!("{}", self.score);
            execute!(
                out,
                MoveTo(
                    width
                        - score_hint.len() as u16
                        - digits_num(self.score)
                        - (self.score < 0) as u16,
                    height / 2
                ),
            )?;
            write!(
                out,
                "{}{}",
                score_hint,
                if self.score < 0 {
                    score.red()
                } else {
                    score.green()
                }
            )?;
        }

        execute!(out, MoveTo(0, 0))
    }

    fn update(
        &mut self,
        input: &Option<KeyEvent>,
        delta_time: &std::time::Duration,
    ) -> UpdateEvent {
        enum OutOfBoard {
            OnEnemySize,
            OnPlayerSize,
        }

        let (width, height) = terminal::size().expect("Failed to get terminal size");

        // quit on Esc
        if let Some(key) = input {
            if key.code == crossterm::event::KeyCode::Esc {
                return UpdateEvent::GameOver;
            }
        }

        // player input
        // modifies self.player
        {
            let mut next_position = self.player.position.clone();

            if let Some(key) = input {
                match key.code {
                    crossterm::event::KeyCode::Left => {
                        next_position.x -= planks::SPEED;
                    }
                    crossterm::event::KeyCode::Right => {
                        next_position.x += planks::SPEED;
                    }
                    _ => {}
                }
            }

            if next_position.x - self.player.length as f32 / 2.0 > -1.0
                && next_position.x + self.player.length as f32 / 2.0 < width as f32 / 2.0
            {
                self.player.position = next_position;
            }
        }

        // enemy input
        // modifies self.enemy
        {
            let mut next_position = self.enemy.position.clone();

            if self.ball.position.x < next_position.x {
                next_position.x -= planks::SPEED;
            } else if self.ball.position.x > next_position.x {
                next_position.x += planks::SPEED;
            }

            if next_position.x - self.enemy.length as f32 / 2.0 > -1.0
                && next_position.x + self.enemy.length as f32 / 2.0 < width as f32 / 2.0
            {
                self.enemy.position = next_position;
            }
        }

        // ball
        // modifies self.ball
        let out_of_board: Option<OutOfBoard> = {
            let mut next_position = self.ball.position.clone();
            let mut next_velocity = self.ball.velocity.clone();

            if next_position.x < 0.0 || next_position.x > width as f32 / 2.0 {
                next_velocity.x *= -1.0;
            }

            let out_of_board = if next_position.y < -1.0 {
                Some(OutOfBoard::OnEnemySize)
            } else if next_position.y > height as f32 {
                Some(OutOfBoard::OnPlayerSize)
            } else {
                None
            };

            // enemy/player collision
            {
                let plank = if next_velocity.y < 0.0 {
                    &self.enemy
                } else {
                    &self.player
                };

                if next_position.y > plank.position.y - 1.0
                    && next_position.y < plank.position.y + 1.0
                    && next_position.x > plank.position.x - plank.length as f32 / 2.0
                    && next_position.x < plank.position.x + plank.length as f32 / 2.0
                {
                    next_velocity.y *= -1.0;
                    // velocity.x change depends on ball position relative to plank
                    next_velocity.x += (next_position.x - plank.position.x) * VELOCITY_X_SCALE;
                    next_velocity.y *= VELOCITY_Y_SCALE;
                }
            }

            next_position.x += next_velocity.x * delta_time.as_secs_f32();
            next_position.y += next_velocity.y * delta_time.as_secs_f32();

            self.ball.position = next_position;
            self.ball.velocity = next_velocity;

            out_of_board
        };

        // check collision
        // modifies self.score, self.ball, self.enemy, self.player
        if let Some(out_of_board) = out_of_board {
            match out_of_board {
                OutOfBoard::OnEnemySize => {
                    self.score += 1;
                    self.reset_positions()
                }
                OutOfBoard::OnPlayerSize => {
                    self.score -= 1;
                    self.reset_positions()
                }
            }
        }

        UpdateEvent::GameContinue
    }
}
