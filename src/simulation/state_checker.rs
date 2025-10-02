use crate::simulation::{CharState, DebugObj, WindowObj};
use crate::win_util::get_pixel_color;

pub trait StateChecker {
    fn get_state(&self) -> CharState;
}

impl StateChecker for DebugObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        let state = self.test_state;
        println!("Setting state to {:?}", state);
        state
    }
}

impl StateChecker for WindowObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        //TODO.....
        let x = get_pixel_color(self.hwnd, 100, 100);
        let state = CharState::Fighting;
        println!("Setting state to {:?}", state);
        state
    }
}
