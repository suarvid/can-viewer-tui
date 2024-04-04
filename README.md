# Tool for viewing CAN-frames via the terminal

## Usage
To view can frames, run:
`can-viewer-tui <can-interface>`

Example usage:
`can-viewer-tui can0`

In order to view the names of available can interfaces, you can run `ip a` on unixy-systems.
It may be necessary to manually set up the can interface before use, which can be done
via the following command:
`sudo ip link set up <can-interface> type can bitrate <bitrate>`
where `<can-interface>` and `<bitrate>` should be replaced by sensible values, such as:
`sudo ip link set up can0 type can bitrate 500000`

