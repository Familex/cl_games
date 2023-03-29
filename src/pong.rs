use crate::game::{Game, Score, UpdateEvent, EXIT_BUTTON};
use crate::point::{BoundsCollision, GameBasis, Line, Point, ScreenBasis};
use crossterm::{cursor::MoveTo, event::KeyEvent, execute, style::Print, terminal};
use rand::Rng;

mod planks {
    pub const FROM_BOUNDS_INDENT: u16 = 5;
    pub const DEFAULT_LENGTH: u16 = 5;
    pub const PLAYER_SPEED: f32 = 2.0;
    pub const ENEMY_SPEED: f32 = 25.0;
    pub const COLLISION_EXTRA_LENGTH: f32 = 1.0;
}
mod ball {
    use crate::point::{GameBasis, Point};
    pub const MAX_INITIAL_SPEED: Point<GameBasis> = Point::new(10.0, 10.0);
    pub const MIN_INITIAL_SPEED: Point<GameBasis> = Point::new(5.0, 5.0);
}
const VELOCITY_X_SCALE: f32 = 3.0;
const VELOCITY_Y_SCALE: f32 = 1.1;

#[derive(Debug)]
pub struct Plank {
    position: Point<GameBasis>,
    length: u16,
}

impl Plank {
    fn new(w: u16, y: u16) -> Self {
        Self {
            position: Point::new(w as f32 / 2.0 / 2.0, y as f32),
            length: planks::DEFAULT_LENGTH,
        }
    }

    fn draw(&self, out: &mut std::io::Stdout) -> crossterm::Result<()> {
        let screen_pos = Point::<ScreenBasis>::from(self.position);
        let bx = (screen_pos.x.round() - self.length as f32) as u16;

        for dx in (0..self.length).map(|x| x * 2) {
            execute!(
                out,
                MoveTo(bx + dx, screen_pos.y.round() as u16),
                Print("==")
            )?;
        }

        Ok(())
    }

    fn bounds_check(&self, w: u16, next_position: Option<Point<GameBasis>>) -> bool {
        next_position.unwrap_or(self.position).x - self.length as f32 / 2.0 > 0.0
            && next_position.unwrap_or(self.position).x + self.length as f32 / 2.0 < w as f32 / 2.0
    }
}

pub struct Ball {
    position: Point<GameBasis>,
    velocity: Point<GameBasis>,
}

impl Ball {
    fn new(w: u16, h: u16) -> Self {
        let mut rng = rand::thread_rng();
        let mut velocity = Point::<GameBasis>::new(
            rng.gen::<i32>() as f32 % ball::MAX_INITIAL_SPEED.x,
            rng.gen::<i32>() as f32 % ball::MAX_INITIAL_SPEED.y,
        );
        // Make sure that ball will move
        if velocity.y.abs() < ball::MIN_INITIAL_SPEED.y {
            velocity.y = ball::MIN_INITIAL_SPEED.y * velocity.y.signum();
        }
        if velocity.x.abs() < ball::MIN_INITIAL_SPEED.x {
            velocity.x = ball::MIN_INITIAL_SPEED.x * velocity.x.signum();
        }

        Self {
            position: Point::<ScreenBasis>::new(w as f32 / 2.0, h as f32 / 2.0).into(),
            velocity,
        }
    }
}

/// Ball moves from `prev_ball_pos` to `ball_pos`
/// Returns true if ball collides with plank on its way
fn collides(
    plank_pos: &Point<GameBasis>,
    plank_length: f32,
    prev_ball_pos: &Point<GameBasis>,
    ball_pos: &Point<GameBasis>,
) -> bool {
    let plank = Line::new(
        Point::new(
            plank_pos.x - plank_length / 2.0 - planks::COLLISION_EXTRA_LENGTH,
            plank_pos.y,
        ),
        Point::new(
            plank_pos.x + plank_length / 2.0 + planks::COLLISION_EXTRA_LENGTH,
            plank_pos.y,
        ),
    );
    let ball = Line::new(*prev_ball_pos, *ball_pos);

    plank.intersects(&ball)
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

        // self.enemy = Plank::new(width, planks::FROM_BOUNDS_INDENT);
        // self.player = Plank::new(width, height - planks::FROM_BOUNDS_INDENT - 1);
        self.ball = Ball::new(width, height);
    }
}

impl Default for PongGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for PongGame {
    fn update(
        &mut self,
        input: &Option<KeyEvent>,
        delta_time: &std::time::Duration,
    ) -> UpdateEvent {
        enum OutOfBoard {
            OnEnemySide,
            OnPlayerSide,
        }

        let (width, height) = terminal::size().expect("Failed to get terminal size");

        // quit
        if let Some(key) = input {
            if key.code == EXIT_BUTTON {
                return UpdateEvent::GameOver;
            }
        }

        // player input
        // modifies self.player
        {
            let prev_position = self.player.position;

            if let Some(key) = input {
                match key.code {
                    crossterm::event::KeyCode::Left => {
                        self.player.position.x -= planks::PLAYER_SPEED;
                    }
                    crossterm::event::KeyCode::Right => {
                        self.player.position.x += planks::PLAYER_SPEED;
                    }
                    _ => {}
                }
            }

            if !self.player.bounds_check(width, None) {
                self.player.position = prev_position;
            }
        }

        // enemy input
        // modifies self.enemy
        {
            let prev_position = self.enemy.position;

            if self.ball.position.x < self.enemy.position.x {
                self.enemy.position.x -= planks::ENEMY_SPEED * delta_time.as_secs_f32();
            } else if self.ball.position.x > self.enemy.position.x {
                self.enemy.position.x += planks::ENEMY_SPEED * delta_time.as_secs_f32();
            }

            if !self.enemy.bounds_check(width, None) {
                self.enemy.position = prev_position;
            }
        }

        // ball
        // modifies self.ball
        let out_of_board: Option<OutOfBoard> = {
            let prev_position = self.ball.position;
            let mut out_of_board = None;

            self.ball.position.x += self.ball.velocity.x * delta_time.as_secs_f32();
            self.ball.position.y += self.ball.velocity.y * delta_time.as_secs_f32();

            match self.ball.position.bounds_check(width, height) {
                Some(BoundsCollision::Left | BoundsCollision::Right) => {
                    self.ball.velocity.x *= -1.0;
                    self.ball.position.x = prev_position.x;
                }
                Some(BoundsCollision::Top) => {
                    out_of_board = Some(OutOfBoard::OnEnemySide);
                }
                Some(BoundsCollision::Bottom) => {
                    out_of_board = Some(OutOfBoard::OnPlayerSide);
                }
                None => {}
            }

            // enemy/player collision
            {
                let plank = if self.ball.velocity.y < 0.0 {
                    &self.enemy
                } else {
                    &self.player
                };

                if collides(
                    &plank.position,
                    plank.length as f32,
                    &prev_position,
                    &self.ball.position,
                ) {
                    self.ball.velocity.y *= -1.0;
                    // velocity.x change depends on ball position relative to plank
                    self.ball.velocity.x +=
                        (self.ball.position.x - plank.position.x) * VELOCITY_X_SCALE;
                    self.ball.velocity.y *= VELOCITY_Y_SCALE;
                }
            }

            self.ball.position = prev_position;
            self.ball.position.x += self.ball.velocity.x * delta_time.as_secs_f32();
            self.ball.position.y += self.ball.velocity.y * delta_time.as_secs_f32();

            out_of_board
        };

        // check collision
        // modifies self.score, self.ball, self.enemy, self.player
        if let Some(out_of_board) = out_of_board {
            match out_of_board {
                OutOfBoard::OnEnemySide => {
                    self.score += 1;
                    self.reset_positions();
                }
                OutOfBoard::OnPlayerSide => {
                    self.score -= 1;
                    self.reset_positions();
                }
            }
        }

        UpdateEvent::GameContinue
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
            let screen_pos = Point::<ScreenBasis>::from(self.ball.position);

            execute!(
                out,
                MoveTo(screen_pos.x.round() as u16, screen_pos.y.round() as u16),
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

    fn get_score(&self) -> Score {
        Score { value: self.score }
    }
}
