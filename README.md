# dload

`dload` is a simple crate to help downloading files from the internet easily. It is a simple wrapper around the `reqwest` crate.

## Usage

Add it to your package:

```bash
cargo add dload
```

Then you can use it like this:

```rust
use dload::Downloader;

fn main() {
  let url = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

  Downloader::new()
    .set_output_dir(dir)
    .file_name("rust-logo.png")
    .download(url)
    .await
    .unwrap();
}
```

This will download the file from the given URL and save it to the given directory with the given file name.
