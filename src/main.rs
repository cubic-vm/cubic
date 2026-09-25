#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    // Restore the terminal when a panic hits an ssh or console session
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        crossterm::terminal::disable_raw_mode().ok();
        default_hook(info);
    }));

    // reqwest uses the process wide crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let code = cubic::CubicApp::run().await;
    std::process::exit(code)
}
