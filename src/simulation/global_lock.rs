use std::sync::{LazyLock, Mutex};

pub static CRITICAL_SECTION: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
