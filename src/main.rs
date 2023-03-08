extern crate static_assertions as sa;
pub mod game;
pub mod pong;
pub mod snake;
pub mod space_invaders;
pub mod tetris;

enum MenuChoice {
    Exit = 0,
    SnakeGame = 1,
    TetrisGame = 2,
    SpaceInvadersGame = 3,
    Pong,

    #[allow(dead_code)]
    LastElement, // for static check
}

fn main() -> crossterm::Result<()> {
    use crossterm::{cursor, event::read, execute, terminal};
    use game::Game;
    use snake::{Point, SnakeGame};
    use terminal::{Clear, ClearType};

    let mut stdout = std::io::stdout();
    let stdin_chan = spawn_stdin_channel();

    'main_loop: loop {
        // Create all games on stack
        let (mut snake, mut tetris, mut space_invaders, mut pong);

        // Make game from player choice
        let game: &mut dyn Game = match {
            let mut choice;
            'input_read: loop {
                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                println!("Choose a game:");
                println!("   {}. Exit", MenuChoice::Exit as usize);
                println!("   {}. Snake", MenuChoice::SnakeGame as usize);
                println!("   {}. Tetris", MenuChoice::TetrisGame as usize);
                println!(
                    "   {}. Space invaders",
                    MenuChoice::SpaceInvadersGame as usize
                );
                println!("   {}. Pong", MenuChoice::Pong as usize);

                choice = read_game_choice();

                if choice.is_some() {
                    break 'input_read;
                }
            }
            choice
        } {
            Some(MenuChoice::SnakeGame) => {
                snake = SnakeGame::new(Point { x: 10, y: 10 });
                &mut snake
            }
            Some(MenuChoice::TetrisGame) => {
                tetris = tetris::TetrisGame::new();
                &mut tetris
            }
            Some(MenuChoice::SpaceInvadersGame) => {
                let (w, h) = crossterm::terminal::size().expect("Failed to get terminal size");

                space_invaders = space_invaders::SpaceInvadersGame::new(
                    h,
                    w,
                    space_invaders::EnemyPreset::RandomFire,
                    space_invaders::PropsPreset::Wall,
                );
                &mut space_invaders
            }
            Some(MenuChoice::Pong) => {
                pong = pong::PongGame::new();
                &mut pong
            }
            Some(MenuChoice::Exit) => break 'main_loop,
            Some(MenuChoice::LastElement) | None => unreachable!(),
        };

        let mut prev_time = std::time::SystemTime::now();

        'game_loop: loop {
            use std::thread;
            use std::time::Duration;
            use std::time::SystemTime;

            let current_time = SystemTime::now();

            // Clear the screen
            execute!(stdout, Clear(ClearType::All))?;

            // Update the game state
            if let game::UpdateEvent::GameOver = game.update(
                &read_input(&stdin_chan),
                &current_time.duration_since(prev_time).unwrap(),
            ) {
                break 'game_loop;
            }

            // Draw the game state
            game.draw(
                &mut stdout,
                &current_time.duration_since(prev_time).unwrap(),
            )?;

            // Wait for the next frame
            thread::sleep(Duration::from_millis(100));

            prev_time = current_time;
        }

        println!("Game over! Score: {}", game.get_score().value);
        println!("Press any key to restart.");
        // Wait for prevent the game from restarting immediately
        std::thread::sleep(std::time::Duration::from_millis(750));
        _ = read();
    }

    Ok(())
}

// https://stackoverflow.com/questions/30012995/how-can-i-read-non-blocking-from-stdin
fn spawn_stdin_channel() -> std::sync::mpsc::Receiver<crossterm::event::KeyEvent> {
    let (tx, rx) = std::sync::mpsc::channel::<crossterm::event::KeyEvent>();
    use crossterm::event::{read, Event};
    use std::thread;

    thread::spawn(move || loop {
        if let Ok(Event::Key(key)) = read() {
            match tx.send(key) {
                Ok(_) => {}
                Err(_) => break, // stdin disconnected
            }
        }
    });

    rx
}

fn read_input(
    rx: &std::sync::mpsc::Receiver<crossterm::event::KeyEvent>,
) -> Option<crossterm::event::KeyEvent> {
    use std::sync::mpsc::TryRecvError;

    let result = match rx.try_recv() {
        Ok(input) => Some(input),
        Err(TryRecvError::Disconnected) => panic!("stdin disconnected"),
        Err(TryRecvError::Empty) => None,
    };

    // Skip all other inputs
    loop {
        match rx.try_recv() {
            Ok(_) => {}
            Err(TryRecvError::Disconnected) => panic!("stdin disconnected"),
            Err(TryRecvError::Empty) => break,
        }
    }

    result
}

fn read_game_choice() -> Option<MenuChoice> {
    use std::io::stdin;

    let mut input = String::new();
    stdin().read_line(&mut input).ok()?;
    let choice = input.trim().parse::<usize>().ok()?;

    sa::const_assert!(MenuChoice::Exit as usize == 0);
    sa::const_assert!(MenuChoice::SnakeGame as usize == 1);
    sa::const_assert!(MenuChoice::TetrisGame as usize == 2);
    sa::const_assert!(MenuChoice::SpaceInvadersGame as usize == 3);
    sa::const_assert!(MenuChoice::Pong as usize == 4);

    sa::const_assert!(MenuChoice::LastElement as usize == 5);

    match choice {
        0 => Some(MenuChoice::Exit),
        1 => Some(MenuChoice::SnakeGame),
        2 => Some(MenuChoice::TetrisGame),
        3 => Some(MenuChoice::SpaceInvadersGame),
        4 => Some(MenuChoice::Pong),
        _ => None,
    }
}
