use std::fs::File;
use std::io::Read;
use memflow::prelude::{ConnectorArgs, PhysicalAddress, PhysicalMemory};
use memflow_lime::create_connector;

/// Compare a physical read done through memflow_lime with the output of
/// [Volatility3](https://github.com/volatilityfoundation/volatility3) on the same dump.
#[test]
fn read_physical_location() {
    let addr = PhysicalAddress::from(0x1000);
    let mut volatility_file = File::open("./tests/deb-x86_64-slice_0x1000_volatility3_out").unwrap();
    let mut volatility_output = [0u8; 128];
    volatility_file.read_exact(&mut volatility_output).unwrap();

    let args = ConnectorArgs::new(
        Some("./tests/deb-x86_64-slice.lime"),
        Default::default(),
        None,
    );
    let mut con = create_connector(&args).unwrap();

    let mut buff = [0u8; 128];
    con.phys_read_into(addr, &mut buff).unwrap();

    assert_eq!(buff, volatility_output);
}
