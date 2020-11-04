use libheif_rs::{ColorSpace, HeifContext, RgbChroma};
use png;

fn main() -> Result<(), Box<dyn Error>> {
    write_png("./data/image.png", "./data/test3.heic")
}

// For reading and opening files
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::time;

///
/// write_png writes the image of the `path_input` inside `path_output`
///
pub fn write_png<P: AsRef<Path>>(path_output: P, path_input: P) -> Result<(), Box<dyn Error>> {
    let heig_ctx = HeifContext::read_from_file(path_input.as_ref().to_str().unwrap())?;
    // Writers to image png
    let buffer = File::create(path_output)?;
    let w = &mut BufWriter::new(buffer);
    // heig handlers
    let handle = heig_ctx.primary_image_handle()?;
    let image_heif = handle.decode(ColorSpace::Rgb(RgbChroma::Rgba), false)?;
    let heic_plane = image_heif.planes().interleaved.unwrap();
    let target_size = heic_plane.width * 4 * heic_plane.height;
    println!(
        "Width: {}, Height: {}, Target Size: {}, Actual HEIC Buffer Size: {}",
        heic_plane.width,
        heic_plane.height,
        target_size,
        heic_plane.data.len(),
    );

    // Png Encoder
    let mut encoder = png::Encoder::new(w, heic_plane.width, heic_plane.height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_filter(png::FilterType::NoFilter);
    let time_start = time::SystemTime::now();
    // create png writer
    let mut writer = encoder.write_header().unwrap();
    let chunk_size = if heic_plane.data.len() != target_size as usize {
        heic_plane.data.len() / heic_plane.height as usize
    } else {
        target_size as usize
    };
    let offset = if chunk_size == target_size as usize {
        0
    } else {
        4
    };
    if offset == 0 {
        write_all(heic_plane.data, writer)?;
    } else {
        stream_all_diff(
            &heic_plane.data,
            // The chunks are like this don't ask why...
            writer.stream_writer_with_size(chunk_size),
            chunk_size,
            offset,
        )?;
    };
    println!(
        "time passed: {}",
        time::SystemTime::now()
            .duration_since(time_start)
            .unwrap()
            .as_millis()
    );

    Ok(())
}

fn stream_all_diff<T: Write>(
    data: &[u8],
    mut writer: T,
    chunk_size: usize,
    offset_chunk: usize,
) -> Result<(), String> {
    for chunk in data.chunks(chunk_size) {
        if let Err(err) = writer.write(&chunk[..chunk.len().min(chunk_size - offset_chunk)]) {
            return Err(err.to_string());
        }
    }
    Ok(())
}

fn write_all<T: Write>(data: &[u8], mut writer: png::Writer<T>) -> std::io::Result<()> {
    writer.stream_writer().write_all(&data)
}

#[cfg(test)]
mod tests {
    use super::write_png;
    #[test]
    fn check_that_your_data_dir_works() {
        use threadpool::ThreadPool;
        let pool = ThreadPool::new(6);
        let directory_input = "./data/input";
        for dir in std::fs::read_dir(directory_input).unwrap() {
            pool.execute(|| {
                let directory_output = "./data/output";
                let dir = dir.unwrap();
                let string: String = dir.path().into_os_string().into_string().unwrap();
                let split: Vec<&str> = string.split(".").collect();
                let path_without_extension = split[split.len() - 2].to_string();
                let file_name = path_without_extension.split("/").last().unwrap();
                let extension = split.last().unwrap().to_lowercase();
                if extension == "heic" {
                    if let Err(err) = write_png(
                        format!("{}/{}.png", directory_output, file_name).to_string(),
                        format!(".{}.heic", path_without_extension).to_string(),
                    ) {
                        panic!(err.to_string());
                    }
                    println!("finished");
                }
            });
        }
        pool.join();
    }
}

// // Get Exif
// let meta_ids = handle.list_of_metadata_block_ids("Exif", 1);
// assert_eq!(meta_ids.len(), 1);
// let exif: Vec<u8> = handle.metadata(meta_ids[0])?;

// // Decode the image
// let image = handle.decode(ColorSpace::Rgb(RgbChroma::Rgb), false)?;
// assert_eq!(image.color_space(), Some(ColorSpace::Rgb(RgbChroma::Rgb)));

// // Scale the image
// let small_img = image.scale(1024, 800, None)?;
// assert_eq!(small_img.width(Channel::Interleaved)?, 1024);
// assert_eq!(small_img.height(Channel::Interleaved)?, 800);

// // Get "pixels"
// let planes = small_img.planes();
// let interleaved_plane = planes.interleaved.unwrap();
// assert_eq!(interleaved_plane.width, 1024);
// assert_eq!(interleaved_plane.height, 800);
// assert!(!interleaved_plane.data.is_empty());
// assert!(interleaved_plane.stride > 0);
// println!("{:?}", interleaved_plane.data[0]);
// Ok(())

// fn stream_all<T: Write>(data: &[u8], size: usize, mut writer: png::StreamWriter<T>) {
//     let mut i = 0;
//     let mut initial = 0;
//     let mut times_that_it_happens = 0;
//     while i < data[0..40000].len() {
//         let byte = data[i];
//         let next_byte = if i + 1 < data.len() { data[i + 1] } else { 0 };
//         let next_next_byte = if i + 2 < data.len() { data[i + 2] } else { 0 };
//         let next_next_next_byte = if i + 3 < data.len() { data[i + 3] } else { 0 };
//         if byte == 0 && next_byte == 0 && next_next_byte == 0 && next_next_next_byte == 0 {
//             // println!("{} {} {}", initial, i, i - initial);
//             writer.write(&data[initial..i]).unwrap();
//             times_that_it_happens += 1;
//             i += 4;
//             initial = i;
//             continue;
//         }
//         i += 1;
//     }
//     println!("{}", times_that_it_happens);
// }
