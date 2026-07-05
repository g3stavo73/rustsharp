using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;
using RustSharp.Core;

var builder = WebApplication.CreateBuilder(args);
builder.Services.ConfigureHttpJsonOptions(o => {
    o.SerializerOptions.DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull;
    o.SerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower;
    o.SerializerOptions.WriteIndented = true;
});

var app = builder.Build();

app.MapGet("/", () => Results.Redirect("/info"));

app.MapGet("/info", () => new {
    name        = "RustSharp",
    description = "Uma ponte entre Rust e o ecossistema .NET",
    version     = RustString.Version(),
    stack       = new { core = "Rust cdylib", bridge = "P/Invoke (LibraryImport)", api = ".NET 9 ASP.NET Core" },
    endpoints   = new[] {
        "GET  /info",
        "GET  /math/add?a=&b=",
        "GET  /math/fibonacci/{n}",
        "GET  /math/prime/{n}",
        "GET  /math/primes/count/{limit}",
        "POST /strings/reverse",
        "POST /strings/uppercase",
        "POST /collections/sort",
        "POST /collections/stats",
        "GET  /benchmark/fibonacci/{n}",
        "GET  /benchmark/primes/{limit}",
    },
});

app.MapGet("/math/add", (long a, long b) => Timed(() => RustMath.Add(a, b), new { a, b }));

app.MapGet("/math/fibonacci/{n}", (uint n) => Timed(() => RustMath.Fibonacci(n), new { n }));

app.MapGet("/math/prime/{n}", (ulong n) => Timed(() => RustMath.IsPrime(n), new { n }));

app.MapGet("/math/primes/count/{limit}", (ulong limit) => {
    if (limit > 100_000_000UL)
        return Results.BadRequest(new { error = "limit must be ≤ 100,000,000." });
    return Results.Ok(Timed(() => RustMath.CountPrimes(limit), new { limit }));
});

app.MapPost("/strings/reverse", async (HttpRequest req) => {
    var s = await ReadBody(req);
    return Timed(() => RustString.Reverse(s), new { input = s });
});

app.MapPost("/strings/uppercase", async (HttpRequest req) => {
    var s = await ReadBody(req);
    return Timed(() => (result: RustString.ToUppercase(s), chars: (long)RustString.CharCount(s)),
        new { input = s });
});

app.MapPost("/collections/sort", async (HttpRequest req) => {
    var nums = await req.ReadFromJsonAsync<long[]>() ?? [];
    return Timed(() => RustCollection.Sort(nums), new { input = nums });
});

app.MapPost("/collections/stats", async (HttpRequest req) => {
    var nums = await req.ReadFromJsonAsync<long[]>() ?? [];
    if (nums.Length == 0) return Results.BadRequest(new { error = "Array must not be empty." });
    return Results.Ok(Timed(() => {
        var sorted = RustCollection.Sort(nums);
        return (sum: RustCollection.Sum(nums), max: RustCollection.Max(nums), min: sorted[0], sorted);
    }, new { count = nums.Length }));
});

app.MapGet("/benchmark/fibonacci/{n}", (uint n) => {
    const int iter = 10_000;
    Warmup(() => { RustMath.Fibonacci(n); FibCs(n); });
    var (rt, rr) = Bench(iter, () => RustMath.Fibonacci(n));
    var (ct, _)  = Bench(iter, () => FibCs(n));
    return new { n, result = rr, iterations = iter,
        rust_avg_ns = rt / iter, csharp_avg_ns = ct / iter,
        note = "Rust includes P/Invoke overhead per call." };
});

app.MapGet("/benchmark/primes/{limit}", (ulong limit) => {
    if (limit > 100_000_000UL)
        return Results.BadRequest(new { error = "limit must be ≤ 100,000,000." });
    Warmup(() => { RustMath.CountPrimes(limit); PrimesCs(limit); });
    var (rt, rr) = Bench(1, () => RustMath.CountPrimes(limit));
    var (ct, _)  = Bench(1, () => PrimesCs(limit));
    return Results.Ok(new { limit, prime_count = rr,
        rust_ms = rt / 1_000_000.0, csharp_ms = ct / 1_000_000.0,
        agreement = rr == PrimesCs(limit),
        note = "Both use Sieve of Eratosthenes. Rust: lto=true, codegen-units=1." });
});

var port = Environment.GetEnvironmentVariable("PORT") ?? "8099";
app.Run($"http://0.0.0.0:{port}");

static object Timed<T>(Func<T> fn, object extra) {
    var sw = Stopwatch.StartNew();
    var result = fn();
    sw.Stop();
    return new { result, elapsed_ns = sw.Elapsed.TotalNanoseconds, engine = "Rust", extra };
}

static (long ns, T result) Bench<T>(int n, Func<T> fn) {
    var sw = Stopwatch.StartNew();
    T last = default!;
    for (var i = 0; i < n; i++) last = fn();
    sw.Stop();
    return ((long)sw.Elapsed.TotalNanoseconds, last);
}

static void Warmup(Action fn) { fn(); fn(); }

static async Task<string> ReadBody(HttpRequest req) {
    using var r = new StreamReader(req.Body);
    return (await r.ReadToEndAsync()).Trim().Trim('"');
}

static long FibCs(uint n) {
    if (n <= 1) return n;
    long a = 0, b = 1;
    for (uint i = 2; i <= n; i++) (a, b) = (b, a + b);
    return b;
}

static ulong PrimesCs(ulong limit) {
    if (limit < 2) return 0;
    var sieve = new bool[(int)limit + 1];
    Array.Fill(sieve, true);
    sieve[0] = sieve[1] = false;
    for (var i = 2; (long)i * i <= (long)limit; i++)
        if (sieve[i])
            for (var j = i * i; j <= (int)limit; j += i)
                sieve[j] = false;
    ulong c = 0; foreach (var b in sieve) if (b) c++; return c;
}
