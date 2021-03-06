use regex::Regex;

use std::path::PathBuf;
use std::str;

use ::errors::{ErrorKind, Result, ResultExt};

/// A file mask variable description.
///
/// Encodes whether the variable is zero-padded, and how many digits the full variable would have.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileMask {
    zero_padded: bool,
    digits: usize,
}

/// A description of what the path provided actually means; is the path the user provided a folder,
/// or is it a file mask of some kind?
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum PathMode<'a> {
    /// A file mask with a variable.
    FileMask(&'a str),
    /// A folder path.
    Folder(&'a str),
}

/// Parse a given file mask.
///
/// In essence, this means finding the sequence variable (i.e. the portion of the path that goes
/// 0, 1, ...) and taking it out so we can vary over it.
///
/// # Arguments
/// * `s` - The string describing the file mask.
///
/// # Errors
/// This will fail if there is not *exactly* one variable in the file mask. No variables means there
/// is no sequence to be used, and multiple means ambiguity in which sequence variable to use.
pub(crate) fn parse_filemask<S: Into<String>>(s: S) -> Result<(String, FileMask, String)> {
    let s = s.into();
    // This matches something of the format %Nd or %0Nd where N is the number of digits in the mask.
    let re = Regex::new(r"%(0)?([\d]+)d").unwrap();
    let mut result: Option<(String, FileMask, String)> = None;

    for cap in re.captures_iter(&s) {
        if result.is_some() {
            bail!(ErrorKind::MultipleFileMasks);
        }
        let mask = FileMask {
            zero_padded: cap.get(1).is_some(),
            digits: cap.get(2)
                .unwrap()
                .as_str()
                .parse()
                .unwrap(),
        };

        let mat = cap.get(0).unwrap();
        result = Some(((&s[..mat.start()]).into(), mask, (&s[mat.end()..]).into()));
    }

    match result {
        Some(result) => Ok(result),
        None => bail!(ErrorKind::NoFileMaskFound),
    }
}

/// Given a `PathMode`, retrieve the set of image paths.
///
/// # Caveats
/// There is no guarantee about the order of image paths provided using the `PathMode::Folder` mode.
/// The returned order will likely be system-dependent.
///
/// # Arguments
/// * `path_mode` - The `PathMode` describing how to determine the image paths.
///
/// # Errors
/// This can fail if given a `PathMode::FileMask` that cannot be parsed, or if there are no images
/// that exist in the sequence the file mask provides. If a folder is provided, this will fail if
/// the folder does not exist or if there are no images in that directory.
pub fn get_paths(path_mode: &PathMode) -> Result<Vec<PathBuf>> {
    match *path_mode {
        PathMode::FileMask(filemask) => {
            let (left, mask, right) = parse_filemask(filemask)
                .chain_err(|| ErrorKind::CouldNotParseFilemask(filemask.into()))?;

            let mut paths = vec![];
            let width = if mask.zero_padded { mask.digits } else { 0 };
            for i in 0..10u32.pow(mask.digits as u32) {
                let filename = format!("{}{:0width$}{}", left, i, right, width = width);
                let buf: PathBuf = filename.into();

                if !buf.exists() {
                    if !paths.is_empty() {
                        // If we already started, then let's just end (i.e. we want the first actual
                        // sequence of files).
                        break;
                    }
                    // Otherwise do nothing
                } else {
                    paths.push(buf);
                }
            }

            if paths.is_empty() {
                bail!(ErrorKind::NoFilesFound);
            }

            Ok(paths)
        }
        PathMode::Folder(_folder) => {
            // TODO
            bail!(ErrorKind::Unimplemented)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ::errors::*;

    #[test]
    fn test_parse_filemask() {
        let input1 = "foo%03d.png";
        let input2 = "foo%5d.jpg";
        let input3 = "foo.png";
        let input4 = "foo%02dbar%4d.png";

        let expected1 = ("foo".into(),
                         FileMask {
            zero_padded: true,
            digits: 3,
        },
                         ".png".into());

        let expected2 = ("foo".into(),
                         FileMask {
            zero_padded: false,
            digits: 5,
        },
                         ".jpg".into());

        assert_eq!(expected1, parse_filemask(input1).unwrap());
        assert_eq!(expected2, parse_filemask(input2).unwrap());
        match parse_filemask(input3) {
            Err(Error(ErrorKind::NoFileMaskFound, _)) => (),
            Err(e) => {
                assert!(false,
                        "expected `{}`, got `{}` instead.",
                        ErrorKind::NoFileMaskFound.description(),
                        e.description())
            }
            Ok(result) => {
                assert!(false,
                        "expected no file mask found error, but parsing succeeded with {:?}.",
                        result)
            }
        }
        match parse_filemask(input4) {
            Err(Error(ErrorKind::MultipleFileMasks, _)) => (),
            Err(Error(e, _)) => {
                assert!(false,
                        "expected `{}`, got `{}` instead.",
                        ErrorKind::MultipleFileMasks.description(),
                        e.description())
            }
            Ok(result) => {
                assert!(false,
                        "expected no file mask found error, but parsing succeeded with {:?}.",
                        result)
            }
        }
    }
}
