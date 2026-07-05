using System.Runtime.InteropServices;

namespace RustSharp.Core;

public static class RustString
{
    public static string Version()
        => Marshal.PtrToStringUTF8(Interop.Native.Version()) ?? "unknown";

    public static string Reverse(string input)
        => FromRust(Interop.Native.StringReverse(input));

    public static string ToUppercase(string input)
        => FromRust(Interop.Native.StringToUppercase(input));

    public static nuint CharCount(string input)
        => Interop.Native.StringCharCount(input);

    private static string FromRust(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
            throw new InvalidOperationException("Rust returned a null string pointer.");
        try   { return Marshal.PtrToStringUTF8(ptr) ?? string.Empty; }
        finally { Interop.Native.StringFree(ptr); }
    }
}
