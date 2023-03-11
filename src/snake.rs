use crate::game;
use crate::game::{Game, UpdateEvent};
use crate::point::{GameBasis, Point, ScreenBasis};
use crossterm::{cursor, execute, style::Stylize, terminal};

const APPLES_MAX: usize = 5;
const APPLES_SPAWN_RATE: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Clone, Copy, Debug)]
pub struct Apple(Point<GameBasis>);

pub struct Score(usize);

impl std::ops::AddAssign<i32> for Score {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs as usize;
    }
}

pub struct Snake {
    pub head: Point<GameBasis>,
    pub tail: Vec<Point<GameBasis>>,
}

impl Snake {
    pub fn new(start: Point<GameBasis>) -> Self {
        Self {
            head: start,
            tail: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    pub fn empty(&self) -> bool {
        !self.up && !self.down && !self.left && !self.right
    }
}

/// Read the input from the given input stream.
fn read_to_input(event: &Option<crossterm::event::KeyEvent>) -> Input {
    use crossterm::event::KeyCode;

    let mut input = Input::new();

    // Handle pressed keys
    if let Some(key_event) = event {
        match key_event.code {
            KeyCode::Up => input.up = true,
            KeyCode::Down => input.down = true,
            KeyCode::Left => input.left = true,
            KeyCode::Right => input.right = true,
            _ => {}
        }
    }

    input
}

pub struct SnakeGame {
    pub snake: Snake,
    pub apples: Vec<Apple>,
    pub prev_non_empty_input: Input,
    pub duration: std::time::Duration,
    pub score: Score,
}

impl Game for SnakeGame {
    /// Move the snake in the direction of the last non-empty input.
    /// If the snake hits the edge of the screen, it wraps around to the other side.
    ///
    /// Returns true if the snake ate an apple.
    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        delta_time: &std::time::Duration,
    ) -> UpdateEvent {
        /// Get the terminal size in rectangular characters
        fn get_terminal_size() -> Point<GameBasis> {
            let size = terminal::size().expect("Failed to get terminal size");
            Point::new(size.0 as f32 / 2.0, size.1 as f32)
        }

        self.duration += *delta_time;

        // Check for collisions
        let is_collided = {
            let mut is_collided = false;
            for point in self.snake.tail.iter() {
                if self.snake.head.compare(point, 0.5) {
                    is_collided = true;
                }
            }
            is_collided
        };

        // Check for eating food
        // Modifies self.apples and self.score
        let is_apple_eaten = {
            let mut i = 0;
            let mut is_apple_eaten = false;
            while i < self.apples.len() {
                if self.snake.head.compare(&self.apples[i].0, 0.5) {
                    is_apple_eaten = true;
                    self.score += 1;
                    self.apples.remove(i);
                } else {
                    i += 1;
                }
            }
            is_apple_eaten
        };

        // Spawn food
        // Zeroes duration if food is spawned
        if self.duration > APPLES_SPAWN_RATE {
            if self.apples.len() < APPLES_MAX {
                /// Check if the given coordinates are on the snake
                fn is_on_snake(snake: &Snake, coords: Point<GameBasis>) -> bool {
                    if coords.compare(&snake.head, 0.5) {
                        return true;
                    }
                    for point in snake.tail.iter() {
                        if coords.compare(point, 0.5) {
                            return true;
                        }
                    }
                    false
                }

                /// Check if the given coordinates are on an apple
                fn is_on_apple(coords: Point<GameBasis>, apples: &[Apple]) -> bool {
                    for apple in apples.iter() {
                        if coords.compare(&apple.0, 0.5) {
                            return true;
                        }
                    }
                    false
                }

                /// Get a random position on the screen (scoreboard excluded)
                fn random_position_on_screen() -> Point<GameBasis> {
                    let screen_size = get_terminal_size();
                    Point::new(
                        (rand::random::<u32>() % (screen_size.x as u32)) as f32,
                        ((rand::random::<u32>() + 1) % (screen_size.y as u32)) as f32,
                    )
                }

                let mut apple_coords = random_position_on_screen();
                while is_on_snake(&self.snake, apple_coords)
                    || is_on_apple(apple_coords, &self.apples)
                {
                    apple_coords = random_position_on_screen();
                }
                self.apples.push(Apple(apple_coords));
            }

            self.duration = std::time::Duration::from_secs(0);
        }

        // Move snake
        // Depends on is_apple_eaten
        // Modifies self.snake and self.prev_non_empty_input
        {
            let screen_size = get_terminal_size();
            let input = read_to_input(input);

            // Move the tail
            self.snake.tail.push(self.snake.head);
            // Don't grow the tail if an apple wasn't eaten
            if !is_apple_eaten {
                self.snake.tail.remove(0);
            }

            let curr_input = if !input.empty()
                && (input.up && !self.prev_non_empty_input.down
                    || input.down && !self.prev_non_empty_input.up
                    || input.left && !self.prev_non_empty_input.right
                    || input.right && !self.prev_non_empty_input.left)
            {
                input
            } else {
                self.prev_non_empty_input
            };

            // Calculate deltas
            let deltas = {
                let mut deltas = Point::<GameBasis>::new(0.0, 0.0);
                if curr_input.up {
                    deltas.y -= 1.0;
                }
                if curr_input.down {
                    deltas.y += 1.0;
                }
                if curr_input.left {
                    deltas.x -= 1.0;
                }
                if curr_input.right {
                    deltas.x += 1.0;
                }
                deltas
            };

            // Apply deltas
            {
                self.snake.head.x += deltas.x;
                self.snake.head.y += deltas.y;

                if self.snake.head.x < 0.0 {
                    self.snake.head.x = screen_size.x - 1.0;
                }
                if self.snake.head.x >= screen_size.x {
                    self.snake.head.x = 0.0;
                }
                if self.snake.head.y < 0.0 {
                    self.snake.head.y = screen_size.y - 1.0;
                }
                if self.snake.head.y >= screen_size.y {
                    self.snake.head.y = 0.0;
                }
            }

            self.prev_non_empty_input = curr_input;
        };

        if is_collided {
            UpdateEvent::GameOver
        } else {
            UpdateEvent::GameContinue
        }
    }

    /// Draw the snake to the screen.
    fn draw(
        &self,
        out: &mut std::io::Stdout,
        _delta_time: &std::time::Duration,
    ) -> crossterm::Result<()> {
        use cursor::MoveTo;
        use std::io::Write;
        use terminal::size;

        let (max_x, _max_y) = size().expect("Failed to get terminal size");

        // Draw snake
        {
            let snake_head_on_screen: Point<ScreenBasis> = self.snake.head.into();

            for point in self
                .snake
                .tail
                .iter()
                .map(|p| Point::<ScreenBasis>::from(*p))
            {
                execute!(out, MoveTo(point.x.round() as u16, point.y.round() as u16))?;
                write!(out, "++")?;
            }
            execute!(
                out,
                MoveTo(
                    snake_head_on_screen.x.round() as u16,
                    snake_head_on_screen.y.round() as u16
                )
            )?;
            write!(out, "{}", "||".green())?;
        }

        // Draw apples
        {
            for apple in self.apples.iter().map(|p| Point::<ScreenBasis>::from(p.0)) {
                execute!(out, MoveTo(apple.x.round() as u16, apple.y.round() as u16))?;
                write!(out, "{}", "<>".red())?;
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
                    (max_x - score_hint.len() as u16 - digits_num(self.score.0)) / 2,
                    0
                )
            )?;
            let score = format!("{}", self.score.0);
            write!(
                out,
                "Score: {}",
                if self.score.0 < 10 {
                    score.white()
                } else if self.score.0 < 40 {
                    score.green()
                } else if self.score.0 < 100 {
                    score.yellow()
                } else {
                    score.red()
                }
            )?;
        }

        // Reset cursor
        execute!(out, MoveTo(0, 0))
    }

    fn get_score(&self) -> game::Score {
        game::Score {
            value: self.score.0 as i64,
        }
    }
}

impl SnakeGame {
    /// Create a new game instance with the given settings.
    /// Snake starts at the given point and moves right.
    /// Tail is 2 points long.
    pub fn new(setup: Point<GameBasis>) -> Self {
        Self {
            snake: Snake {
                head: setup,
                tail: vec![
                    Point::new(setup.x - 1.0, setup.y),
                    Point::new(setup.x - 2.0, setup.y),
                    Point::new(setup.x - 3.0, setup.y),
                ],
            },
            apples: Vec::new(),
            duration: std::time::Duration::from_millis(2),
            prev_non_empty_input: Input {
                up: false,
                down: false,
                left: false,
                right: true, // Start moving right
            },
            score: Score(0),
        }
    }
}
