pub mod memory_pool;
pub mod miner;


// Instruction set for the miner
pub enum Instruction{
    StartMining,
    StopMining,
    NewContent
// To be added later for efficiency
//    NewTx,
//    NewTxBlockContent,
//    NewPropBlockContent,
//    NewVoterBlockContent(u16)
}