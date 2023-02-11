use crate::game::*;
use colored::Colorize;
use crossterm::{cursor, execute, terminal};

const APPLES_MAX: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Apple(Point);

pub struct Snake {
    pub head: Point,
    pub tail: Vec<Point>,
}

impl Snake {
    pub fn new(start: Point) -> Self {
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

pub struct SnakeGame {
    pub snake: Snake,
    pub apples: Vec<Apple>,
    pub prev_non_empty_input: Input,
    pub duration: std::time::Duration,
}

impl Game for SnakeGame {
    type Settings = Point;
    type Input = Input;
    // None if the game is over, otherwise [`is_apple_eaten`]
    type Events = Option<bool>;

    /// Create a new game instance with the given settings.
    /// Snake starts at the given point and moves right.
    /// Tail is 2 points long.
    fn new(setup: Self::Settings) -> Self {
        Self {
            snake: Snake {
                head: setup,
                tail: vec![
                    Point {
                        x: setup.x - 1,
                        y: setup.y,
                    },
                    Point {
                        x: setup.x - 2,
                        y: setup.y,
                    },
                    Point {
                        x: setup.x - 3,
                        y: setup.y,
                    },
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
        }
    }

    /// Move the snake in the direction of the last non-empty input.
    /// If the snake hits the edge of the screen, it wraps around to the other side.
    ///
    /// Returns true if the snake ate an apple.
    fn update(&mut self, input: &Self::Input, delta_time: &std::time::Duration) -> Self::Events {
        fn get_terminal_size() -> (i32, i32) {
            let size = terminal::size().expect("Failed to get terminal size");
            (size.0 as i32 / 2, size.1 as i32)
        }

        self.duration += *delta_time;

        // Check for collisions
        let is_collided = {
            let mut is_collided = false;
            for point in self.snake.tail.iter() {
                if self.snake.head == *point {
                    is_collided = true;
                }
            }
            is_collided
        };

        // Check for eating food
        // Modifies self.apples
        let is_apple_eaten = {
            let mut i = 0;
            let mut is_apple_eaten = false;
            while i < self.apples.len() {
                if self.snake.head == self.apples[i].0 {
                    is_apple_eaten = true;
                    self.apples.remove(i);
                } else {
                    i += 1;
                }
            }
            is_apple_eaten
        };

        // Spawn food
        // Zeroes duration if food is spawned
        if self.duration > std::time::Duration::from_secs(3) {
            let (max_x, max_y) = get_terminal_size();

            if self.apples.len() < APPLES_MAX {
                self.apples.push(Apple(Point {
                    x: (rand::random::<u32>() % (max_x as u32)) as i32,
                    y: (rand::random::<u32>() % (max_y as u32)) as i32,
                }));
            }

            self.duration = std::time::Duration::from_millis(0);
        }

        // Move snake
        // Depends on is_apple_eaten
        // Modifies self.snake and self.prev_non_empty_input
        {
            let (max_x, max_y) = get_terminal_size();

            // Move the tail
            self.snake.tail.push(self.snake.head.clone());
            // Don't grow the tail if an apple wasn't eaten
            if !is_apple_eaten {
                self.snake.tail.remove(0);
            }

            let curr_input = if !input.empty()
                && ((*input).up && !self.prev_non_empty_input.down
                    || (*input).down && !self.prev_non_empty_input.up
                    || (*input).left && !self.prev_non_empty_input.right
                    || (*input).right && !self.prev_non_empty_input.left)
            {
                *input
            } else {
                self.prev_non_empty_input
            };

            // Calculate deltas
            let deltas = {
                let mut deltas = Point { x: 0, y: 0 };
                if curr_input.up {
                    deltas.y -= 1;
                }
                if curr_input.down {
                    deltas.y += 1;
                }
                if curr_input.left {
                    deltas.x -= 1;
                }
                if curr_input.right {
                    deltas.x += 1;
                }
                deltas
            };

            // Apply deltas
            {
                self.snake.head.x += deltas.x;
                self.snake.head.y += deltas.y;

                if self.snake.head.x < 0 {
                    self.snake.head.x = max_x - 1;
                }
                if self.snake.head.x >= max_x {
                    self.snake.head.x = 0;
                }
                if self.snake.head.y < 0 {
                    self.snake.head.y = max_y - 1;
                }
                if self.snake.head.y >= max_y {
                    self.snake.head.y = 0;
                }
            }

            self.prev_non_empty_input = curr_input;
        };

        if is_collided {
            None
        } else {
            Some(is_apple_eaten)
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

        // Draw snake
        {
            for point in self.snake.tail.iter() {
                execute!(out, MoveTo(point.x as u16 * 2, point.y as u16))?;
                write!(out, "{}", "++")?;
            }
            execute!(
                out,
                MoveTo(self.snake.head.x as u16 * 2, self.snake.head.y as u16)
            )?;
            write!(out, "{}", "||".green())?;
        }

        // Draw apples
        {
            for apple in self.apples.iter() {
                execute!(out, MoveTo(apple.0.x as u16 * 2, apple.0.y as u16))?;
                write!(out, "{}", "<>".red())?;
            }
        }

        // Reset cursor
        execute!(out, MoveTo(0, 0))
    }

    fn read_to_input(&self, event: &Option<crossterm::event::KeyEvent>) -> Self::Input {
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
}
