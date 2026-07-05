namespace RustSharp.Core;

public static class RustMath
{
    public static long Add(long a, long b) => Interop.Native.Add(a, b);

    public static long Fibonacci(uint n) => Interop.Native.Fibonacci(n);

    public static bool IsPrime(ulong n) => Interop.Native.IsPrime(n);

    public static ulong CountPrimes(ulong limit)
    {
        if (limit > 100_000_000UL)
            throw new ArgumentOutOfRangeException(nameof(limit), "Limit must be ≤ 100,000,000.");

        var result = Interop.Native.CountPrimes(limit);

        return result == ulong.MaxValue
            ? throw new InvalidOperationException("Rust count_primes returned an error.")
            : result;
    }
} 
