
//Access
pub static  O_RDONLY    : u32 = 0x0000; // Open file in read only
pub static  O_WRONLY    : u32 = 0x0001; // Open file in write only
pub static  O_RDWR		: u32 = 0x0002; // Open file in read write
pub static  O_ACCMODE	: u32 = 0x0003; // Mask for write


//Modifiers
pub static O_CREAT		: u32 = 0x0100;
/*
 If  the file exists, this flag has no effect except as noted under O_EXCL below.
Otherwise, the file shall be created; the user ID of the file shall  be  set  to
the  effective  user ID of the process; the group ID of the file shall be set to
the group ID of the file's parent directory or to the effective group ID of  the
process;  and  the  access  permission  bits (see <sys/stat.h>) of the file mode
shall be set to the value of the argument following the oflag argument taken  as
type  mode_t  modified  as  follows: a bitwise AND is performed on the file-mode
bits and the corresponding bits in the complement of the process' file mode cre‐
ation  mask. Thus, all bits in the file mode whose corresponding bit in the file
mode creation mask is set are cleared. When bits other than the file  permission
bits  are set, the effect is unspecified. The argument following the oflag argu‐
ment does not affect whether the file is open for reading, writing, or for both.
Implementations  shall  provide  a  way to initialize the file's group ID to the
group ID of the parent directory. Implementations may, but need not, provide  an
implementation-defined  way  to  initialize the file's group ID to the effective
group ID of the calling process.
*/

pub static O_EXCL		: u32 = 0x0200;
/*
If  O_CREAT  and O_EXCL are set, open() shall fail if the file exists. The check
for the existence of the file and the creation of the file if it does not  exist
shall  be  atomic with respect to other threads executing open() naming the same
filename in the same directory with  O_EXCL  and  O_CREAT  set.  If  O_EXCL  and
O_CREAT are set, and path names a symbolic link, open() shall fail and set errno
to [EEXIST], regardless of the contents of the symbolic link. If O_EXCL  is  set
and O_CREAT is not set, the result is undefined.
*/

pub static O_NOCTTY	    : u32 = 0x0400;
/*
If  set and path identifies a terminal device, open() shall not cause the termi‐
nal device to become the controlling terminal for the process. If path does  not
identify a terminal device, O_NOCTTY shall be ignored.

*/

pub static O_TRUNC		: u32 = 0x0800;
/*
If  the  file  exists and is a regular file, and the file is successfully opened
O_RDWR or O_WRONLY, its length shall be truncated to 0, and the mode  and  owner
shall  be  unchanged.  It shall have no effect on FIFO special files or terminal
device files. Its effect on other  file  types  is  implementation-defined.  The
result of using O_TRUNC without either O_RDWR or O_WRONLY is undefined.
*/

pub static O_APPEND	    : u32 = 0x1000;
/* If set, the file offset shall be set to the end of the file prior to each write. */

