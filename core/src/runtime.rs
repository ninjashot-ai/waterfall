use crate::instruction::Instruction;

#[async_trait::async_trait]
pub trait Runtime<IX: Instruction<T>, T: Clone>: Clone + Send + Sync + 'static {
    fn push_instruction(&mut self, instruction: IX);
    async fn execute_one(&mut self, instruction: &IX) -> Result<(), anyhow::Error>;
    async fn execute(&mut self) -> Result<(), anyhow::Error>;
}
