use std::cmp::Ordering;
use std::cmp;

pub fn lcb_and_ucb_from_vote_depths(votes: Vec<u32>) -> (f32, f32) {
    let answer: f32 = votes.len() as f32;
    return (answer, answer); //todo: Apply the confirmation logic from the paper
}

#[derive(Eq, PartialEq, Clone)]
pub struct PropOrderingHelper{
    pub level: u32, 
    pub position: Vec<u32>
}

impl Ord for PropOrderingHelper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.level < other.level{ return Ordering::Less }
        else if self.level > other.level{ return Ordering::Greater }

        // If they have same levels then use the position
        let len = cmp::min( self.position.len(), other.position.len() );
        for i in 0..len{
            if self.position[i] < other.position[i] { return Ordering::Less }
            else if self.position[i] > other.position[i] { return Ordering::Greater }
        }
        if self.position.len() == other.position.len() { return Ordering::Equal}
        panic!("This is not supposed to happen");
    }
}

impl PartialOrd for PropOrderingHelper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PropOrderingHelper {
    pub fn new(level: u32, position: Vec<u32>) -> Self{
        return PropOrderingHelper{level, position}
    }
}