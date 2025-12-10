#[macro_export]
macro_rules! time_func {
    ($vec:expr, $body:block) => {{
        let __start = std::time::Instant::now();
        let __result = { $body };
        let __elapsed = __start.elapsed().as_nanos() as u64;
        $vec.push(__elapsed);
        __result
    }};
}