use crate::simulation::shared_state::SharedStateMessage::*;
use crate::simulation::type_of;
use colored::Colorize;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug)]
#[non_exhaustive]
enum SharedStateMessage {
    SetSkillHasteApplied(bool, Sender<()>),
    SetFrenzyApplied(bool, Sender<()>),
    GetSkillHasteApplied(Sender<bool>),
    GetFrenzyApplied(Sender<bool>),
    GetSkillHastePercent(Sender<f32>),
    GetFrenzyPercent(Sender<f32>),
    Stop(Sender<()>),
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
            //println!("Received message: {:?}", type_of(&msg));
            match msg {
                SetSkillHasteApplied(bool, sender) => {
                    self.skill_haste_buff_applied = bool;
                    let _ = sender.send(());
                }
                SetFrenzyApplied(bool, sender) => {
                    self.frenzy_buff_applied = bool;
                    let _ = sender.send(());
                }
                GetSkillHasteApplied(sender) => {
                    let _ = sender.send(self.skill_haste_buff_applied);
                }
                GetFrenzyApplied(sender) => {
                    let _ = sender.send(self.frenzy_buff_applied);
                }
                GetSkillHastePercent(sender) => {
                    let _ = sender.send(self.skill_haste_percent);
                }
                GetFrenzyPercent(sender) => {
                    let _ = sender.send(self.frenzy_percent);
                }
                Stop(sender) => {
                    print!("Shutting down {}", type_of(&self));
                    let _ = sender.send(());
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
        println!("Starting {}", type_of(&actor));
        thread::spawn(move || {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| actor.run())) {
                eprintln!("{}", format!("Actor panicked: {:?}", e).red());
            }
        });
        Self { sender: tx }
    }

    fn ask<T>(&self, msg: impl FnOnce(Sender<T>) -> SharedStateMessage) -> T {
        let (tx, rx) = mpsc::channel();
        let _ = self.sender.send(msg(tx));
        rx.recv()
            .expect("Actor thread died or failed to send reply")
    }

    pub fn set_skill_haste_applied(&self, state: bool) {
        self.ask(|tx| SetSkillHasteApplied(state, tx));
    }

    pub fn set_frenzy_applied(&self, state: bool) {
        self.ask(|tx| SetFrenzyApplied(state, tx));
    }

    pub fn get_skill_haste_applied(&self) -> bool {
        self.ask(GetSkillHasteApplied)
    }

    pub fn get_frenzy_applied(&self) -> bool {
        self.ask(GetFrenzyApplied)
    }

    pub fn get_skill_haste_percent(&self) -> f32 {
        self.ask(GetSkillHastePercent)
    }

    pub fn get_frenzy_percent(&self) -> f32 {
        self.ask(GetFrenzyPercent)
    }

    pub fn stop(&self) {
        let (tx, _) = mpsc::channel();
        let _ = self.sender.send(Stop(tx));
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
