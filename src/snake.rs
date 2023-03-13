use crate::game;
use crate::game::{Game, UpdateEvent};
use crate::point::{BoundsCollision, GameBasis, Line, Point, ScreenBasis};
use crate::util::MORE_THAN_HALF_CELL;
use crossterm::{cursor, execute, style::Stylize, terminal};

mod apples {
    use crate::util::MORE_THAN_HALF_CELL;
    pub const MAX: usize = 5;
    pub const SPAWN_RATE: std::time::Duration = std::time::Duration::from_secs(2);
    pub const RADIUS: f32 = MORE_THAN_HALF_CELL;
    pub const GROWTH: f32 = 1.0;
}
mod snakes {
    pub(crate) const SPEED: f32 = 12.0;

    pub(crate) const WIDTH: f32 = 0.25;
}

#[derive(Clone, Copy, Debug)]
pub struct Apple(Point<GameBasis>);

pub struct Score(usize);

impl std::ops::AddAssign<i32> for Score {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs as usize;
    }
}

pub struct Snake {
    pub segments: Vec<Line<GameBasis>>,
}

impl Snake {
    pub fn new(begin: Point<GameBasis>) -> Self {
        Self {
            segments: vec![Line {
                begin,
                end: begin + Point::new(3.0, 0.0),
            }],
        }
    }

    pub fn head(&self) -> &Line<GameBasis> {
        self.segments.last().unwrap()
    }

    pub fn mut_head(&mut self) -> &mut Line<GameBasis> {
        self.segments.last_mut().unwrap()
    }

    pub fn first(&self) -> &Line<GameBasis> {
        &self.segments[0]
    }

    pub fn mut_first(&mut self) -> &mut Line<GameBasis> {
        &mut self.segments[0]
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

    pub fn as_vec(&self, length: f32) -> Point<GameBasis> {
        let mut vec = Point::new(0.0, 0.0);

        if self.up {
            vec.y -= length;
        }
        if self.down {
            vec.y += length;
        }
        if self.left {
            vec.x -= length;
        }
        if self.right {
            vec.x += length;
        }

        vec
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

impl SnakeGame {
    /// Create a new game instance with the given settings.
    /// Snake starts at the given point and moves right.
    /// Tail is 2 points long.
    pub fn new(setup: Point<GameBasis>) -> Self {
        Self {
            snake: Snake::new(setup),
            apples: Vec::new(),
            duration: std::time::Duration::from_millis(2),
            prev_non_empty_input: Input {
                up: false,
                down: false,
                left: false,
                right: true, // Start moving right
            },
            score: Score(0),
            to_growth: 0.0,
        }
    }
}

pub struct SnakeGame {
    pub snake: Snake,
    pub apples: Vec<Apple>,
    pub prev_non_empty_input: Input,
    pub duration: std::time::Duration,
    pub score: Score,
    pub to_growth: f32,
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
        let is_collided = if self.snake.segments.len() > 2 {
            let mut is_collided = false;
            for segment_ind in 0..self.snake.segments.len() - 2
            /* last two segments is head and pre-head */
            {
                let segment = &self.snake.segments[segment_ind];
                if self.snake.head().intersects(segment)
                    || segment.distance_to(&self.snake.head().end) < snakes::WIDTH
                {
                    is_collided = true;
                }
            }
            is_collided
        } else {
            false
        };

        // Check for eating food
        // Modifies self.apples, self.score and self.to_growth
        {
            let mut i = 0;
            while i < self.apples.len() {
                if self
                    .snake
                    .head()
                    .end
                    .compare(&self.apples[i].0, apples::RADIUS)
                {
                    self.to_growth += apples::GROWTH;
                    self.score += 1;
                    self.apples.remove(i);
                } else {
                    i += 1;
                }
            }
        };

        // Spawn food
        // Zeroes duration if food is spawned
        if self.duration > apples::SPAWN_RATE {
            if self.apples.len() < apples::MAX {
                /// Check if the given coordinates are on the snake
                fn is_on_snake(snake: &Snake, coords: Point<GameBasis>) -> bool {
                    for segment in snake.segments.iter() {
                        if coords.compare(&segment.end, apples::RADIUS) {
                            return true;
                        }
                    }
                    false
                }

                /// Check if the given coordinates are on an apple
                fn is_on_apple(coords: Point<GameBasis>, apples: &[Apple]) -> bool {
                    for apple in apples.iter() {
                        if coords.compare(&apple.0, apples::RADIUS) {
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
            let real_screen_size: Point<ScreenBasis> = screen_size.into();
            let input = read_to_input(input);
            let distance_traveled = snakes::SPEED * delta_time.as_secs_f32();

            let input = if !input.empty()
                && (input.up && !self.prev_non_empty_input.down
                    || input.down && !self.prev_non_empty_input.up
                    || input.left && !self.prev_non_empty_input.right
                    || input.right && !self.prev_non_empty_input.left)
            {
                input
            } else {
                self.prev_non_empty_input
            };

            // Growth head
            // FIXME bound check
            if input != self.prev_non_empty_input {
                let new_head_end = input.as_vec(distance_traveled) + self.snake.head().end;
                match new_head_end.bounds_check(
                    real_screen_size.x.round() as u16,
                    real_screen_size.y.round() as u16,
                ) {
                    None => self
                        .snake
                        .segments
                        .push(Line::new(self.snake.head().end, new_head_end)),
                    Some(BoundsCollision::Bottom) => self.snake.segments.push({
                        let begin = Point::new(self.snake.head().end.x, 0.0);
                        Line::new(begin, begin + new_head_end)
                    }),
                    Some(BoundsCollision::Top) => self.snake.segments.push({
                        let begin = Point::new(self.snake.head().end.x, screen_size.y);
                        Line::new(begin, begin + new_head_end)
                    }),
                    Some(BoundsCollision::Left) => self.snake.segments.push({
                        let begin = Point::new(0.0, self.snake.head().end.y);
                        Line::new(begin, begin + new_head_end)
                    }),
                    Some(BoundsCollision::Right) => self.snake.segments.push({
                        let begin = Point::new(screen_size.x, self.snake.head().end.y);
                        Line::new(begin, begin + new_head_end)
                    }),
                }
            } else {
                self.snake.mut_head().end += input.as_vec(distance_traveled);
            }

            // Shrink tail
            let mut to_shrink = 0.0_f32.max(distance_traveled - self.to_growth);
            self.to_growth = 0.0_f32.max(self.to_growth - distance_traveled);
            while to_shrink > f32::EPSILON {
                if self.snake.first().length() > to_shrink {
                    let first_dir = self.snake.first().direction() * -1.0;
                    self.snake.mut_first().begin -= first_dir * to_shrink;
                    to_shrink = 0.0;
                } else {
                    to_shrink -= self.snake.first().length();
                    self.snake.segments.remove(0);
                }
            }

            self.prev_non_empty_input = input;
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
            // Draw snake body
            {
                for segment in self.snake.segments.iter() {
                    use once_cell::sync::Lazy;
                    static EPS: Lazy<f32> = Lazy::new(|| 2.0_f32.hypot(1.0_f32));
                    let segment_begin: Point<ScreenBasis> = segment.begin.into();
                    let segment_end: Point<ScreenBasis> = segment.end.into();
                    let segment_direction = (segment_end - segment_begin);

                    // Calculate the unit vector of segment_direction
                    let segment_direction_unit = segment_direction / segment_direction.length();

                    let mut segment_point = segment_begin;
                    let segment_length = segment_direction.length();
                    let mut distance_traveled = 0.0;
                    let angle_to_x_axis = segment_direction_unit.y.atan2(segment_direction_unit.x);
                    let scale_factor = if segment_direction_unit.x >= 0.0 {
                        if angle_to_x_axis.abs() < std::f32::consts::FRAC_PI_4 {
                            2.0
                        } else {
                            1.0
                        }
                    } else {
                        if angle_to_x_axis.abs() > std::f32::consts::FRAC_PI_4 {
                            2.0
                        } else {
                            1.0
                        }
                    };
                    'draw_segment: loop {
                        execute!(
                            out,
                            MoveTo(
                                segment_point.x.round() as u16,
                                segment_point.y.round() as u16
                            )
                        )?;
                        write!(out, "{}", "()".green())?;

                        segment_point += Point::new(
                            segment_direction_unit.x * scale_factor,
                            segment_direction_unit.y * scale_factor,
                        );

                        // Update the distance traveled along the segment
                        distance_traveled += segment_direction_unit.length() * scale_factor;
                        if distance_traveled >= segment_length {
                            break 'draw_segment;
                        }
                    }

                    // Draw the endpoint of the segment if it was not already drawn
                    if segment_point.distance_to(&segment_end) >= *EPS {
                        execute!(
                            out,
                            MoveTo(segment_end.x.round() as u16, segment_end.y.round() as u16)
                        )?;
                        write!(out, "{}", "()".green())?;
                    }
                }
            }

            // Draw snake's head
            {
                let snake_head_on_screen: Point<ScreenBasis> = self.snake.head().end.into();

                execute!(
                    out,
                    MoveTo(
                        snake_head_on_screen.x.round() as u16,
                        snake_head_on_screen.y.round() as u16
                    )
                )?;
                write!(out, "{}", "❮❯".green())?;
            }
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
