// https://github.com/systemd/systemd/blob/65537f059c24bcaba6e70afb7767e67be62b5738/src/systemd/sd-daemon.h#L54
// This is part of the public API and people hardcode this, so I would be surprised if it changed.
// ferris clueless
const SD_LISTEN_FDS_START: u16 = 3;

pub struct AmbientFd {
    name: FdName,
    fd: FdKind
}

/// Represents the name and context of what the name means.
pub enum FdName {
    // Name provided.
    Name(Box<str>),
    /// Was in systemd's FD store, but no name was provided.
    Stored,
    /// No name was received for this FD, and was not in the FD store.
    Unknown,
    /// Activated via `Accept=yes` in a systemd unit file
    /// This is the connection.
    Connection
}

/// Possible types of FDs received
pub FdKind {
    
}

pub fn get_ambient_fds() -> Vec<AmbientFd> {
    let Ok(pid) = std::env::var("LISTEN_PID") else {
        // Nothing to get
        return Vec::new();
    };
    let Ok(pid) = pid.parse::<u32>() else {
        panic!("The PID provided was unable to be parsed as an integer.")
    };
    if pid != std::process::id() {
        // There are FDs, but not for us.
        return Vec::new();
    }

    let Ok(fd_count) = std::env::var("LISTEN_FDS") else {
        panic!("Unable to get the list of FDs, even though they should be set.")
    };

    // This is how we know what to try and open.
    let fd_count = match fd_count.parse::<usize>() {
        Ok(fd_count) => fd_count,
        Err(e) => panic!("Unable to parse FD count as integer.\n{e:?}"),
    };

    let raw_names = match std::env::var("LISTEN_FDNAMES") {
        Ok(names) => names,
        _ => panic!("Unable to get FD names, systemd should always set this if the rest is true."),
    };
    let names = raw_names.split(":");

    (SD_LISTEN_FDS_START..)
        .skip(1)
        .take(fd_count)
        .zip(names.into_iter())
        .map(|(raw_fd, name)| AmbientFd {
            name: match name {
                "connection" => FdName::Connection,
                "stored" => FdName::Stored,
                "unknown" => FdName::Unknown,
                name => FdName::Name(name.clone().into())

            },
            fd: 
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
