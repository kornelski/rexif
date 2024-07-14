# rexif

RExif is a native [Rust](https://www.rust-lang.org/) crate, written to extract EXIF data from JPEG and TIFF images.

It can be used as a library, or as a command-line tool. The sample binary called 'rexiftool' accepts files as arguments and prints the EXIF data. It gives
a rough idea on how to use the crate.

## Requirements

* Latest stable Rust version from [rustup](https://rustup.rs/).

## Example

```rust
match rexif::parse_file(&file_name) {
    Ok(exif) => {
        println!("{} {} exif entries: {}", file_name,
            exif.mime, exif.entries.len());

        for entry in &exif.entries {
            println!("    {}: {}",
                    entry.tag,
                    entry.value_more_readable);
        }
    },
    Err(e) => {
        eprintln!("Error in {}: {} {}", &file_name,
            Error::description(&e), e.extra).unwrap();
    }
}
```

The `src/main.rs` file is a good starting point to learn how to use the crate,
then take a look into the `ExifEntry` struct.
