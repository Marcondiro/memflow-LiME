use binread::BinRead;

use memflow::prelude::v1::*;
use memflow::mem::MemoryMap;
use memflow::connector::fileio::{CloneFile, FileIoMemory};

use std::fs::File;
use std::io::{Cursor, Read};

/// Header defined by the LiME file format
///
/// https://github.com/504ensicsLabs/LiME/blob/master/doc/README.md#Spec
#[derive(Debug, BinRead)]
#[br(magic = b"LiME")]
struct LimeHeader{
    /// Header version number
    version: u32,
    /// Starting address of physical RAM range
    s_addr: u64,
    /// Ending address of physical RAM range
    e_addr: u64,
    /// Currently all zeros
    reserved: [u8; 8],
}

impl LimeHeader {
    const fn size_in_bytes() -> usize {
        32
    }
}

#[connector(name = "lime", help_fn="help")]
pub fn create_connector(args: &ConnectorArgs) -> Result<FileIoMemory<CloneFile>> {
    let mut dump_file = File::open(
        args.target
            .as_ref()
            .ok_or(Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile)
                .log_error("LiME file path not specified"))?
            .as_ref()
    ).map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

    let mut buff = [0u8; LimeHeader::size_in_bytes()];
    dump_file.read_exact(&mut buff)
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

    let header = LimeHeader::read(&mut Cursor::new(&buff))
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

    println!("{:?}", header);

    // mem.read(&mut head).ok();

    // let mut map = MemoryMap::new();

    // FileIoMemory::with_mem_map(mem.into(), map)
    Err(Error(ErrorOrigin::Connector, ErrorKind::Unknown))
}

pub fn help() -> String {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unspecified_file_causes_error() {
        let connector_args = ConnectorArgs::default();
        let connector = create_connector(&connector_args);
        assert_eq!(connector.err().unwrap(), Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile));
    }
}
