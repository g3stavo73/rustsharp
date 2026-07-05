using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace RustSharp.Core.Interop;

internal static partial class Native
{
    private const string Lib = "rustsharp";

    [LibraryImport(Lib, EntryPoint = "rustsharp_version")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr Version();

    [LibraryImport(Lib, EntryPoint = "rustsharp_add")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial long Add(long a, long b);

    [LibraryImport(Lib, EntryPoint = "rustsharp_fibonacci")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial long Fibonacci(uint n);

    [LibraryImport(Lib, EntryPoint = "rustsharp_is_prime")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    [return: MarshalAs(UnmanagedType.U1)]
    internal static partial bool IsPrime(ulong n);

    // Returns ulong.MaxValue on error.
    [LibraryImport(Lib, EntryPoint = "rustsharp_count_primes")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial ulong CountPrimes(ulong limit);

    [LibraryImport(Lib, EntryPoint = "rustsharp_string_reverse", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr StringReverse(string input);

    [LibraryImport(Lib, EntryPoint = "rustsharp_string_to_uppercase", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr StringToUppercase(string input);

    [LibraryImport(Lib, EntryPoint = "rustsharp_string_char_count", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nuint StringCharCount(string input);

    [LibraryImport(Lib, EntryPoint = "rustsharp_string_free")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StringFree(IntPtr ptr);

    [LibraryImport(Lib, EntryPoint = "rustsharp_sort_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SortI64(long[] data, nuint len);

    [LibraryImport(Lib, EntryPoint = "rustsharp_sum_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial long SumI64(long[] data, nuint len);

    [LibraryImport(Lib, EntryPoint = "rustsharp_max_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial long MaxI64(long[] data, nuint len);
}
