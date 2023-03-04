use crate::game::{Game, UpdateEvent};
use std::time::Duration;

const FOR_ENEMY_SCORE: usize = 1;
const FOR_PROP_SCORE: usize = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum EnemyBehaviorType {
    LeftRight(Duration),
}

#[derive(Clone)]
pub struct EnemyBehavior {
    behavior_type: EnemyBehaviorType,
    speed: f32,
    from_last_move: Duration,
    last_move_direction: Direction,
}

#[derive(Clone)]
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
                                behavior: EnemyBehavior {
                                    behavior_type: EnemyBehaviorType::LeftRight(
                                        Duration::from_secs(1),
                                    ),
                                    speed: 1.0,
                                    from_last_move: Duration::from_secs(0),
                                    last_move_direction: Direction::Left,
                                },
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
        {
            let enemies = &mut self.enemies;
            for enemy in enemies {
                let enemy_position = &mut enemy.position;
                let enemy_behavior = &mut enemy.behavior;
                let enemy_speed = enemy_behavior.speed;
                match enemy_behavior.behavior_type {
                    EnemyBehaviorType::LeftRight(duration) => {
                        if enemy_behavior.from_last_move >= duration {
                            enemy_behavior.from_last_move = Duration::from_secs(0);
                            let next_position = match enemy_behavior.last_move_direction {
                                Direction::Left => {
                                    enemy_behavior.last_move_direction = Direction::Right;
                                    Some(Point {
                                        x: enemy_position.x + enemy_speed,
                                        y: enemy_position.y,
                                    })
                                }
                                Direction::Right => {
                                    enemy_behavior.last_move_direction = Direction::Left;
                                    Some(Point {
                                        x: enemy_position.x - enemy_speed,
                                        y: enemy_position.y,
                                    })
                                }
                                _ => None,
                            };
                            if let Some(next_position) = next_position {
                                if bounds_check(&next_position)
                                    && self.props.iter().all(|prop| prop.position != next_position)
                                {
                                    *enemy_position = next_position;
                                }
                            }
                        } else {
                            enemy_behavior.from_last_move += *delta_time;
                        }
                    }
                }
            }
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

        if is_player_collided {
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
