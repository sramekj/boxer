#[macro_export]
#[allow(clippy::redundant_closure_call)]
macro_rules! with_critical_section {
    ($wait_ms:expr, $body:block) => {{
        let _lock = CRITICAL_SECTION.lock().unwrap();

        #[allow(clippy::redundant_closure_call)]
        let result = (|| $body)();

        std::thread::sleep(std::time::Duration::from_millis($wait_ms));
        drop(_lock);
        result
    }};
}
