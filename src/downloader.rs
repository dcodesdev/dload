use futures::StreamExt;
use reqwest::header::IntoHeaderName;
use reqwest::{header::HeaderMap, Client};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct Downloader {
    header: HeaderMap,
    output_dir: PathBuf,
    file_name: Option<String>,
    client: Client,
    verbose: bool,
}

impl Downloader {
    pub fn new() -> Self {
        let header = HeaderMap::new();

        Downloader {
            header: header.clone(),
            output_dir: std::env::current_dir().unwrap(),
            file_name: None,
            client: Client::builder().default_headers(header).build().unwrap(),
            verbose: false,
        }
    }

    pub fn file_name(mut self, file_name: &'static str) -> Self {
        self.file_name = Some(file_name.to_string());
        self
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn set_output_dir(mut self, output_dir: &str) -> Self {
        self.output_dir = PathBuf::from(output_dir);
        self
    }

    pub fn replace_header(mut self, header: HeaderMap) -> Self {
        self.header = header;
        self
    }

    pub fn insert_header(mut self, k: impl IntoHeaderName, v: &str) -> Self {
        self.header.insert(k, v.parse().unwrap());
        self
    }

    pub async fn download(self, url: &str) -> Result<Self, Box<dyn Error>> {
        let file_name = if self.file_name.is_some() {
            self.file_name.clone().unwrap()
        } else {
            self.get_last_segment_from_url(url)
        };

        let dir_exists = std::fs::metadata(&self.output_dir).is_ok();
        if !dir_exists {
            std::fs::create_dir_all(&self.output_dir)?;
        }

        let output_path = self.output_dir.join(file_name);

        let mut file = File::create(output_path)?;

        if self.verbose {
            println!("Downloading {}", url);
        }

        let mut stream = self
            .client
            .get(url)
            .headers(self.header.clone())
            .send()
            .await
            .unwrap()
            .bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            file.write_all(&chunk_result?)?;
        }

        file.flush()?;

        if self.verbose {
            println!("Downloaded {}", url);
        }

        Ok(self)
    }

    fn get_last_segment_from_url(&self, url: &str) -> String {
        url.split('/').last().unwrap().to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::downloader::*;
    use std::{fs, path::PathBuf};
    use sysinfo::{Pid, System};

    const URL: &'static str = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

    #[tokio::test]
    async fn should_download() {
        // make sure the file doesn't exist
        // rm -f the file
        let _ = fs::remove_file("rust-logo.png");

        Downloader::new()
            .file_name("rust-logo.png")
            .download(URL)
            .await
            .unwrap();

        // it should exist
        let exists = fs::metadata("rust-logo.png").is_ok();
        assert_eq!(exists, true);

        // remove the file
        let _ = fs::remove_file("rust-logo.png");
    }

    #[tokio::test]
    async fn should_create_dir_if_not_exists() {
        let dir = "temp/some/dir";
        let _ = fs::remove_dir_all(dir);

        Downloader::new()
            .set_output_dir(dir)
            .file_name("rust-logo.png")
            .download(URL)
            .await
            .unwrap();

        // it should exist
        let exists = fs::metadata(dir).is_ok();
        assert_eq!(exists, true);

        // remove the file
        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn large_file_download() {
        let file_path = PathBuf::from("temp/rust.zip");
        let _ = fs::remove_file(&file_path);

        let url = "https://github.com/rust-lang/rust/archive/refs/tags/1.77.2.zip";

        let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(1);
        let join = tokio::spawn(async move {
            // download the file
            Downloader::new()
                .set_output_dir("temp")
                .file_name("rust.zip")
                .verbose()
                .download(url)
                .await
                .unwrap();

            tx.send(true).await.unwrap();
        });
        let join2 = tokio::spawn(async move {
            loop {
                let current_memory = get_memory_usage();
                assert_eq!(current_memory <= 30.0, true);

                let finished = rx.try_recv().unwrap_or(false);
                if finished {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });

        join.await.unwrap();
        join2.await.unwrap();

        // file should exist
        let exists = fs::metadata(&file_path).is_ok();
        assert_eq!(exists, true);

        // remove the file
        let _ = fs::remove_file(&file_path);
    }

    /// Get memory usage of the current process
    /// Example usage:
    /// ```
    /// let memory_usage = get_memory_usage();
    /// println!("Memory usage: {} MB", memory_usage);
    /// ```
    /// Returns memory usage in megabytes
    ///
    /// Example Result:
    /// ```bash
    /// Memory usage: 123.456789 MB
    /// ```
    fn get_memory_usage() -> f64 {
        let mut system = System::new_all();
        system.refresh_all();

        // Get current process using its PID
        let current_pid = std::process::id();
        let process = system
            .process(Pid::from_u32(current_pid))
            .expect("process should be present");

        // Get memory usage in bytes and convert to megabytes
        process.memory() as f64 / 1024.0 / 1024.0
    }
}
