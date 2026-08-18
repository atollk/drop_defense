#!/bin/bash
(cd rust && cargo build --package drop_defense --lib)
pkill -f "lldb-server gdbserver localhost:9999" 2>/dev/null  # clean up any stale session
lldb-server gdbserver localhost:9999 -- $(which godot) -d --path ./godot > /tmp/lldb-server.log 2>&1 &
disown
sleep 1   # give lldb-server a moment to bind the port before RustRover tries to connect