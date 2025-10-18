use crate::simulation::interactor::Interactor;
use std::collections::HashMap;

pub type Pos = (i32, i32);

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub struct Node {
    pub visited: bool,
    pub neighbors: HashMap<Direction, Pos>,
}

impl Node {
    pub fn new(visited: bool, neighbors: HashMap<Direction, Pos>) -> Self {
        Node { visited, neighbors }
    }

    pub fn make_visited(&self) -> Self {
        Node {
            visited: true,
            neighbors: self.neighbors.clone(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    const ALL: [Direction; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

pub struct Solver {
    map: HashMap<Pos, Node>,
    interactor: Box<dyn Interactor + Send + Sync>,
}

impl Solver {
    pub fn new(interactor: Box<dyn Interactor + Send + Sync>) -> Self {
        Self {
            map: HashMap::new(),
            interactor,
        }
    }

    fn explore(&mut self, pos: Pos) {
        self.map.entry(pos).or_default().visited = true;

        for &dir in &Direction::ALL {
            let (dx, dy) = dir.delta();
            let next_pos = (pos.0 + dx, pos.1 + dy);

            if self.map.get(&next_pos).is_some_and(|n| n.visited) {
                continue; // Already visited
            }

            // Try to go in this direction
            if self.interactor.try_direction(dir) {
                // Record connection in the map
                self.map
                    .get_mut(&pos)
                    .unwrap()
                    .neighbors
                    .insert(dir, next_pos);
                self.map
                    .entry(next_pos)
                    .or_default()
                    .neighbors
                    .insert(dir.opposite(), pos);

                // Walk to the new position
                self.interactor.walk(dir);

                // Recursively explore
                self.explore(next_pos);

                // Backtrack
                self.interactor.walk_back(dir.opposite());
            }
        }
    }

    fn start_explore(&mut self) {
        self.explore((0, 0));
    }
}

mod tests {
    use crate::simulation::char_state::CharState;
    use crate::simulation::maze_solver::Direction::*;
    use crate::simulation::maze_solver::{Node, Solver};
    use crate::simulation::simulation_state::DebugObj;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_solver() {
        // MAP: starting top left at (0, 0)
        //
        // A(0,0) - B(1,0) - C(2,0)
        //          |        |
        //          D(1,1) - E(2,1)
        //          |        |
        // F(0,2) - G(1,2) - H(2,2)

        let test_map = Arc::new(Mutex::new(HashMap::new()));
        {
            let map = test_map.clone();
            let mut map = map.lock().unwrap();
            //A node
            map.insert((0, 0), Node::new(true, HashMap::from([(Right, (1, 0))])));
            //B node
            map.insert(
                (1, 0),
                Node::new(
                    false,
                    HashMap::from([(Left, (0, 0)), (Right, (2, 0)), (Down, (1, 1))]),
                ),
            );
            //C node
            map.insert(
                (2, 0),
                Node::new(false, HashMap::from([(Left, (1, 0)), (Down, (2, 1))])),
            );
            //D node
            map.insert(
                (1, 1),
                Node::new(
                    false,
                    HashMap::from([(Up, (1, 0)), (Down, (1, 2)), (Right, (2, 1))]),
                ),
            );
            //E node
            map.insert(
                (2, 1),
                Node::new(
                    false,
                    HashMap::from([(Up, (2, 0)), (Down, (2, 2)), (Left, (1, 1))]),
                ),
            );
            //F node
            map.insert((0, 2), Node::new(false, HashMap::from([(Right, (1, 2))])));
            //G node
            map.insert(
                (1, 2),
                Node::new(
                    false,
                    HashMap::from([(Up, (1, 1)), (Left, (0, 2)), (Right, (2, 2))]),
                ),
            );
            //H node
            map.insert(
                (2, 2),
                Node::new(false, HashMap::from([(Up, (2, 1)), (Left, (1, 2))])),
            );
        }
        let mut solver = Solver::new(Box::new(DebugObj::new(
            CharState::InDungeon,
            test_map.clone(),
            0.into(),
            0.into(),
        )));
        solver.start_explore();

        assert_eq!(solver.map.len(), 8);
        assert!(solver.map.iter().all(|item| item.1.visited))
    }
}
