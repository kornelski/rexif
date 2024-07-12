# rexif

RExif is a native [Rust](https://www.rust-lang.org/) crate, written to extract EXIF data from JPEG and TIFF images.

The crate also contains a sample binary called 'rexiftool' that accepts files as arguments and prints the EXIF data. It gives
a rough idea on how to use the crate.

I am still filling in
the implementation of most EXIF tags. Merge requests, comments and criticisms about coding style, information
about uncovered EXIF tags, sample images that are not parsed correctly -- in short, any sort of feedback is
welcome!

## Requirements

* Latest stable Rust version (1.39 currently)

## Example

```
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
        print!("Error in {}: {} {}", &file_name,
            Error::description(&e), e.extra).unwrap();
    }
}
```

The included tool `refixtool` accepts image file names as command-line
parameters and prints EXIF data for them. The `src/main.rs` file is a
good starting point to learn how to use the crate, then take a look into
the `ExifEntry` struct.
