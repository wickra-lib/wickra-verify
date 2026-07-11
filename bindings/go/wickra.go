// Package wickra provides idiomatic Go bindings for wickra-verify over its C ABI
// hub: create a Verifier, drive it with command JSON (verify, explain,
// canonicalize, version) and read back the response JSON — the same protocol as
// the CLI and every other binding.
//
// The binding links the prebuilt C ABI library, staged per platform under
// ./lib/<goos>_<goarch>/, with the header vendored under ./include.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_verify -Wl,-rpath,${SRCDIR}/lib/linux_amd64
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/lib/linux_arm64 -lwickra_verify -Wl,-rpath,${SRCDIR}/lib/linux_arm64
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -lwickra_verify -Wl,-rpath,${SRCDIR}/lib/darwin_amd64
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_verify -Wl,-rpath,${SRCDIR}/lib/darwin_arm64
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -l:wickra_verify.dll
#cgo windows,arm64 LDFLAGS: -L${SRCDIR}/lib/windows_arm64 -l:wickra_verify.dll
#include <stdlib.h>
#include "wickra_verify.h"
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"
)

// Verifier is driven by JSON commands.
type Verifier struct {
	handle *C.WickraVerify
}

// New creates a verifier with the default tolerances. Call Close when done (a
// finalizer also frees it, but explicit Close is preferred).
func New() *Verifier {
	v := &Verifier{handle: C.wickra_verify_new()}
	runtime.SetFinalizer(v, (*Verifier).Close)
	return v
}

// Command applies a command JSON and returns the response JSON. It uses the C
// ABI's length-out protocol: a first call learns the length, then the response
// is read into a caller-owned buffer.
func (v *Verifier) Command(cmdJSON string) (string, error) {
	ccmd := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(ccmd))

	n := C.wickra_verify_command(v.handle, ccmd, nil, 0)
	if n < 0 {
		return "", fmt.Errorf("wickra-verify: command failed (code %d)", int(n))
	}
	buf := make([]byte, int(n)+1)
	C.wickra_verify_command(
		v.handle,
		ccmd,
		(*C.char)(unsafe.Pointer(&buf[0])),
		C.size_t(len(buf)),
	)
	return string(buf[:n]), nil
}

// Close frees the verifier handle. Safe to call more than once.
func (v *Verifier) Close() {
	if v.handle != nil {
		C.wickra_verify_free(v.handle)
		v.handle = nil
	}
	runtime.SetFinalizer(v, nil)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.wickra_verify_version())
}
