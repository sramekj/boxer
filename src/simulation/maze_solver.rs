use crate::simulation::interactor::Interactor;
use crate::simulation::keys::{Key, WALK_DOWN, WALK_LEFT, WALK_RIGHT, WALK_UP};
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

    pub fn to_key(self) -> Key {
        match self {
            Direction::Left => WALK_LEFT,
            Direction::Right => WALK_RIGHT,
            Direction::Up => WALK_UP,
            Direction::Down => WALK_DOWN,
        }
    }

    const ALL: [Direction; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

pub struct Solver {
    map: HashMap<Pos, Node>,
    interactor: Box<dyn Interactor + Send + Sync>,
    stack: Vec<(Pos, Vec<Direction>)>,
    current_pos: Pos,
}

impl Solver {
    pub fn new(interactor: Box<dyn Interactor + Send + Sync>) -> Self {
        let mut map = HashMap::new();
        let start_pos = (0, 0);
        map.insert(start_pos, Node::default());
        Self {
            map,
            interactor,
            stack: vec![(start_pos, Direction::ALL.to_vec())],
            current_pos: start_pos,
        }
    }

    fn explore_step(&mut self) -> bool {
        if self.stack.is_empty() {
            return true; // Exploration done
        }

        let (pos, directions) = self.stack.last_mut().unwrap();

        // Mark current node visited
        self.map.entry(*pos).or_default().visited = true;

        while let Some(dir) = directions.pop() {
            let delta = dir.delta();
            let next_pos = (pos.0 + delta.0, pos.1 + delta.1);

            if self.map.get(&next_pos).is_some_and(|n| n.visited) {
                continue; // Already visited
            }

            // Try to move in this direction
            if self.interactor.try_direction(dir) {
                // Update map with the connection
                self.map
                    .get_mut(pos)
                    .unwrap()
                    .neighbors
                    .insert(dir, next_pos);
                self.map
                    .entry(next_pos)
                    .or_default()
                    .neighbors
                    .insert(dir.opposite(), *pos);

                // Perform the move
                self.interactor.walk(dir);
                self.current_pos = next_pos;

                // Push new node onto the stack with all 4 directions
                self.stack.push((next_pos, Direction::ALL.to_vec()));

                return false; // only 1 move per step
            }
        }

        // No more directions to try — backtrack
        self.stack.pop();

        if let Some((parent_pos, _)) = self.stack.last() {
            let back_dir = Direction::ALL.iter().find(|&&d| {
                let delta = d.delta();
                let candidate = (self.current_pos.0 + delta.0, self.current_pos.1 + delta.1);
                candidate == *parent_pos
            });

            if let Some(dir) = back_dir {
                self.interactor.walk(*dir);
                self.current_pos = *parent_pos;
            }
        }

        false // Backtracked, still not done
    }
}

#[cfg(test)]
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

        while !solver.explore_step() {}

        assert_eq!(solver.map.len(), 8);
        assert!(solver.map.iter().all(|item| item.1.visited))
    }
}
