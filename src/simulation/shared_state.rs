use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum SharedStateMessage {
    SetSkillHasteApplied(bool),
    SetFrenzyApplied(bool),
    GetSkillHasteApplied(Sender<bool>),
    GetFrenzyApplied(Sender<bool>),
    GetSkillHastePercent(Sender<f32>),
    GetFrenzyPercent(Sender<f32>),
    Stop,
}

#[derive(Debug)]
struct SharedStateActor {
    skill_haste_buff_applied: bool,
    skill_haste_percent: f32,
    frenzy_buff_applied: bool,
    frenzy_percent: f32,
    receiver: Receiver<SharedStateMessage>,
}

impl SharedStateActor {
    fn new(
        skill_haste_percent: f32,
        frenzy_percent: f32,
        receiver: Receiver<SharedStateMessage>,
    ) -> SharedStateActor {
        SharedStateActor {
            skill_haste_buff_applied: false,
            skill_haste_percent,
            frenzy_buff_applied: false,
            frenzy_percent,
            receiver,
        }
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                SharedStateMessage::SetSkillHasteApplied(bool) => {
                    self.skill_haste_buff_applied = bool;
                }
                SharedStateMessage::SetFrenzyApplied(bool) => {
                    self.frenzy_buff_applied = bool;
                }
                SharedStateMessage::GetSkillHasteApplied(sender) => {
                    let _ = sender.send(self.skill_haste_buff_applied);
                }
                SharedStateMessage::GetFrenzyApplied(sender) => {
                    let _ = sender.send(self.frenzy_buff_applied);
                }
                SharedStateMessage::GetSkillHastePercent(sender) => {
                    let _ = sender.send(self.skill_haste_percent);
                }
                SharedStateMessage::GetFrenzyPercent(sender) => {
                    let _ = sender.send(self.frenzy_percent);
                }
                SharedStateMessage::Stop => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedStateHandle {
    sender: Sender<SharedStateMessage>,
}

impl SharedStateHandle {
    pub fn new(skill_haste_percent: f32, frenzy_percent: f32) -> Self {
        let (tx, rx) = mpsc::channel();
        let actor = SharedStateActor::new(skill_haste_percent, frenzy_percent, rx);
        thread::spawn(move || actor.run());
        Self { sender: tx }
    }

    pub fn set_skill_haste_applied(&self, state: bool) {
        let _ = self
            .sender
            .send(SharedStateMessage::SetSkillHasteApplied(state));
    }

    pub fn set_frenzy_applied(&self, state: bool) {
        let _ = self
            .sender
            .send(SharedStateMessage::SetFrenzyApplied(state));
    }

    pub fn get_skill_haste_applied(&self) -> bool {
        let (tx, rx) = mpsc::channel();
        let _ = self
            .sender
            .send(SharedStateMessage::GetSkillHasteApplied(tx));
        rx.recv().unwrap()
    }

    pub fn get_frenzy_applied(&self) -> bool {
        let (tx, rx) = mpsc::channel();
        let _ = self.sender.send(SharedStateMessage::GetFrenzyApplied(tx));
        rx.recv().unwrap()
    }

    pub fn get_skill_haste_percent(&self) -> f32 {
        let (tx, rx) = mpsc::channel();
        let _ = self
            .sender
            .send(SharedStateMessage::GetSkillHastePercent(tx));
        rx.recv().unwrap()
    }

    pub fn get_frenzy_percent(&self) -> f32 {
        let (tx, rx) = mpsc::channel();
        let _ = self.sender.send(SharedStateMessage::GetFrenzyPercent(tx));
        rx.recv().unwrap()
    }

    pub fn stop(&self) {
        let _ = self.sender.send(SharedStateMessage::Stop);
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::shared_state::SharedStateHandle;
    use std::sync::Arc;

    #[test]
    fn test_shared_state_is_singleton() {
        let state = Arc::new(SharedStateHandle::new(1.0, 1.0));
        let s1 = state.clone();
        let s2 = state.clone();

        s1.set_skill_haste_applied(true);
        assert!(s2.get_skill_haste_applied());

        s1.set_skill_haste_applied(false);
        assert!(!s2.get_skill_haste_applied());

        s1.stop();
        s2.stop();
    }
}
