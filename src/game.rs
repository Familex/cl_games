/// A trait that defines the interface for a game.
pub trait Game {
    /// The type of event that the game produces.
    type Events;

    /// Update the game state with the given input.
    fn update(
        &mut self,
        input: &Option<crossterm::event::KeyEvent>,
        delta_time: &std::time::Duration,
    ) -> Self::Events;

    /// Draw the game state to the given output.
    fn draw(
        &self,
        out: &mut std::io::Stdout,
        delta_time: &std::time::Duration,
    ) -> crossterm::Result<()>;
}
