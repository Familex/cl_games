use crate::game::{Game, UpdateEvent};
use rand::Rng;
use std::time::Duration;

const FOR_ENEMY_SCORE: usize = 1;
const FOR_PROP_SCORE: usize = 1;
const FIRE_BULLET_OFFSET: f32 = 1.0;

pub fn is_success(chance: f32) -> bool {
    let mut rng = rand::thread_rng();
    let random: f32 = rng.gen();
    random < chance / 100.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
pub struct Bullet {
    move_direction: Direction,
    position: Point,
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
        assert!(chance >= 0.0 && chance <= 100.0);

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

        Self {
            actions,
            to_next_move,
            current_action,
        }
    }

    fn current_action(&self) -> EnemyAction {
        self.actions[self.current_action].clone()
    }

    fn to_next_action(&mut self) {
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
    position: Point,
    behavior: EnemyBehavior,
}

#[derive(Clone)]
pub struct Prop {
    position: Point,
    destroyable: bool,
}

pub struct Player {
    position: Point,
}

pub struct SpaceInvadersGame {
    score: usize,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    props: Vec<Prop>,
    player: Player,
}

pub enum EnemyPreset {
    Empty,
    CheckeredLeftRight,
    CheckeredRightDownLeftUp,
    CheckeredLeft,
}

pub enum PropsPreset {
    Empty,
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
                                position: Point {
                                    x: x as f32 * 2.0 + y as f32 % 2.0,
                                    y: y as f32,
                                },
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
                                position: Point {
                                    x: x as f32 * 2.0 + y as f32 % 2.0,
                                    y: y as f32,
                                },
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
                                position: Point {
                                    x: x as f32 * 2.0 + y as f32 % 2.0,
                                    y: y as f32,
                                },
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
            },
            props: match props_preset {
                PropsPreset::Empty => vec![],
            },
            player: Player {
                position: Point {
                    x: screen_width as f32 / 2.0 / 2.0,
                    y: screen_height as f32 - 1.0,
                },
            },
        }
    }
}

fn bounds_check(position: &Point) -> bool {
    let (max_x, max_y) = crossterm::terminal::size().expect("Failed to get terminal size");
    position.x /* * 2 */ >= 0.0
        && position.x.round() * 2.0 < max_x as f32
        && position.y >= 0.0
        && position.y.round() < max_y as f32
}

impl Game for SpaceInvadersGame {
    fn get_score(&self) -> usize {
        self.score
    }

    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        delta_time: &Duration,
    ) -> UpdateEvent {
        // quit request
        let quit_requested = match input {
            Some(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('q'),
                ..
            }) => true,
            _ => false,
        };

        // player movement
        // modifies self.player
        {
            let next_position = match input {
                Some(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Left,
                    ..
                }) => Some(Point {
                    x: self.player.position.x - 1.0,
                    y: self.player.position.y,
                }),
                Some(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Right,
                    ..
                }) => Some(Point {
                    x: self.player.position.x + 1.0,
                    y: self.player.position.y,
                }),
                Some(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Char(' '),
                    ..
                }) => {
                    self.bullets.push(Bullet {
                        move_direction: Direction::Up,
                        position: Point {
                            x: self.player.position.x,
                            y: self.player.position.y - 1.0,
                        },
                        speed: 1.0,
                    });
                    None
                }
                _ => None,
            };

            if let Some(next_position) = next_position {
                if bounds_check(&next_position)
                    && self.props.iter().all(|prop| prop.position != next_position)
                {
                    self.player.position = next_position;
                }
            }
        }

        // enemies movement
        // modifies self.enemies
        // FIXME deobfuscate self variables access (cause of borrow checker)
        {
            for enemy_ind in 0..self.enemies.len() {
                let enemy_position = self.enemies[enemy_ind].position.clone();
                let behavior = std::cell::RefCell::new(&mut self.enemies[enemy_ind].behavior);
                let action = behavior.borrow().current_action();
                let start_action_ind = behavior.borrow().current_action;
                if self.enemies[enemy_ind].behavior.to_next_move.as_nanos() == 0 {
                    // 'failures is do-while loop
                    'failures: loop {
                        if is_success(action.chance) {
                            if match &action.action_type {
                                EnemyActionType::Move(direction, speed) => {
                                    let next_position = {
                                        match direction {
                                            Direction::Up => Point {
                                                x: enemy_position.x,
                                                y: enemy_position.y - speed,
                                            },
                                            Direction::Down => Point {
                                                x: enemy_position.x,
                                                y: enemy_position.y + speed,
                                            },
                                            Direction::Left => Point {
                                                x: enemy_position.x - speed,
                                                y: enemy_position.y,
                                            },
                                            Direction::Right => Point {
                                                x: enemy_position.x + speed,
                                                y: enemy_position.y,
                                            },
                                        }
                                    };
                                    if bounds_check(&next_position)
                                        && self.enemies.iter().enumerate().all(
                                            |(other_ind, other)| {
                                                other_ind == enemy_ind
                                                    || other.position != next_position
                                            },
                                        )
                                        && self
                                            .props
                                            .iter()
                                            .all(|prop| prop.position != next_position)
                                    {
                                        self.enemies[enemy_ind].position = next_position;
                                        true
                                    } else {
                                        false
                                    }
                                }
                                EnemyActionType::Fire(direction, speed) => {
                                    self.bullets.push(Bullet {
                                        move_direction: *direction,
                                        position: Point {
                                            x: enemy_position.x,
                                            y: enemy_position.y + FIRE_BULLET_OFFSET,
                                        },
                                        speed: *speed,
                                    });
                                    true
                                }
                                EnemyActionType::Wait => true,
                            } {
                                self.enemies[enemy_ind].behavior.to_next_move += action.duration;
                                break 'failures;
                            }
                        }
                        self.enemies[enemy_ind].behavior.to_next_action();

                        if self.enemies[enemy_ind].behavior.current_action == start_action_ind {
                            break 'failures;
                        }
                    }
                }
                self.enemies[enemy_ind].behavior.delta(*delta_time);
            }
        }

        // bullets movement
        // modifies bullets
        // FIXME not work on a certain terminal scale
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
            self.bullets.retain(|bullet| bounds_check(&bullet.position));
        }

        // player bullet collision
        let is_player_collided = self
            .bullets
            .iter()
            .any(|bullet| self.player.position == bullet.position);

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

            for (bullet_ind, is_bullet_collided) in bullets_collision_state.iter_mut().enumerate() {
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

                    if self.bullets[bullet_ind].position == self.enemies[enemy_ind].position {
                        *is_enemy_collided = true;
                        *is_bullet_collided = true;
                        self.score += FOR_ENEMY_SCORE;
                    }
                }

                // prop collision
                for (prop_ind, is_prop_collided) in props_collision_state.iter_mut().enumerate() {
                    if *is_prop_collided {
                        continue;
                    };

                    if self.bullets[bullet_ind].position == self.props[prop_ind].position {
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
                !is_collided && prop.destroyable
            });
        }

        if is_player_collided || quit_requested {
            UpdateEvent::GameOver
        } else {
            UpdateEvent::GameContinue
        }
    }

    fn draw(&self, out: &mut std::io::Stdout, _delta_time: &Duration) -> crossterm::Result<()> {
        use crossterm::{cursor::MoveTo, execute, style, terminal::size};
        use std::io::Write;

        let (max_x, max_y) = size().expect("Failed to get terminal size");

        // enemies
        {
            let enemies = &self.enemies;
            for enemy in enemies {
                let enemy_position = &enemy.position;
                execute!(
                    out,
                    MoveTo(enemy_position.x as u16 * 2, enemy_position.y as u16),
                    style::Print("◥◤")
                )?;
            }
        }

        // bullets
        {
            let bullets = &self.bullets;
            for bullet in bullets {
                let bullet_position = &bullet.position;
                execute!(
                    out,
                    MoveTo(bullet_position.x as u16 * 2, bullet_position.y as u16),
                    style::Print("<>")
                )?;
            }
        }

        // props
        {
            let props = &self.props;
            for prop in props {
                let prop_position = &prop.position;
                execute!(
                    out,
                    MoveTo(prop_position.x as u16 * 2, prop_position.y as u16),
                    style::Print("▓▓")
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
            let player = &self.player;
            let player_position = &player.position;
            execute!(
                out,
                MoveTo(player_position.x as u16 * 2, player_position.y as u16),
                style::Print("◢◣")
            )?;
        }

        execute!(out, MoveTo(0, 0))
    }
}
