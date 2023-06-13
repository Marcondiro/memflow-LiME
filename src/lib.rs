use binread::{BinRead, BinReaderExt};

use memflow::connector::fileio::{CloneFile, FileIoMemory};
use memflow::prelude::v1::*;

use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Header defined by the `LiME` file format, version 1
///
/// source: [LiME Memory Range Header Version 1 Specification](https://github.com/504ensicsLabs/LiME/blob/master/doc/README.md#Spec)
#[derive(Debug, BinRead)]
#[br(magic = 0x4C69_4D45_u32)] //LiME
struct LimeHeader {
    /// Header version number
    #[br(assert(version == 1, "Unsupported LiME version: {}", version))]
    #[allow(dead_code)]
    version: u32,
    /// Starting address of physical RAM range
    s_addr: u64,
    /// Ending address of physical RAM range
    #[br(assert(e_addr >= s_addr, "End address can not be lower than start address"))]
    e_addr: u64,
    /// Currently all zeros
    #[br(assert(reserved == [0; 8], "Unsupported LiME reserved fields values"))]
    #[allow(dead_code)]
    reserved: [u8; 8],
}

impl LimeHeader {
    /// Size in bytes of `LimeHeader`
    const HEADER_SIZE_IN_BYTES: usize = 32;

    /// Get the `LiME` header from file.
    ///
    /// Returns `Ok(None)` if the End Of File is reached\
    /// Returns `Ok(Some(...))` if the `LimeHeader` is parsed correctly\
    ///
    /// # Arguments
    ///
    /// * `lime_dump` - file to read from, the seek of the file  must be already at the start of the header or at EOF.
    ///
    /// # Errors
    ///
    /// Returns `Err` if an error occurred while reading the file or parsing the header
    ///
    fn next_header_from_file(lime_dump: &mut File) -> Result<Option<LimeHeader>> {
        let mut buff = [0u8; LimeHeader::HEADER_SIZE_IN_BYTES];

        match lime_dump.read_exact(&mut buff) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(_) => Err(Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)),
            Ok(()) => {
                let header: LimeHeader = Cursor::new(&buff).read_le().map_err(|_| {
                    Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)
                        .log_error("Unable to parse the LiME file.")
                })?;

                Ok(Some(header))
            }
        }
    }

    /// Size in bytes of the memory represented by this header
    fn mem_section_size(&self) -> u64 {
        self.e_addr - self.s_addr + 1
    }
}

#[connector(name = "lime", help_fn = "help")]
pub fn create_connector(args: &ConnectorArgs) -> Result<FileIoMemory<CloneFile>> {
    let mut lime_dump = File::open(
        args.target
            .as_ref()
            .ok_or(
                Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)
                    .log_error("LiME file path not specified"),
            )?
            .as_ref(),
    )
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

    let mut map = MemoryMap::new();
    let mut offset = 0;

    while let Some(header) = LimeHeader::next_header_from_file(&mut lime_dump)? {
        offset += LimeHeader::HEADER_SIZE_IN_BYTES as u64;

        map.push_remap(
            header.s_addr.into(),
            header.mem_section_size(),
            offset.into(),
        );
        offset = lime_dump
            .seek(SeekFrom::Current(header.mem_section_size() as i64))
            .map_err(|_| {
                Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile)
                    .log_error("Corrupted LiME file")
            })?;
    }

    lime_dump.seek(SeekFrom::Start(0)).map_err(|_| {
        Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile)
            .log_error("Unable to seek back to the beginning of the file")
    })?;
    FileIoMemory::with_mem_map(lime_dump.into(), map)
}

/// Retrieve the help text for the `LiME` Connector.
pub fn help() -> String {
    "\
The `lime` connector implements the LiME file format parser.

The `target` argument specifies the filename of the file to be opened.
    "
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};

    #[test]
    fn unspecified_file_causes_error() {
        let connector_args = ConnectorArgs::default();
        let connector = create_connector(&connector_args);
        assert_eq!(
            connector.err().unwrap(),
            Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)
        );
    }

    #[test]
    fn header_parser_works() {
        let raw_header: [u8; LimeHeader::HEADER_SIZE_IN_BYTES] = [
            69, 77, 105, 76, 1, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 255, 255, 207, 251, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let tmp_file_path = "./test_header.tmp";
        let mut tmp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(tmp_file_path)
            .unwrap();

        tmp_file.write(&raw_header).unwrap();
        tmp_file.seek(SeekFrom::Start(0)).unwrap();

        let header = LimeHeader::next_header_from_file(&mut tmp_file)
            .unwrap()
            .unwrap();

        fs::remove_file(tmp_file_path).unwrap();

        assert_eq!(header.version, 1);
        assert_eq!(header.s_addr, 0x40000000);
        assert_eq!(header.e_addr, 0xFBD00000 - 1);
        assert_eq!(header.reserved, [0; 8]);
    }
}
