use crate::{instruction::Instruction, state::StateDiff};

#[async_trait::async_trait]
pub trait Runtime<IX: Instruction<T>, T> {
    type Error: std::error::Error;

    async fn execute_instruction(&self, instruction: IX) -> Result<StateDiff<T>, Self::Error>;
}