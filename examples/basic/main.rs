//!
//! To be run with basic.sh

use std::io::Read;

use ambients::FdName;

pub fn main() {
    let mut fds = unsafe { ambients::get_ambient_fds() };

    assert!(fds.len() == 1, "One FD is required, got {fds:?}");
    let fd = fds.pop().unwrap();

    assert!(
        matches!(fd.get_name(), FdName::Name(_)),
        "FD name should be 'basic', found {fd:?}"
    );

    let fd = fd.into_fd();

    let fd = fd.into_fd();
    let owned = fd.try_clone_to_owned().unwrap();

    let mut file = std::fs::File::from(owned);

    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();

    assert!(
        output.as_str() == include_str!("./basic.txt"),
        "Basic FD must match contents in basic.txt, got\n{output:?}"
    );
    println!("{}", output);
}
