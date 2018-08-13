use std::os::raw::c_int;

pub static EPERM: c_int = 1; /* Operation not permitted */
pub static ENOENT: c_int = 2; /* No such file or directory */
pub static ESRCH: c_int = 3; /* No such process */
pub static EINTR: c_int = 4; /* Interrupted system call */
pub static EIO: c_int = 5; /* I/O error */
pub static ENXIO: c_int = 6; /* No such device or address */
pub static E2BIG: c_int = 7; /* Arg list too long */
pub static ENOEXEC: c_int = 8; /* Exec format error */
pub static EBADF: c_int = 9; /* Bad file number */
pub static ECHILD: c_int = 10; /* No child processes */
pub static EAGAIN: c_int = 11; /* Try again */
pub static ENOMEM: c_int = 12; /* Out of memory */
pub static EACCES: c_int = 13; /* Permission denied */
pub static EFAULT: c_int = 14; /* Bad address */
pub static ENOTBLK: c_int = 15; /* Block device required */
pub static EBUSY: c_int = 16; /* Device or resource busy */
pub static EEXIST: c_int = 17; /* File exists */
pub static EXDEV: c_int = 18; /* Cross-device link */
pub static ENODEV: c_int = 19; /* No such device */
pub static ENOTDIR: c_int = 20; /* Not a directory */
pub static EISDIR: c_int = 21; /* Is a directory */
pub static EINVAL: c_int = 22; /* Invalid argument */
pub static ENFILE: c_int = 23; /* File table overflow */
pub static EMFILE: c_int = 24; /* Too many open files */
pub static ENOTTY: c_int = 25; /* Not a typewriter */
pub static ETXTBSY: c_int = 26; /* Text file busy */
pub static EFBIG: c_int = 27; /* File too large */
pub static ENOSPC: c_int = 28; /* No space left on device */
pub static ESPIPE: c_int = 29; /* Illegal seek */
pub static EROFS: c_int = 30; /* Read-only file system */
pub static EMLINK: c_int = 31; /* Too many links */
pub static EPIPE: c_int = 32; /* Broken pipe */
pub static EDOM: c_int = 33; /* Math argument out of domain of func */
pub static ERANGE: c_int = 34; /* Math result not representable */
pub static EDEADLK: c_int = 35; /* Resource deadlock would occur */
pub static ENAMETOOLONG: c_int = 36; /* File name too long */
pub static ENOLCK: c_int = 37; /* No record locks available */
pub static ENOSYS: c_int = 38; /* Function not implemented */
pub static ENOTEMPTY: c_int = 39; /* Directory not empty */
pub static ELOOP: c_int = 40; /* Too many symbolic links encountered */
pub static EWOULDBLOCK: c_int = 11; /* Operation would block */
pub static ENOMSG: c_int = 42; /* No message of desired type */
pub static EIDRM: c_int = 43; /* Identifier removed */
pub static ECHRNG: c_int = 44; /* Channel number out of range */
pub static EL2NSYNC: c_int = 45; /* Level 2 not synchronized */
pub static EL3HLT: c_int = 46; /* Level: c_int = 3 halted */
pub static EL3RST: c_int = 47; /* Level: c_int = 3 reset */
pub static ELNRNG: c_int = 48; /* Link number out of range */
pub static EUNATCH: c_int = 49; /* Protocol driver not attached */
pub static ENOCSI: c_int = 50; /* No CSI structure available */
pub static EL2HLT: c_int = 51; /* Level: c_int = 2 halted */
pub static EBADE: c_int = 52; /* Invalid exchange */
pub static EBADR: c_int = 53; /* Invalid request descriptor */
pub static EXFULL: c_int = 54; /* Exchange full */
pub static ENOANO: c_int = 55; /* No anode */
pub static EBADRQC: c_int = 56; /* Invalid request code */
pub static EBADSLT: c_int = 57; /* Invalid slot */
pub static EDEADLOCK: c_int = 35;
pub static EBFONT: c_int = 59; /* Bad font file format */
pub static ENOSTR: c_int = 60; /* Device not a stream */
pub static ENODATA: c_int = 61; /* No data available */
pub static ETIME: c_int = 62; /* Timer expired */
pub static ENOSR: c_int = 63; /* Out of streams resources */
pub static ENONET: c_int = 64; /* Machine is not on the network */
pub static ENOPKG: c_int = 65; /* Package not installed */
pub static EREMOTE: c_int = 66; /* Object is remote */
pub static ENOLINK: c_int = 67; /* Link has been severed */
pub static EADV: c_int = 68; /* Advertise error */
pub static ESRMNT: c_int = 69; /* Srmount error */
pub static ECOMM: c_int = 70; /* Communication error on send */
pub static EPROTO: c_int = 71; /* Protocol error */
pub static EMULTIHOP: c_int = 72; /* Multihop attempted */
pub static EDOTDOT: c_int = 73; /* RFS specific error */
pub static EBADMSG: c_int = 74; /* Not a data message */
pub static EOVERFLOW: c_int = 75; /* Value too large for defined data type */
pub static ENOTUNIQ: c_int = 76; /* Name not unique on network */
pub static EBADFD: c_int = 77; /* File descriptor in bad state */
pub static EREMCHG: c_int = 78; /* Remote address changed */
pub static ELIBACC: c_int = 79; /* Can not access a needed shared library */
pub static ELIBBAD: c_int = 80; /* Accessing a corrupted shared library */
pub static ELIBSCN: c_int = 81; /* .lib section in a.out corrupted */
pub static ELIBMAX: c_int = 82; /* Attempting to link in too many shared libraries */
pub static ELIBEXEC: c_int = 83; /* Cannot exec a shared library directly */
pub static EILSEQ: c_int = 84; /* Illegal byte sequence */
pub static ERESTART: c_int = 85; /* Interrupted system call should be restarted */
pub static ESTRPIPE: c_int = 86; /* Streams pipe error */
pub static EUSERS: c_int = 87; /* Too many users */
pub static ENOTSOCK: c_int = 88; /* Socket operation on non-socket */
pub static EDESTADDRREQ: c_int = 89; /* Destination address required */
pub static EMSGSIZE: c_int = 90; /* Message too long */
pub static EPROTOTYPE: c_int = 91; /* Protocol wrong type for socket */
pub static ENOPROTOOPT: c_int = 92; /* Protocol not available */
pub static EPROTONOSUPPORT: c_int = 93; /* Protocol not supported */
pub static ESOCKTNOSUPPORT: c_int = 94; /* Socket type not supported */
pub static EOPNOTSUPP: c_int = 95; /* Operation not supported on transport endpoint */
pub static EPFNOSUPPORT: c_int = 96; /* Protocol family not supported */
pub static EAFNOSUPPORT: c_int = 97; /* Address family not supported by protocol */
pub static EADDRINUSE: c_int = 98; /* Address already in use */
pub static EADDRNOTAVAIL: c_int = 99; /* Cannot assign requested address */
pub static ENETDOWN: c_int = 100; /* Network is down */
pub static ENETUNREACH: c_int = 101; /* Network is unreachable */
pub static ENETRESET: c_int = 102; /* Network dropped connection because of reset */
pub static ECONNABORTED: c_int = 103; /* Software caused connection abort */
pub static ECONNRESET: c_int = 104; /* Connection reset by peer */
pub static ENOBUFS: c_int = 105; /* No buffer space available */
pub static EISCONN: c_int = 106; /* Transport endpoint is already connected */
pub static ENOTCONN: c_int = 107; /* Transport endpoint is not connected */
pub static ESHUTDOWN: c_int = 108; /* Cannot send after transport endpoint shutdown */
pub static ETOOMANYREFS: c_int = 109; /* Too many references: cannot splice */
pub static ETIMEDOUT: c_int = 110; /* Connection timed out */
pub static ECONNREFUSED: c_int = 111; /* Connection refused */
pub static EHOSTDOWN: c_int = 112; /* Host is down */
pub static EHOSTUNREACH: c_int = 113; /* No route to host */
pub static EALREADY: c_int = 114; /* Operation already in progress */
pub static EINPROGRESS: c_int = 115; /* Operation now in progress */
pub static ESTALE: c_int = 116; /* Stale NFS file handle */
pub static EUCLEAN: c_int = 117; /* Structure needs cleaning */
pub static ENOTNAM: c_int = 118; /* Not a XENIX named type file */
pub static ENAVAIL: c_int = 119; /* No XENIX semaphores available */
pub static EISNAM: c_int = 120; /* Is a named type file */
pub static EREMOTEIO: c_int = 121; /* Remote I/O error */
pub static EDQUOT: c_int = 122; /* Quota exceeded */
pub static ENOMEDIUM: c_int = 123; /* No medium found */
pub static EMEDIUMTYPE: c_int = 124; /* Wrong medium type */
