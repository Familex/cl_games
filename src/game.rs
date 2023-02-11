/// A trait that defines the interface for a game.
pub trait Game {
    /// The type of settings that the game accepts.
    type Settings;

    /// The type of input that the game accepts.
    type Input;

    /// The type of event that the game produces.
    type Events;

    /// Create a new game instance with the given settings.
    fn new(setup: Self::Settings) -> Self
    where
        Self: Sized;

    /// Update the game state with the given input.
    fn update(&mut self, input: &Self::Input, delta_time: &std::time::Duration) -> Self::Events;

    /// Draw the game state to the given output.
    fn draw(
        &self,
        out: &mut std::io::Stdout,
        delta_time: &std::time::Duration,
    ) -> crossterm::Result<()>;

    /// Read the input from the given input stream.
    fn read_to_input(&self, event: &Option<crossterm::event::KeyEvent>) -> Self::Input;
}
