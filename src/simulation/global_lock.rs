use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref CRITICAL_SECTION: Mutex<()> = Mutex::new(());
}
