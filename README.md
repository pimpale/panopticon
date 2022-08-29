# Panopticon

This command line application takes periodic screenshots of your screen, and saves them in the folder specified.

### Platforms

As far as I know, this should be supported on Mac, Windows, and Linux (X11).
Linux with Wayland is not supported since the Wayland protocol is much more restrictive when it comes to applications attempting to access information outside of their window.

### Dependencies

On Ubuntu, the key dependencies can be installed with `apt-get install libxcb1 libxrandr2 libdbus-1-3`.
(See here for details: https://github.com/nashaofu/screenshots-rs)

Additionally, Rust is required to build it.

### Usage

```
panopticon 0.1
Govind Pimpale <gpimpale29@gmail.com>
Takes periodic screenshots

USAGE:
    panopticon [OPTIONS] --dir <DIR>

OPTIONS:
    -a, --afk-threshold <AFK_THRESHOLD>
            Duration in seconds of no mouse or keyboard activity after which the user will be
            considered AFK [default: 60]

    -d, --dir <DIR>
            Directory to store screenshots in

    -h, --help
            Print help information

    -i, --interval <INTERVAL>
            Interval in seconds between screenshots [default: 60]

    -j, --jitter <JITTER>
            Seconds of jitter to add to the screenshot time. Must be less than or equal to interval.
            [default: 0]

    -n, --no-afk
            Don't check whether the user is afk or not

    -V, --version
            Print version information```
