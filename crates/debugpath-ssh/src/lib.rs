pub const LOCAL_DEV_BIND_ADDR: &str = "127.0.0.1:2222";
pub const PRODUCTION_ENTRYPOINT: &str = "ssh debugpath.dev";

pub fn smoke_summary() -> String {
    format!("{PRODUCTION_ENTRYPOINT} planned; local development will bind {LOCAL_DEV_BIND_ADDR}")
}
