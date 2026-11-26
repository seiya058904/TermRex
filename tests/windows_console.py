"""Exercise the packaged Windows game in a resizable ConPTY (Python standard library)."""
import ctypes as c
from ctypes import wintypes as w
import msvcrt
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time

kernel = c.WinDLL("kernel32", use_last_error=True)
pointer = c.c_void_p
size = c.c_size_t


def api(name, restype, *arguments):
    call = getattr(kernel, name)
    call.restype = restype
    call.argtypes = arguments
    return call


class Startup(c.Structure):
    _fields_ = [("base", c.c_byte * (104 if c.sizeof(pointer) == 8 else 68)),
                ("attributes", pointer)]


class Process(c.Structure):
    _fields_ = [("process", w.HANDLE), ("thread", w.HANDLE), ("pid", w.DWORD), ("tid", w.DWORD)]


create_console = api("CreatePseudoConsole", c.c_long, w.DWORD, w.HANDLE, w.HANDLE,
                     w.DWORD, c.POINTER(pointer))
resize_console = api("ResizePseudoConsole", c.c_long, pointer, w.DWORD)
close_console = api("ClosePseudoConsole", None, pointer)
init_attrs = api("InitializeProcThreadAttributeList", w.BOOL, pointer, w.DWORD, w.DWORD, c.POINTER(size))
add_attr = api("UpdateProcThreadAttribute", w.BOOL, pointer, w.DWORD, size, pointer, size, pointer, pointer)
free_attrs = api("DeleteProcThreadAttributeList", None, pointer)
create = api("CreateProcessW", w.BOOL, w.LPCWSTR, w.LPWSTR, pointer, pointer, w.BOOL,
             w.DWORD, pointer, w.LPCWSTR, c.POINTER(Startup), c.POINTER(Process))
close = api("CloseHandle", w.BOOL, w.HANDLE)
wait = api("WaitForSingleObject", w.DWORD, w.HANDLE, w.DWORD)
exit_code = api("GetExitCodeProcess", w.BOOL, w.HANDLE, c.POINTER(w.DWORD))


def check(success):
    if not success:
        raise c.WinError(c.get_last_error())


def coord(cols, rows):
    return cols | (rows << 16)


def run(binary, interrupt):
    read_in, write_in = os.pipe()
    read_out, write_out = os.pipe()
    console = pointer()
    assert create_console(coord(120, 30), msvcrt.get_osfhandle(read_in),
                          msvcrt.get_osfhandle(write_out), 0, c.byref(console)) == 0
    os.close(read_in)
    os.close(write_out)
    chunks = queue.Queue()

    def drain():
        with os.fdopen(read_out, "rb", buffering=0) as stream:
            while data := stream.read(65536):
                chunks.put(data)
    reader = threading.Thread(target=drain, daemon=True)
    reader.start()

    def until(needle):
        buf = bytearray()
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            try:
                buf.extend(chunks.get(timeout=.1))
            except queue.Empty:
                continue
            if needle in buf:
                return
        raise AssertionError((needle, bytes(buf[-512:])))

    length = size()
    init_attrs(None, 1, 0, c.byref(length))
    attrs = c.create_string_buffer(length.value)
    check(init_attrs(attrs, 1, 0, c.byref(length)))
    proc = Process()
    try:
        check(add_attr(attrs, 0, 0x00020016, console, c.sizeof(pointer), None, None))
        startup = Startup()
        w.DWORD.from_buffer(startup).value = c.sizeof(startup)
        startup.attributes = c.cast(attrs, pointer)
        cmd = c.create_unicode_buffer(subprocess.list2cmdline([sys.executable, __file__, "--inside", str(binary)]))
        check(create(None, cmd, None, None, False, 0x80000, None, None,
                     c.byref(startup), c.byref(proc)))
        until(b"SPACE to run")
        os.write(write_in, b" ")
        until(b"SPACE jump")
        os.write(write_in, b"p")
        until(b"Paused")
        assert resize_console(console, coord(70, 12)) == 0
        until(b"Resize to")
        assert resize_console(console, coord(115, 28)) == 0
        until(b"Paused")
        os.write(write_in, b"\x03" if interrupt else b"q")
        until(b"shell restored")
        assert wait(proc.process, 5000) == 0
        code = w.DWORD()
        check(exit_code(proc.process, c.byref(code)))
        assert code.value == 0
    finally:
        os.close(write_in)
        close_console(console)
        free_attrs(attrs)
        if proc.process:
            close(proc.process)
            close(proc.thread)
        reader.join(timeout=2)


if __name__ == "__main__":
    if sys.argv[1] == "--inside":
        # Bind to the new pseudo console even when the CI parent redirects its own stdio.
        with open("CONIN$", "rb") as stdin, open("CONOUT$", "wb") as stdout:
            def mode(stream):
                value = w.DWORD()
                check(kernel.GetConsoleMode(w.HANDLE(msvcrt.get_osfhandle(stream.fileno())), c.byref(value)))
                return value.value
            stdout.write(b"TermRex test shell\r\n")
            stdout.flush()
            original = mode(stdin), mode(stdout)
            result = subprocess.run([sys.argv[2], "--mute"], stdin=stdin, stdout=stdout, stderr=stdout)
            assert (mode(stdin), mode(stdout)) == original
            text = c.create_unicode_buffer(18)
            read = w.DWORD()
            current = api("CreateFileW", w.HANDLE, w.LPCWSTR, w.DWORD, w.DWORD, pointer, w.DWORD, w.DWORD, w.HANDLE)("CONOUT$", 0x80000000, 3, None, 3, 0, None)
            try:
                check(kernel.ReadConsoleOutputCharacterW(w.HANDLE(current), text, 18, 0, c.byref(read)))
            finally:
                close(current)
            assert text.value.startswith("TermRex test shell"), text.value
            stdout.write(b"shell restored\r\n")
            stdout.flush()
            sys.exit(result.returncode)
    with tempfile.TemporaryDirectory() as state:
        os.environ["LOCALAPPDATA"] = state
        for interrupt in [False, True]:
            run(Path(sys.argv[1]).resolve(), interrupt)
    print("Windows ConPTY input, viewport resize and shell return passed.")
