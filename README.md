# memflow LiME files connector

This is a connector for [memflow](https://github.com/memflow/memflow) that
targets [LiME](https://github.com/504ensicsLabs/LiME) dumps files.

At the moment of writing, the physical to virtual memory translation is not yet
implemented in memflow, therefore this is intended as for future usage.

To run the tests to check the correctness of physical memory parsing you can
use `cargo test`. A sample slice of a LiME dump is provided in the `./test`
folder and used in the tests.
