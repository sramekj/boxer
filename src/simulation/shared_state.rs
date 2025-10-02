#[derive(Debug)]
pub struct SharedState {
    pub skill_haste_buff_applied: bool,
    pub skill_haste_percent: f32,
}

impl SharedState {
    pub fn new(skill_haste_percent: f32) -> SharedState {
        SharedState {
            skill_haste_buff_applied: false,
            skill_haste_percent,
        }
    }

    pub fn set_skill_haste(&mut self, state: bool) {
        println!("Skill haste available changed to: {}", state);
        self.skill_haste_buff_applied = state;
    }

    pub fn get_skill_haste(&self) -> bool {
        self.skill_haste_buff_applied
    }

    pub fn get_skill_haste_percent(&self) -> f32 {
        self.skill_haste_percent
    }
}
