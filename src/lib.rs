use binread::{BinRead, BinReaderExt};

use memflow::prelude::v1::*;
use memflow::connector::fileio::{CloneFile, FileIoMemory};

use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Header defined by the LiME file format, version 1
///
/// https://github.com/504ensicsLabs/LiME/blob/master/doc/README.md#Spec
#[derive(Debug, BinRead)]
#[br(magic = 0x4C694D45u32)] //LiME
struct LimeHeader{
    /// Header version number
    #[br(assert(version == 1, "Unsupported LiME version: {}", version))]
    version: u32,
    /// Starting address of physical RAM range
    s_addr: u64,
    /// Ending address of physical RAM range
    #[br(assert(e_addr >= s_addr, "End address can not be lower than start address"))]
    e_addr: u64,
    /// Currently all zeros
    reserved: [u8; 8],
}

impl LimeHeader {
    /// Size in bytes of LimeHeader
    const fn header_size_in_bytes() -> u64 {
        32
    }

    fn next_header_from_file(lime_dump: &mut File) -> Result<LimeHeader> {
        //todo better error logging
        let mut buff = [0u8; LimeHeader::header_size_in_bytes() as usize];
        lime_dump.read_exact(&mut buff)
            .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

        let header:LimeHeader = (&mut Cursor::new(&buff)).read_le()
            .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

        // TODO warning if reserved != 0s
        Ok(header)
    }

    fn ram_section_size(&self) -> u64 {
        self.e_addr -self.s_addr + 1
    }
}

#[connector(name = "lime", help_fn="help")]
pub fn create_connector(args: &ConnectorArgs) -> Result<FileIoMemory<CloneFile>> {
    let mut lime_dump = File::open(
        args.target
            .as_ref()
            .ok_or(Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)
                .log_error("LiME file path not specified"))?
            .as_ref()
    ).map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

    let mut map = MemoryMap::new();
    let mut offset = 0;

    loop{
        let header = match LimeHeader::next_header_from_file(&mut lime_dump) {
            Ok(h) => h,
            Err(_) => break,
        };
        offset += LimeHeader::header_size_in_bytes();
        println!("{:?}", header);

        map.push_remap(header.s_addr.into(), header.ram_section_size(), offset.into());
        offset = lime_dump.seek(SeekFrom::Current(header.ram_section_size() as i64)).unwrap();
    }

    FileIoMemory::with_mem_map(lime_dump.into(), map)
}

pub fn help() -> String {
    todo!();
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    use super::*;

    #[test]
    fn unspecified_file_causes_error() {
        let connector_args = ConnectorArgs::default();
        let connector = create_connector(&connector_args);
        assert_eq!(connector.err().unwrap(), Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile));
    }

    #[test]
    fn header_parser_works() {
        let raw_header:[u8; LimeHeader::header_size_in_bytes() as usize] = [69, 77, 105, 76, 1, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 255, 255, 207, 251, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let tmp_file_path = "./test_header.tmp";
        let mut tmp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(tmp_file_path)
            .unwrap();

        tmp_file.write(&raw_header).unwrap();
        tmp_file.seek(SeekFrom::Start(0)).unwrap();

        let header = LimeHeader::next_header_from_file(&mut tmp_file).unwrap();

        fs::remove_file(tmp_file_path).unwrap();

        assert_eq!(header.version, 1);
        assert_eq!(header.s_addr, 0x40000000);
        assert_eq!(header.e_addr, 0xFBD00000-1);
        assert_eq!(header.reserved, [0;8]);
    }

    #[test]
    fn it_works() {
        let connector_args = ConnectorArgs::new(Some("deb-x86.lime"), Args::default(), None);
        let connector = create_connector(&connector_args).unwrap();
    }
}
