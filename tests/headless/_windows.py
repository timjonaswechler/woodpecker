"""Read-only CoreGraphics audit, returning records only for the test-owned PID."""
import ctypes as c
import sys


def owned_windows(pid):
    if sys.platform != "darwin" or type(pid) is not int or pid <= 1:
        raise RuntimeError("Metal headless window audit requires macOS and an owned PID")
    cg = c.CDLL("/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics")
    cf = c.CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
    cg.CGWindowListCopyWindowInfo.argtypes = [c.c_uint32, c.c_uint32]
    cg.CGWindowListCopyWindowInfo.restype = c.c_void_p
    cf.CFArrayGetCount.argtypes = [c.c_void_p]
    cf.CFArrayGetCount.restype = c.c_long
    cf.CFArrayGetValueAtIndex.argtypes = [c.c_void_p, c.c_long]
    cf.CFArrayGetValueAtIndex.restype = c.c_void_p
    cf.CFDictionaryGetValue.argtypes = [c.c_void_p, c.c_void_p]
    cf.CFDictionaryGetValue.restype = c.c_void_p
    cf.CFNumberGetValue.argtypes = [c.c_void_p, c.c_int, c.c_void_p]
    cf.CFNumberGetValue.restype = c.c_bool
    cf.CFRelease.argtypes = [c.c_void_p]
    owner_key = c.c_void_p.in_dll(cg, "kCGWindowOwnerPID")
    number_key = c.c_void_p.in_dll(cg, "kCGWindowNumber")
    records = cg.CGWindowListCopyWindowInfo(0, 0)
    if not records:
        raise RuntimeError("CoreGraphics did not return a window list")
    def number(record, key):
        value = cf.CFDictionaryGetValue(record, key)
        result = c.c_int64()
        if not value or not cf.CFNumberGetValue(value, 4, c.byref(result)):
            raise RuntimeError("Invalid CoreGraphics numeric record")
        return result.value
    try:
        # Never log foreign names/titles or return any foreign process records.
        return [number(record, number_key) for index in range(cf.CFArrayGetCount(records))
                if number(record := cf.CFArrayGetValueAtIndex(records, index), owner_key) == pid]
    finally:
        cf.CFRelease(records)
