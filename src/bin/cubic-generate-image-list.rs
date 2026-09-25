use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    let Some(output) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: cubic-generate-image-list <file>");
        std::process::exit(1)
    };

    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let code = cubic::GenerateImageListApp::run(&output).await;
    std::process::exit(code)
}
