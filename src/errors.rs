use image;

use std::path::PathBuf;

error_chain! {
    foreign_links {
        Image(image::ImageError);
        Io(::std::io::Error);
    }

    errors {
        Unimplemented {
            description("unimplemented")
            display("The desired operation on line {} of {} is unimplemented.", line!(), file!())
        }
        CouldNotOpenImage(filename: PathBuf) {
            description("could not open image")
            display("Could not open image {}.", filename.display())
        }
        CouldNotProcessImage(filename: PathBuf) {
            description("could not process image")
            display("Could not process image {}.", filename.display())
        }
        CouldNotSaveOutput(filename: PathBuf) {
            description("could not save image")
            display("Could not save image {}.", filename.display())
        }
        CouldNotParseFilemask(mask: String) {
            description("could not parse file mask")
            display("Could not parse file mask '{}'.", mask)
        }
        CouldNotGetPaths {
            description("could not get file paths")
            display("Could not get file paths to process.")
        }
        NoFileMaskFound {
            description("could not find file mask")
            display("Could not find file mask.")
        }
        NoFilesFound {
            description("could not find any files")
            display("Could not find any files with the provided file mask or folder.")
        }
        MultipleFileMasks {
            description("too many file masks")
            display("Only one sequential file mask variable is allowed.")
        }
    }
}
