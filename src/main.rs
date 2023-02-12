pub mod game;
pub mod snake;

fn main() -> crossterm::Result<()> {
    use crossterm::{execute, terminal};
    use game::Game;
    use snake::{Point, SnakeGame};
    use terminal::{Clear, ClearType};

    let mut stdout = std::io::stdout();
    let stdin_chan = spawn_stdin_channel();
    let settings = Point { x: 10, y: 10 };
    let mut game: Box<dyn Game<Events = _>> = Box::new(SnakeGame::new(settings));
    let mut prev_time = std::time::SystemTime::now();

    loop {
        use std::thread;
        use std::time::Duration;
        use std::time::SystemTime;

        let current_time = SystemTime::now();

        // Clear the screen
        execute!(stdout, Clear(ClearType::All))?;

        // Update the game state
        match game.update(
            &read_input(&stdin_chan),
            &current_time.duration_since(prev_time).unwrap(),
        ) {
            None => panic!("Game over!"),
            _ => {}
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
}

// https://stackoverflow.com/questions/30012995/how-can-i-read-non-blocking-from-stdin
fn spawn_stdin_channel() -> std::sync::mpsc::Receiver<crossterm::event::KeyEvent> {
    let (tx, rx) = std::sync::mpsc::channel::<crossterm::event::KeyEvent>();
    use crossterm::event::{read, Event};
    use std::thread;

    thread::spawn(move || loop {
        if let Ok(event) = read() {
            if let Event::Key(key) = event {
                tx.send(key).unwrap();
            }
        }
    });
    rx
}

fn read_input(
    rx: &std::sync::mpsc::Receiver<crossterm::event::KeyEvent>,
) -> Option<crossterm::event::KeyEvent> {
    use std::sync::mpsc::TryRecvError;

    let result;
    match rx.try_recv() {
        Ok(input) => result = Some(input),
        Err(TryRecvError::Disconnected) => panic!("stdin disconnected"),
        Err(TryRecvError::Empty) => result = None,
    }
    loop {
        match rx.try_recv() {
            Ok(_) => {}
            Err(TryRecvError::Disconnected) => panic!("stdin disconnected"),
            Err(TryRecvError::Empty) => break,
        }
    }
    result
}
