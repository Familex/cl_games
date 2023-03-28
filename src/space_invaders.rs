use crate::game::{Game, Score, UpdateEvent};
use crate::point::{GameBasis, Point, ScreenBasis};
use crate::util::MORE_THAN_HALF_CELL;
use rand::Rng;
use std::time::Duration;

const FOR_ENEMY_SCORE: usize = 1;
const FOR_PROP_SCORE: usize = 0;
const FIRE_BULLET_OFFSET: f32 = 1.0;
const PLAYER_SPEED: f32 = 1.0;
const PLAYER_FIRE_RATE: Duration = Duration::from_millis(500);
const GAME_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub fn is_success(chance: f32) -> bool {
    let mut rng = rand::thread_rng();
    let random: f32 = rng.gen();
    random < chance / 100.0
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Bullet {
    move_direction: Direction,
    position: Point<GameBasis>,
    speed: f32,
}

#[derive(Clone, Debug)]
pub enum EnemyActionType {
    Move(Direction, f32),
    Fire(Direction, f32),
    Wait,
}

#[derive(Clone, Debug)]
pub struct EnemyAction {
    action_type: EnemyActionType,
    duration: Duration,
    chance: f32,
}

impl EnemyAction {
    pub fn new(action_type: EnemyActionType, duration: Duration, chance: f32) -> Self {
        assert!((0.0..=100.0).contains(&chance));

        Self {
            action_type,
            duration,
            chance,
        }
    }

    fn move_by_one(direction: Direction, chance: f32) -> Self {
        Self::new(
            EnemyActionType::Move(direction, 1.0),
            Duration::from_secs(1),
            chance,
        )
    }

    pub fn left(chance: f32) -> Self {
        Self::move_by_one(Direction::Left, chance)
    }

    pub fn right(chance: f32) -> Self {
        Self::move_by_one(Direction::Right, chance)
    }

    pub fn up(chance: f32) -> Self {
        Self::move_by_one(Direction::Up, chance)
    }

    pub fn down(chance: f32) -> Self {
        Self::move_by_one(Direction::Down, chance)
    }

    pub fn wait(duration: Duration, chance: f32) -> Self {
        Self::new(EnemyActionType::Wait, duration, chance)
    }

    pub fn fire_down(chance: f32) -> Self {
        Self::new(
            EnemyActionType::Fire(Direction::Down, 1.0),
            Duration::from_secs(1),
            chance,
        )
    }
}

#[derive(Clone, Debug)]
pub struct EnemyBehavior {
    actions: Vec<EnemyAction>,
    to_next_move: Duration,
    current_action: usize,
}

impl EnemyBehavior {
    fn new(actions: Vec<EnemyAction>, to_next_move: Duration, current_action: usize) -> Self {
        assert!(current_action < actions.len());
        assert!(!actions.is_empty());

        Self {
            actions,
            to_next_move,
            current_action,
        }
    }

    fn current_action(&self) -> EnemyAction {
        self.actions[self.current_action].clone()
    }

    fn next_action(&mut self) {
        self.current_action += 1;
        if self.current_action >= self.actions.len() {
            self.current_action = 0;
        }
    }

    /// FIXME rename
    fn delta(&mut self, delta_time: Duration) {
        if self.to_next_move < delta_time {
            self.to_next_move = Duration::from_nanos(0);
        } else {
            self.to_next_move -= delta_time;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    position: Point<GameBasis>,
    behavior: EnemyBehavior,
}

#[derive(Clone, Debug)]
pub struct Prop {
    position: Point<GameBasis>,
    destroyable: bool,
}

pub struct Player {
    position: Point<GameBasis>,
}

pub struct SpaceInvadersGame {
    score: usize,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    props: Vec<Prop>,
    player: Player,
    from_last_update: Duration,
    from_last_fire: Duration,
}

pub enum EnemyPreset {
    Empty,
    CheckeredLeftRight,
    CheckeredRightDownLeftUp,
    CheckeredLeft,
    RandomFire,
}

pub enum PropsPreset {
    Empty,
    Wall,
}

impl SpaceInvadersGame {
    pub fn new(
        screen_height: u16,
        screen_width: u16,
        enemy_preset: EnemyPreset,
        props_preset: PropsPreset,
    ) -> Self {
        Self {
            score: 0,
            bullets: vec![],
            enemies: match enemy_preset {
                EnemyPreset::Empty => vec![],
                EnemyPreset::CheckeredLeftRight => {
                    let mut enemies = vec![];
                    for y in 0..5 {
                        for x in 0..screen_width / 2 / 2 {
                            enemies.push(Enemy {
                                position: Point::new(x as f32 * 2.0 + y as f32 % 2.0, y as f32),
                                behavior: EnemyBehavior::new(
                                    vec![EnemyAction::right(100.0), EnemyAction::left(100.0)],
                                    Duration::from_millis(0),
                                    0,
                                ),
                            });
                        }
                    }
                    enemies
                }
                EnemyPreset::CheckeredRightDownLeftUp => {
                    let mut enemies = vec![];
                    for y in 0..5 {
                        for x in 0..screen_width / 2 / 2 {
                            enemies.push(Enemy {
                                position: Point::new(x as f32 * 2.0 + y as f32 % 2.0, y as f32),
                                behavior: EnemyBehavior::new(
                                    vec![
                                        EnemyAction::right(100.0),
                                        EnemyAction::down(100.0),
                                        EnemyAction::left(100.0),
                                        EnemyAction::up(100.0),
                                    ],
                                    Duration::from_millis(0),
                                    0,
                                ),
                            });
                        }
                    }
                    enemies
                }
                EnemyPreset::CheckeredLeft => {
                    let mut enemies = vec![];
                    for y in 0..5 {
                        for x in 0..screen_width / 2 / 2 {
                            enemies.push(Enemy {
                                position: Point::new(x as f32 * 2.0 + y as f32 % 2.0, y as f32),
                                behavior: EnemyBehavior::new(
                                    vec![EnemyAction::left(100.0)],
                                    Duration::from_millis(0),
                                    0,
                                ),
                            });
                        }
                    }
                    enemies
                }
                EnemyPreset::RandomFire => {
                    let mut enemies = vec![];
                    for y in 0..8 {
                        for x in 0..screen_width / 2 / 7 {
                            enemies.push(Enemy {
                                position: Point::new(
                                    x as f32 * 7.0 + y as f32 + (rand::random::<u8>() % 7) as f32,
                                    y as f32,
                                ),
                                behavior: EnemyBehavior::new(
                                    vec![
                                        EnemyAction::fire_down(10.0),
                                        EnemyAction::left(20.0),
                                        EnemyAction::down(5.0),
                                        EnemyAction::wait(Duration::from_secs(1), 50.0),
                                    ],
                                    Duration::from_millis(0),
                                    0,
                                ),
                            });
                        }
                    }
                    enemies
                }
            },
            props: match props_preset {
                PropsPreset::Empty => vec![],
                PropsPreset::Wall => {
                    let mut props = vec![];
                    for x in 0..screen_width / 2 / 2 {
                        props.push(Prop {
                            position: Point::new(x as f32 * 2.0, screen_height as f32 - 3.0),
                            destroyable: false,
                        });
                    }
                    for x in 0..screen_width / 2 {
                        for y in 0..3 {
                            props.push(Prop {
                                position: Point::new(
                                    x as f32,
                                    screen_height as f32 - 4.0 - y as f32,
                                ),
                                destroyable: true,
                            });
                        }
                    }
                    props
                }
            },
            player: Player {
                position: Point::<ScreenBasis>::new(
                    (screen_width / 2) as f32,
                    screen_height as f32 - 1.0,
                )
                .into(),
            },
            from_last_update: Duration::from_nanos(0),
            from_last_fire: Duration::from_nanos(0),
        }
    }
}

impl Game for SpaceInvadersGame {
    fn get_score(&self) -> Score {
        Score {
            value: self.score as i64,
        }
    }

    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        delta_time: &Duration,
    ) -> UpdateEvent {
        let (screen_width, screen_height) =
            crossterm::terminal::size().expect("Failed to get terminal size");

        // last update time
        {
            self.from_last_update += *delta_time;
        }

        // what not depends on self.last_update_time
        let (quit_requested, is_player_collided) = {
            // quit request
            let quit_requested = matches!(
                input,
                Some(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Char('q'),
                    ..
                })
            );

            // deltas
            {
                // enemies delta
                for enemy in &mut self.enemies {
                    enemy.behavior.delta(*delta_time);
                }

                // player fire delta
                self.from_last_fire += *delta_time;
            }

            // player movement
            // modifies self.player
            {
                let next_position: Option<Point<GameBasis>> = match input {
                    Some(crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Left,
                        ..
                    }) => Some(Point::new(
                        self.player.position.x - PLAYER_SPEED,
                        self.player.position.y,
                    )),
                    Some(crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Right,
                        ..
                    }) => Some(Point::new(
                        self.player.position.x + PLAYER_SPEED,
                        self.player.position.y,
                    )),
                    Some(crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Char(' '),
                        ..
                    }) => {
                        if self.from_last_fire > PLAYER_FIRE_RATE {
                            self.from_last_fire = Duration::from_nanos(0);
                            self.bullets.push(Bullet {
                                move_direction: Direction::Up,
                                position: Point::new(
                                    self.player.position.x,
                                    self.player.position.y - 1.0,
                                ),
                                speed: 1.0,
                            });
                        }
                        None
                    }
                    _ => None,
                };

                if let Some(next_position) = next_position {
                    if next_position
                        .bounds_check(screen_width, screen_height)
                        .is_none()
                        && self
                            .props
                            .iter()
                            .all(|prop| !prop.position.compare(&next_position, MORE_THAN_HALF_CELL))
                        && self.enemies.iter().all(|enemy| {
                            !enemy.position.compare(&next_position, MORE_THAN_HALF_CELL)
                        })
                    {
                        self.player.position = next_position;
                    }
                }
            }

            // player bullet collision
            let is_player_collided = self.bullets.iter().any(|bullet| {
                self.player
                    .position
                    .compare(&bullet.position, MORE_THAN_HALF_CELL)
            });

            (quit_requested, is_player_collided)
        };

        // what depends on self.last_update_time
        if self.from_last_update > GAME_UPDATE_INTERVAL {
            self.from_last_update = Duration::from_nanos(0);

            // enemies movement
            // modifies self.enemies
            {
                let mut new_enemies = self.enemies.clone();

                for new_enemy in &mut new_enemies {
                    let action = new_enemy.behavior.current_action();
                    let behavior = &mut new_enemy.behavior;
                    let start_action_ind = behavior.current_action;

                    if behavior.to_next_move.as_nanos() == 0 {
                        // 'failures is do-while loop
                        'failures: loop {
                            if is_success(action.chance)
                                && match &action.action_type {
                                    EnemyActionType::Move(direction, speed) => {
                                        let next_position: Point<GameBasis> = {
                                            match direction {
                                                Direction::Up => Point::new(
                                                    new_enemy.position.x,
                                                    new_enemy.position.y - speed,
                                                ),
                                                Direction::Down => Point::new(
                                                    new_enemy.position.x,
                                                    new_enemy.position.y + speed,
                                                ),
                                                Direction::Left => Point::new(
                                                    new_enemy.position.x - speed,
                                                    new_enemy.position.y,
                                                ),
                                                Direction::Right => Point::new(
                                                    new_enemy.position.x + speed,
                                                    new_enemy.position.y,
                                                ),
                                            }
                                        };
                                        if next_position
                                        .bounds_check(screen_width, screen_height)
                                        .is_none()
                                        && self.enemies.iter().all(
                                            |other| {
                                                !other
                                                    .position
                                                    .compare(&next_position, MORE_THAN_HALF_CELL)
                                            }, /* check with self will forbid to move on the spot */
                                        )
                                        && self.props.iter().all(|prop| {
                                            !prop
                                                .position
                                                .compare(&next_position, MORE_THAN_HALF_CELL)
                                        })
                                        && !self
                                            .player
                                            .position
                                            .compare(&next_position, MORE_THAN_HALF_CELL)
                                    {
                                        new_enemy.position = next_position;
                                        true
                                    } else {
                                        false
                                    }
                                    }
                                    EnemyActionType::Fire(direction, speed) => {
                                        self.bullets.push(Bullet {
                                            move_direction: *direction,
                                            position: Point::new(
                                                new_enemy.position.x,
                                                new_enemy.position.y + FIRE_BULLET_OFFSET,
                                            ),
                                            speed: *speed,
                                        });
                                        true
                                    }
                                    EnemyActionType::Wait => true,
                                }
                            {
                                behavior.to_next_move += action.duration;
                                behavior.next_action();
                                break 'failures;
                            }
                            behavior.next_action();

                            if behavior.current_action == start_action_ind {
                                break 'failures;
                            }
                        }
                    }
                }

                self.enemies = new_enemies;
            }

            // bullets movement
            // modifies bullets
            {
                for bullet in &mut self.bullets {
                    let bullet_position = &mut bullet.position;
                    let bullet_speed = bullet.speed;
                    match bullet.move_direction {
                        Direction::Up => {
                            bullet_position.y -= bullet_speed;
                        }
                        Direction::Down => {
                            bullet_position.y += bullet_speed;
                        }
                        Direction::Left => {
                            bullet_position.x -= bullet_speed;
                        }
                        Direction::Right => {
                            bullet_position.x += bullet_speed;
                        }
                    }
                }

                // delete out of bounds bullets
                self.bullets.retain(|bullet| {
                    bullet
                        .position
                        .bounds_check(screen_width, screen_height)
                        .is_none()
                });
            }

            // enemies, bullets, props collision
            // modifies self.bullets, self.enemies, self.props, self.score
            {
                // collision states
                // assigned with self values by index
                let mut bullets_collision_state: Vec<bool> =
                    std::iter::repeat(false).take(self.bullets.len()).collect();
                let mut enemies_collision_state: Vec<bool> =
                    std::iter::repeat(false).take(self.enemies.len()).collect();
                let mut props_collision_state: Vec<bool> =
                    std::iter::repeat(false).take(self.props.len()).collect();

                for (bullet_ind, is_bullet_collided) in
                    bullets_collision_state.iter_mut().enumerate()
                {
                    if *is_bullet_collided {
                        continue;
                    };

                    // enemy collision
                    for (enemy_ind, is_enemy_collided) in
                        &mut enemies_collision_state.iter_mut().enumerate()
                    {
                        if *is_enemy_collided {
                            continue;
                        };

                        if self.bullets[bullet_ind]
                            .position
                            .compare(&self.enemies[enemy_ind].position, MORE_THAN_HALF_CELL)
                        {
                            *is_enemy_collided = true;
                            *is_bullet_collided = true;
                            self.score += FOR_ENEMY_SCORE;
                        }
                    }

                    // prop collision
                    for (prop_ind, is_prop_collided) in props_collision_state.iter_mut().enumerate()
                    {
                        if *is_prop_collided {
                            continue;
                        };

                        if self.bullets[bullet_ind]
                            .position
                            .compare(&self.props[prop_ind].position, MORE_THAN_HALF_CELL)
                        {
                            *is_prop_collided = true;
                            *is_bullet_collided = true;
                            if self.props[prop_ind].destroyable {
                                self.score += FOR_PROP_SCORE;
                            }
                        }
                    }
                }

                let mut bullets_collision_state = bullets_collision_state.iter();
                let mut enemies_collision_state = enemies_collision_state.iter();
                let mut props_collision_state = props_collision_state.iter();

                self.bullets.retain(|_| {
                    let is_collided = bullets_collision_state.next().unwrap();
                    !is_collided
                });
                self.enemies.retain(|_| {
                    let is_collided = enemies_collision_state.next().unwrap();
                    !is_collided
                });
                self.props.retain(|prop| {
                    let is_collided = props_collision_state.next().unwrap();
                    !is_collided || !prop.destroyable
                });
            }
        }

        if is_player_collided || quit_requested || self.enemies.is_empty() {
            UpdateEvent::GameOver
        } else {
            UpdateEvent::GameContinue
        }
    }

    fn draw(&self, out: &mut std::io::Stdout, _delta_time: &Duration) -> crossterm::Result<()> {
        use crossterm::{
            cursor::MoveTo,
            execute,
            style::{Print, Stylize},
            terminal::size,
        };
        use std::io::Write;

        let (max_x, max_y) = size().expect("Failed to get terminal size");

        // enemies
        {
            let enemy_rows: Vec<Vec<char>> = {
                let mut enemy_rows = vec![vec![' '; max_x as usize]; max_y as usize];

                for enemy in &self.enemies {
                    let enemy_screen_position = enemy.position;

                    let enemy_row = &mut enemy_rows[enemy_screen_position.y as usize];

                    if enemy_screen_position.x.round() as u16 * 2 < max_x {
                        enemy_row[enemy_screen_position.x as usize * 2] = '◥';
                        enemy_row[enemy_screen_position.x as usize * 2 + 1] = '◤';
                    }
                }

                enemy_rows
            };

            for (ind, enemy_row) in enemy_rows.iter().enumerate() {
                execute!(
                    out,
                    MoveTo(0, ind as u16),
                    Print(enemy_row.iter().collect::<String>().red())
                )?;
            }
        }

        // bullets
        {
            for bullet in &self.bullets {
                let bullet_screen_position = Point::<ScreenBasis>::from(bullet.position);

                execute!(
                    out,
                    MoveTo(
                        bullet_screen_position.x as u16,
                        bullet_screen_position.y as u16
                    ),
                    Print(match bullet.move_direction {
                        Direction::Up => "<>".green(),
                        Direction::Left | Direction::Right => "<>".yellow(),
                        Direction::Down => "<>".red(),
                    })
                )?;
            }
        }

        // props
        {
            for prop in &self.props {
                let prop_screen_position = Point::<ScreenBasis>::from(prop.position);

                execute!(
                    out,
                    MoveTo(prop_screen_position.x as u16, prop_screen_position.y as u16),
                    Print(if prop.destroyable {
                        "▓▓".green()
                    } else {
                        "▓▓".blue()
                    })
                )?;
            }
        }

        // score
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
                    max_x - score_hint.len() as u16 - digits_num(self.score),
                    max_y - 1
                )
            )?;
            write!(out, "Score: {}", self.score)?;
        }

        // player
        {
            let player_screen_position: Point<ScreenBasis> = self.player.position.into();

            execute!(
                out,
                MoveTo(
                    player_screen_position.x as u16,
                    player_screen_position.y as u16
                ),
                Print("◢◣".green())
            )?;
        }

        execute!(out, MoveTo(0, 0))
    }
}
