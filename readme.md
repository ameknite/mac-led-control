# mac-led-control

A tiny command-line tool to control the front status LED on a Mac mini.

macOS does not expose any setting to turn off the front LED, so it stays lit
(or pulses) whenever the machine is powered on or sleeping. This tool talks
directly to the **System Management Controller (SMC)** to write the `LSOO`
key, which controls the LED state.

## Requirements

- macOS (tested on recent versions; uses IOKit, so it is macOS-only)
- **Root privileges** — writing to the SMC is a privileged operation
- Apple silicon and Intel Mac minis are both supported in principle, though
  the `LSOO` key behavior may vary across models and macOS releases

## Installation

```sh
cargo install mac-led-control
```

The binary will be placed in `~/.cargo/bin/mac-led-control`.

## Usage

```sh
sudo mac-led-control off   # turn the LED off
sudo mac-led-control on    # restore the LED to its normal state
```

`sudo` is required. Without it, the SMC write is rejected by the kernel with
`kIOReturnNotPermitted`, and the tool exits with an error.

Example:

```sh
$ sudo mac-led-control off
LED turned off.

$ sudo mac-led-control on
LED turned on.
```

Help output:

```sh
$ mac-led-control --help
Control the front status LED on a Mac mini

Usage: mac-led-control <COMMAND>

Commands:
  on   Turn the LED on
  off  Turn the LED off
  help Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Running as a daemon on macOS

Because the SMC resets the `LSOO` key on every boot, the LED will be back on
after a restart. To keep it off automatically, install a **LaunchDaemon** —
macOS's built-in mechanism for running a program as `root` at boot time,
before any user logs in. This is the correct place for a privileged,
boot-time hardware tweak (a user-level `LaunchAgent` will not work, because
it runs as your user and cannot write to the SMC).

### 1. Create the plist

```sh
sudo nano /Library/LaunchDaemons/com.user.macledcontrol.plist
```

Paste the following, replacing the path with the output of
`which mac-led-control` (typically `/Users/<you>/.cargo/bin/mac-led-control`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.macledcontrol</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/you/.cargo/bin/mac-led-control</string>
        <string>off</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

### 2. Set ownership and permissions

launchd refuses to load plists that are writable by non-root users.

```sh
sudo chown root:wheel /Library/LaunchDaemons/com.user.macledcontrol.plist
sudo chmod 644 /Library/LaunchDaemons/com.user.macledcontrol.plist
```

### 3. Validate and load

```sh
plutil -lint /Library/LaunchDaemons/com.user.macledcontrol.plist
sudo launchctl load -w /Library/LaunchDaemons/com.user.macledcontrol.plist
```

The LED should turn off immediately. It will also turn off on every
subsequent boot, without you having to run anything.

### 4. Verify it ran

```sh
sudo launchctl list | grep macledcontrol
```

Expected output:

```
-    0    com.user.macledcontrol
```

- `-` means the process is not currently running (normal: it did its job
  and exited within milliseconds).
- `0` means the last exit status was success. A non-zero number indicates
  the daemon failed — check the log (see below).

### 5. Remove the daemon

```sh
sudo launchctl unload /Library/LaunchDaemons/com.user.macledcontrol.plist
sudo rm /Library/LaunchDaemons/com.user.macledcontrol.plist
sudo mac-led-control on   # restore the LED right now
```

### Troubleshooting the daemon

If the LED stays on after loading, or if the daemon stops working after a
macOS update, inspect the unified log:

```sh
log show --predicate 'process == "mac-led-control"' --last 1h
```

A panic message (e.g. `called Result::unwrap() on an Err value: ...`) almost
always means the SMC interface changed in a way the current version of
`smc-lib` does not handle. Updating the dependency and reinstalling the
binary usually fixes it.

## How it works

The tool uses the [`smc-lib`](https://crates.io/crates/smc-lib) crate to
open a connection to the `AppleSMC` kernel service through IOKit, then
writes `0x00` (off) or `0x01` (on) to the SMC key `LSOO`. Because this is a
direct hardware write rather than a supported macOS API, a major macOS
update can break it. In that case, check for a newer version of `smc-lib`.

## Caveats

- **Requires root.** The SMC is a privileged interface; there is no way
  around this.
- **Not persistent on its own.** The SMC resets `LSOO` on reboot — use the
  LaunchDaemon setup above if you want persistence.
- **Not an official Apple API.** This relies on reverse-engineered SMC
  behavior and can break with macOS or firmware updates.
- **Use at your own risk.** Writing incorrect values to SMC keys can cause
  system instability. This tool only writes to `LSOO`, which is the
  documented-in-community key for the front LED.

## License

Licensed under the [Mozilla Public License 2.0](LICENSE).
