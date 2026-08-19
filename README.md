# Ambients

Use systemd's [OpenFile](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html#OpenFile=) and [FD Store](https://systemd.io/FILE_DESCRIPTOR_STORE/) easily.
For restart tolerance the FD store may be of significant interest as you can persist FDs across service starts, and even `kexec` events.

## Why?

Capabilities are cool. In theory what you can do with this library is to make a systemd service in a namespace with zero files, and zero networking.
Then you just have systemd insert any FDs required at start-up. So add you config file, your sqlite database, your postgres unix socket, your IPv4 socket,
whatever via FDs, then the program is quite secure since the only things it can possibly access are the FDs provided.

Most programs do not support FDs for their config files. But they should.

## Why is the API unsafe?

Technically you can just set the environment variables that systemd is supposed to set to whatever nonsense you want. Don't do that, and you are good to go.

## Systemd Sandbox
Here's an example of a sandboxed systemd unit file. It is placed within an empty mount namespace, and has as many thing unshared as possible.
The syscalls can probably be restricted further to deny file reading and writing if deired, but since the namespaces is ephemeral and empty, it doesn't matter much.

```ini
[Unit]
Description=CoolService 

[Service]
Type=simple
ExecStart=CoolService
Group=CoolService
User=CoolService
CapabilityBoundingSet=
LockPersonality=true
NoNewPrivileges=true
PrivateDevices=true
PrivateIPC=true
PrivateMounts=true
PrivatePIDs=true
PrivateTmp=true
PrivateUsers=true
ProtectClock=true
ProtectControlGroup=true
ProtectHome=true
ProtectHostname=true
ProtectKernelLogs=true
ProtectKernelModules=true
ProtectKernelTunables=true
ProtectProc=invisible
ProtectSystem=strict
ReadWritePaths=
RemoveIPC=true
Restart=on-failure
RestrictNamespaces=true
RestrictRealtime=true
RestrictAddressFamilies=none
SocketBindDeny=deny
SystemCallArchitectures=native
SystemCallErrorNumber=EPERM
SystemCallFilter=@system-service
WorkingDirectory=/

# This creates a mount namespaces that is empty within a tmpfs
# This tmpfs is directly tied to the service lifetime. So when
# the service stop, the tmpfs is destroyed. All persistant files
# are provided via OpenFile
RootDirectory=/run/CoolService
# This tells systemd to actually create `/run/CoolService`
RuntimeDirectory=CoolService 

# Add files to inject via FD at startup. The process doesn't
# even need proper user/group permissions to access the file.
OpenFile=/var/config/CoolService.json:config:read-only

# FD for a postgres UDS
OpenFile=/run/postgres/server.sock:database

# For sockets, use an associated .socket unit file.
# There is a lot of documentation that can explain this better
# than I elsewhere.
 
[Install]
WantedBy=multi-user.target
```

## TODO

* Make more tests/examples
