/// Represents a move operation between 2 registers.
pub struct MoveOperation<T> {
    /// The source register for the move operation.
    pub source: T,

    /// The target (destination) register for the move operation.
    pub target: T,
}
