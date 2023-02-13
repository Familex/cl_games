extern crate static_assertions as sa;
pub mod game;
pub mod snake;

enum GameChoice {
    None = 0,
    Snake = 1,
}

fn main() -> crossterm::Result<()> {
    use crossterm::{cursor, event::read, execute, terminal};
    use game::Game;
    use snake::{Point, SnakeGame};
    use terminal::{Clear, ClearType};

    let mut stdout = std::io::stdout();
    let stdin_chan = spawn_stdin_channel();
    let settings = Point { x: 10, y: 10 };
    let mut game: Box<dyn Game>;
    let mut prev_time = std::time::SystemTime::now();

    'main_loop: loop {
        // Menu
        match {
            let mut choice;
            'input_read: loop {
                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                println!("Choose a game:");
                println!("   {}. Exit", GameChoice::None as usize);
                println!("   {}. Snake", GameChoice::Snake as usize);

                choice = read_game_choice();

                if choice.is_some() {
                    break 'input_read;
                }
            }
            choice
        } {
            Some(GameChoice::Snake) => game = Box::new(SnakeGame::new(settings)),
            Some(GameChoice::None) => break 'main_loop,
            None => unreachable!(),
        }

        'game_loop: loop {
            use std::thread;
            use std::time::Duration;
            use std::time::SystemTime;

            let current_time = SystemTime::now();

            // Clear the screen
            execute!(stdout, Clear(ClearType::All))?;

            // Update the game state
            if !game.update(
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

        println!("Game over! Score: {}", game.get_score());
        println!("Press any key to restart.");
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

fn read_game_choice() -> Option<GameChoice> {
    use std::io::stdin;

    let mut input = String::new();
    stdin().read_line(&mut input).ok()?;
    let choice = input.trim().parse::<usize>().ok()?;

    sa::const_assert!(GameChoice::None as usize == 0);
    sa::const_assert!(GameChoice::Snake as usize == 1);

    match choice {
        0 => Some(GameChoice::None),
        1 => Some(GameChoice::Snake),
        _ => None,
    }
}
