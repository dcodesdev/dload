use dload::Downloader;

#[tokio::main]
async fn main() {
    let url = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

    Downloader::new()
        .set_output_dir("temp")
        .file_name("rust-logo.png")
        .download(url)
        .await
        .unwrap();
}
