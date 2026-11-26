"""Exercise the shipping binary over a real POSIX PTY, including its original mode."""
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time


def resize(slave, rows, cols):
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def until(master, needle):
    received = bytearray()
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        if select.select([master], [], [], 0.1)[0]:
            received.extend(os.read(master, 65536))
            if needle in received:
                return
    raise AssertionError((needle, received[-512:]))


def check(binary, ending, state_dir):
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    resize(slave, 32, 120)
    proc = subprocess.Popen([binary, "--mute"], stdin=slave, stdout=slave, stderr=slave,
                            env={**os.environ, "XDG_STATE_HOME": state_dir})
    try:
        until(master, b"SPACE to run")
        assert not termios.tcgetattr(slave)[3] & termios.ECHO
        # Split an enhanced press across reads, then release and pause.
        os.write(master, b"\x1b[?11u\x1b[11")
        time.sleep(.04)
        os.write(master, b"9;1:1u")
        until(master, b"SPACE jump")
        os.write(master, b"\x1b[119;1:3up")
        until(master, b"Paused")
        resize(slave, 12, 70)
        until(master, b"Resize to")
        resize(slave, 28, 100)
        until(master, b"Paused")
        if isinstance(ending, bytes):
            os.write(master, ending)
        else:
            proc.send_signal(ending)
        until(master, b"\x1b[?1049l")
        assert proc.wait(timeout=5) == 0
        assert termios.tcgetattr(slave) == original
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as state_dir:
        for ending in [b"q", b"\x03", signal.SIGTERM]:
            check(sys.argv[1], ending, state_dir)
    print("TTY press, release, resize, exit and mode restoration passed.")
