use std::ffi::c_void;
use std::io::{self, Read, Write};
use std::net::TcpStream;

type OSStatus = i32;
type SSLContextRef = *mut c_void;
type ReadFn = extern "C" fn(*const c_void, *mut c_void, *mut usize) -> OSStatus;
type WriteFn = extern "C" fn(*const c_void, *const c_void, *mut usize) -> OSStatus;

const NO_ERR: OSStatus = 0;
const ERR_SSL_WOULD_BLOCK: OSStatus = -9803;
const ERR_SSL_CLOSED_GRACEFUL: OSStatus = -9805;
const ERR_SSL_CLOSED_ABORT: OSStatus = -9806;
const ERR_SSL_CLOSED_NO_NOTIFY: OSStatus = -9816;
const K_SSL_CLIENT_SIDE: i32 = 1;
const K_SSL_STREAM_TYPE: i32 = 0;

#[link(name = "Security", kind = "framework")]
extern "C" {
    fn SSLCreateContext(alloc: *const c_void, side: i32, kind: i32) -> SSLContextRef;
    fn SSLSetIOFuncs(ctx: SSLContextRef, read: ReadFn, write: WriteFn) -> OSStatus;
    fn SSLSetConnection(ctx: SSLContextRef, conn: *const c_void) -> OSStatus;
    fn SSLSetPeerDomainName(ctx: SSLContextRef, name: *const u8, len: usize) -> OSStatus;
    fn SSLHandshake(ctx: SSLContextRef) -> OSStatus;
    fn SSLRead(ctx: SSLContextRef, data: *mut c_void, len: usize, processed: *mut usize) -> OSStatus;
    fn SSLWrite(ctx: SSLContextRef, data: *const c_void, len: usize, processed: *mut usize) -> OSStatus;
    fn SSLClose(ctx: SSLContextRef) -> OSStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(obj: *const c_void);
}

extern "C" fn read_cb(conn: *const c_void, data: *mut c_void, len: *mut usize) -> OSStatus {
    let stream = unsafe { &mut *(conn as *mut TcpStream) };
    let wanted = unsafe { *len };
    let buf = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, wanted) };
    let mut filled = 0;
    let status = loop {
        if filled == wanted {
            break NO_ERR;
        }
        match stream.read(&mut buf[filled..]) {
            Ok(0) => break ERR_SSL_CLOSED_NO_NOTIFY,
            Ok(n) => filled += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break ERR_SSL_CLOSED_ABORT,
        }
    };
    unsafe { *len = filled };
    status
}

extern "C" fn write_cb(conn: *const c_void, data: *const c_void, len: *mut usize) -> OSStatus {
    let stream = unsafe { &mut *(conn as *mut TcpStream) };
    let buf = unsafe { std::slice::from_raw_parts(data as *const u8, *len) };
    match stream.write_all(buf) {
        Ok(()) => NO_ERR,
        Err(_) => {
            unsafe { *len = 0 };
            ERR_SSL_CLOSED_ABORT
        }
    }
}

fn tls_error(op: &str, status: OSStatus) -> io::Error {
    io::Error::other(format!("TLS {op} falhou (OSStatus {status})"))
}

pub struct TlsStream {
    ctx: SSLContextRef,
    _tcp: Box<TcpStream>,
}

impl TlsStream {
    pub fn connect(tcp: TcpStream, host: &str) -> io::Result<TlsStream> {
        let tcp = Box::new(tcp);
        let ctx = unsafe { SSLCreateContext(std::ptr::null(), K_SSL_CLIENT_SIDE, K_SSL_STREAM_TYPE) };
        if ctx.is_null() {
            return Err(io::Error::other("SSLCreateContext retornou nulo"));
        }
        let stream = TlsStream { ctx, _tcp: tcp };
        let conn = &*stream._tcp as *const TcpStream as *const c_void;
        unsafe {
            check("SSLSetIOFuncs", SSLSetIOFuncs(ctx, read_cb, write_cb))?;
            check("SSLSetConnection", SSLSetConnection(ctx, conn))?;
            check("SSLSetPeerDomainName", SSLSetPeerDomainName(ctx, host.as_ptr(), host.len()))?;
        }
        loop {
            match unsafe { SSLHandshake(ctx) } {
                NO_ERR => return Ok(stream),
                ERR_SSL_WOULD_BLOCK => continue,
                status => return Err(tls_error("handshake", status)),
            }
        }
    }
}

fn check(op: &str, status: OSStatus) -> io::Result<()> {
    if status == NO_ERR {
        Ok(())
    } else {
        Err(tls_error(op, status))
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            let mut processed = 0;
            let status = unsafe { SSLRead(self.ctx, buf.as_mut_ptr() as *mut c_void, buf.len(), &mut processed) };
            if processed > 0 {
                return Ok(processed);
            }
            match status {
                NO_ERR | ERR_SSL_WOULD_BLOCK => continue,
                ERR_SSL_CLOSED_GRACEFUL | ERR_SSL_CLOSED_NO_NOTIFY => return Ok(0),
                status => return Err(tls_error("read", status)),
            }
        }
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut processed = 0;
        let status = unsafe { SSLWrite(self.ctx, buf.as_ptr() as *const c_void, buf.len(), &mut processed) };
        match status {
            NO_ERR | ERR_SSL_WOULD_BLOCK => Ok(processed),
            status => Err(tls_error("write", status)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for TlsStream {
    fn drop(&mut self) {
        unsafe {
            SSLClose(self.ctx);
            CFRelease(self.ctx as *const c_void);
        }
    }
}
