use image::{self, GenericImage, ImageBuffer};
use pbr::ProgressBar;

use std::path::{PathBuf, Path};

use ::Direction;
use ::errors::{ErrorKind, Result, ResultExt};

fn generage_subimage_coords(bounds: (u32, u32, u32, u32),
                            index: u32,
                            direction: Direction)
                            -> Option<(u32, u32, u32, u32)> {
    let (bx, by, bw, bh) = bounds;
    match direction {
        Direction::N => {
            // N -> S
            if index >= bh {
                return None;
            }
            Some((bx, by + index, bw, 1))
        }
        Direction::S => {
            // S -> N
            if index >= bh {
                return None;
            }
            Some((bx, by + bh - index - 1, bw, 1))
        }
        Direction::W => {
            // W -> E
            if index >= bw {
                return None;
            }
            Some((bx + index, by, 1, bh))
        }
        Direction::E => {
            // E -> W
            if index >= bw {
                return None;
            }
            Some((bx + bw - index - 1, by, 1, bh))
        }
    }
}

fn process_image<I, J>(current_buffer: &mut I,
                       image: &mut J,
                       index: usize,
                       direction: Direction)
                       -> Result<bool>
    where I: GenericImage,
          I::Pixel: 'static,
          J: GenericImage<Pixel = I::Pixel> + 'static
{
    if let Some((x, y, width, height)) = generage_subimage_coords(image.bounds(),
                                                                  index as u32,
                                                                  direction) {
        let subimage = image.sub_image(x, y, width, height);
        Ok(current_buffer.copy_from(&subimage, x, y))
    } else {
        Ok(false)
    }
}

/// Given a set of image paths, an output path, and a shutter direction, generate an output image.
///
/// # Arguments
/// * `paths` - An iterator of `PathBuf`s that describe the input images, in the correct order.
/// * `output` - The output image path.
/// * `direction` - The direction from which the shutter *starts* moving.
/// * `suppress_output` - Whether to suppress output or not.
///
/// # Errors
/// This may fail if an individual image cannot be opened or processed, or if the output cannot be
/// properly saved.
pub(crate) fn process_images<I, P>(paths: I,
                                   output: P,
                                   direction: Direction,
                                   suppress_output: bool)
                                   -> Result<()>
    where I: Iterator<Item = PathBuf> + ExactSizeIterator,
          P: AsRef<Path>
{
    let mut iter = paths.peekable();

    let count = iter.len() as u64;

    // Note that we can access the first item without checking because we already ensured that only
    // non-empty sets of paths will be allowed in.
    let first_path = iter.peek().unwrap().clone();
    let mut cur_img =
        image::open(&first_path).chain_err(|| ErrorKind::CouldNotOpenImage(first_path.clone()))?;
    let (width, height) = cur_img.dimensions();
    let mut buf: image::RgbaImage = ImageBuffer::new(width, height);

    let num_frames = ::std::cmp::min(count,
                                     match direction {
                                         Direction::N | Direction::S => height,
                                         Direction::E | Direction::W => width,
                                     } as u64);
    let mut frame_pb = ProgressBar::new(num_frames);
    frame_pb.message("Processing frames: ");
    frame_pb.set_max_refresh_rate(Some(::std::time::Duration::from_millis(50)));

    for (i, path) in iter.enumerate() {
        if i > 0 {
            cur_img = image::open(&path).chain_err(|| ErrorKind::CouldNotOpenImage(path.clone()))?;
        }
        let process_result = process_image(&mut buf, &mut cur_img, i, direction)
            .chain_err(|| ErrorKind::CouldNotProcessImage(path.clone()))?;
        if process_result {
            // This is sort of an arbitrary number at which to show progress. Could probably turn
            // this into a progress bar sort of thing.
            if !suppress_output {
                frame_pb.inc();
            }
        } else {
            // Ran out of space to do shutters, so don't continue.
            break;
        }
    }

    if !suppress_output {
        frame_pb.finish();
    }

    let output = output.as_ref();

    buf.save(output).chain_err(|| ErrorKind::CouldNotSaveOutput(output.to_path_buf().clone()))?;
    if !suppress_output {
        println!("\nDone.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::Direction;

    #[test]
    fn test_subimage_coords() {
        let x = 3u32;
        let y = 4u32;
        let width = 640u32;
        let height = 480u32;
        let bounds = (x, y, width, height);

        assert_eq!(generage_subimage_coords(bounds, 0, Direction::N),
                   Some((x, y, width, 1)));
        assert_eq!(generage_subimage_coords(bounds, 30, Direction::N),
                   Some((x, y + 30, width, 1)));
        assert_eq!(generage_subimage_coords(bounds, height + 5, Direction::N),
                   None);

        assert_eq!(generage_subimage_coords(bounds, 0, Direction::S),
                   Some((x, y + height - 1, width, 1)));
        assert_eq!(generage_subimage_coords(bounds, 30, Direction::S),
                   Some((x, y + height - 30 - 1, width, 1)));
        assert_eq!(generage_subimage_coords(bounds, height + 5, Direction::S),
                   None);

        assert_eq!(generage_subimage_coords(bounds, 0, Direction::W),
                   Some((x, y, 1, height)));
        assert_eq!(generage_subimage_coords(bounds, 30, Direction::W),
                   Some((x + 30, y, 1, height)));
        assert_eq!(generage_subimage_coords(bounds, width + 5, Direction::W),
                   None);

        assert_eq!(generage_subimage_coords(bounds, 0, Direction::E),
                   Some((x + width - 1, y, 1, height)));
        assert_eq!(generage_subimage_coords(bounds, 30, Direction::E),
                   Some((x + width - 30 - 1, y, 1, height)));
        assert_eq!(generage_subimage_coords(bounds, width + 5, Direction::E),
                   None);
    }
}
