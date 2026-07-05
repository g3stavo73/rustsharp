namespace RustSharp.Core;

public static class RustCollection
{
    public static long[] Sort(long[] data)
    {
        var copy = (long[])data.Clone();
        SortInPlace(copy);
        return copy;
    }

    public static void SortInPlace(long[] data)
    {
        if (data.Length > 0)
            Interop.Native.SortI64(data, (nuint)data.Length);
    }

    public static long Sum(long[] data)
        => data.Length == 0 ? 0 : Interop.Native.SumI64(data, (nuint)data.Length);

    public static long Max(long[] data)
    {
        if (data.Length == 0) throw new InvalidOperationException("Array is empty.");
        return Interop.Native.MaxI64(data, (nuint)data.Length);
    }
}
