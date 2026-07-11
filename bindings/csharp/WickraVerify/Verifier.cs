using System.Runtime.InteropServices;
using System.Text;

namespace Wickra.Verify;

/// <summary>
/// A verifier driven by JSON commands, over the Wickra C ABI. Create one, drive it
/// with command JSON (<c>verify</c>, <c>explain</c>, <c>canonicalize</c>,
/// <c>version</c>) and read back the response JSON — the same protocol as the CLI
/// and every other binding.
/// </summary>
public sealed class Verifier : IDisposable
{
    private readonly VerifierHandle _handle;

    /// <summary>Create a verifier with the default tolerances.</summary>
    public Verifier()
    {
        IntPtr ptr = Native.wickra_verify_new();
        if (ptr == IntPtr.Zero)
        {
            throw new InvalidOperationException("wickra-verify: failed to create a verifier");
        }
        _handle = new VerifierHandle(ptr);
    }

    /// <summary>Apply a command JSON and return the response JSON.</summary>
    /// <remarks>
    /// Uses the C ABI's length-out protocol: a first call learns the length, then
    /// the response is read into a caller-owned buffer. Domain errors (a bad claim,
    /// an unknown command) come back in-band as <c>{"ok":false,...}</c> JSON, not as
    /// an exception.
    /// </remarks>
    /// <exception cref="InvalidOperationException">A required argument was unusable or a panic was caught.</exception>
    public string Command(string cmdJson)
    {
        ObjectDisposedException.ThrowIf(_handle.IsInvalid, this);

        byte[] cmd = Utf8(cmdJson);
        IntPtr h = _handle.DangerousGetHandle();
        int n = Native.wickra_verify_command(h, cmd, null, 0);
        if (n < 0)
        {
            throw new InvalidOperationException($"wickra-verify: command failed (code {n})");
        }
        var buf = new byte[n + 1];
        Native.wickra_verify_command(h, cmd, buf, (nuint)buf.Length);
        return Encoding.UTF8.GetString(buf, 0, n);
    }

    /// <summary>The library version.</summary>
    public static string Version() =>
        Marshal.PtrToStringUTF8(Native.wickra_verify_version()) ?? string.Empty;

    /// <summary>Free the native verifier handle.</summary>
    public void Dispose() => _handle.Dispose();

    /// <summary>Encode a string as NUL-terminated UTF-8 for the C ABI.</summary>
    private static byte[] Utf8(string s)
    {
        int len = Encoding.UTF8.GetByteCount(s);
        var buf = new byte[len + 1];
        Encoding.UTF8.GetBytes(s, 0, s.Length, buf, 0);
        return buf;
    }
}

/// <summary>A safe handle owning a native verifier pointer.</summary>
internal sealed class VerifierHandle : SafeHandle
{
    public VerifierHandle(IntPtr handle)
        : base(IntPtr.Zero, ownsHandle: true) => SetHandle(handle);

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {
        Native.wickra_verify_free(handle);
        return true;
    }
}
