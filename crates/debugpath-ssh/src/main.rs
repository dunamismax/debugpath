#[tokio::main]
async fn main() -> debugpath_ssh::Result<()> {
    let config = debugpath_ssh::LocalSshConfig::from_env()?;
    debugpath_ssh::run_local_dev_server(config).await
}
