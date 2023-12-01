#[cfg(target_os = "freebsd")]
use crate::Error;
use crate::{Errno, NixPath, Result};
use libc::c_int;
#[cfg(target_os = "freebsd")]
use libc::{c_char, c_uint, c_void};
#[cfg(target_os = "freebsd")]
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    fmt, io,
    marker::PhantomData,
};

libc_bitflags!(
    /// Used with [`Nmount::nmount`].
    pub struct MntFlags: c_int {
        /// ACL support enabled.
        #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
        MNT_ACLS;
        /// All I/O to the file system should be done asynchronously.
        MNT_ASYNC;
        /// dir should instead be a file system ID encoded as “FSID:val0:val1”.
        #[cfg(target_os = "freebsd")]
        MNT_BYFSID;
        /// Force a read-write mount even if the file system appears to be
        /// unclean.
        MNT_FORCE;
        /// GEOM journal support enabled.
        #[cfg(target_os = "freebsd")]
        MNT_GJOURNAL;
        /// MAC support for objects.
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        MNT_MULTILABEL;
        /// Disable read clustering.
        #[cfg(freebsdlike)]
        MNT_NOCLUSTERR;
        /// Disable write clustering.
        #[cfg(freebsdlike)]
        MNT_NOCLUSTERW;
        /// Enable NFS version 4 ACLs.
        #[cfg(target_os = "freebsd")]
        MNT_NFS4ACLS;
        /// Do not update access times.
        MNT_NOATIME;
        /// Disallow program execution.
        MNT_NOEXEC;
        /// Do not honor setuid or setgid bits on files when executing them.
        MNT_NOSUID;
        /// Do not follow symlinks.
        #[cfg(freebsdlike)]
        MNT_NOSYMFOLLOW;
        /// Mount read-only.
        MNT_RDONLY;
        /// Causes the vfs subsystem to update its data structures pertaining to
        /// the specified already mounted file system.
        MNT_RELOAD;
        /// Create a snapshot of the file system.
        ///
        /// See [mksnap_ffs(8)](https://www.freebsd.org/cgi/man.cgi?query=mksnap_ffs)
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        MNT_SNAPSHOT;
        /// Using soft updates.
        #[cfg(any(freebsdlike, netbsdlike))]
        MNT_SOFTDEP;
        /// Directories with the SUID bit set chown new files to their own
        /// owner.
        #[cfg(freebsdlike)]
        MNT_SUIDDIR;
        /// All I/O to the file system should be done synchronously.
        MNT_SYNCHRONOUS;
        /// Union with underlying fs.
        #[cfg(any(
                target_os = "macos",
                target_os = "freebsd",
                target_os = "netbsd"
        ))]
        MNT_UNION;
        /// Indicates that the mount command is being applied to an already
        /// mounted file system.
        MNT_UPDATE;
        /// Check vnode use counts.
        #[cfg(target_os = "freebsd")]
        MNT_NONBUSY;
    }
);

/// The Error type of [`Nmount::nmount`].
///
/// It wraps an [`Errno`], but also may contain an additional message returned
/// by `nmount(2)`.
#[cfg(target_os = "freebsd")]
#[derive(Debug)]
pub struct NmountError {
    errno: Error,
    errmsg: Option<String>,
}

#[cfg(target_os = "freebsd")]
impl NmountError {
    /// Returns the additional error string sometimes generated by `nmount(2)`.
    pub fn errmsg(&self) -> Option<&str> {
        self.errmsg.as_deref()
    }

    /// Returns the inner [`Error`]
    pub const fn error(&self) -> Error {
        self.errno
    }

    fn new(error: Error, errmsg: Option<&CStr>) -> Self {
        Self {
            errno: error,
            errmsg: errmsg.map(CStr::to_string_lossy).map(Cow::into_owned),
        }
    }
}

#[cfg(target_os = "freebsd")]
impl std::error::Error for NmountError {}

#[cfg(target_os = "freebsd")]
impl fmt::Display for NmountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(errmsg) = &self.errmsg {
            write!(f, "{:?}: {}: {}", self.errno, errmsg, self.errno.desc())
        } else {
            write!(f, "{:?}: {}", self.errno, self.errno.desc())
        }
    }
}

#[cfg(target_os = "freebsd")]
impl From<NmountError> for io::Error {
    fn from(err: NmountError) -> Self {
        err.errno.into()
    }
}

/// Result type of [`Nmount::nmount`].
#[cfg(target_os = "freebsd")]
pub type NmountResult = std::result::Result<(), NmountError>;

/// Mount a FreeBSD file system.
///
/// The `nmount(2)` system call works similarly to the `mount(8)` program; it
/// takes its options as a series of name-value pairs.  Most of the values are
/// strings, as are all of the names.  The `Nmount` structure builds up an
/// argument list and then executes the syscall.
///
/// # Examples
///
/// To mount `target` onto `mountpoint` with `nullfs`:
/// ```
/// # use nix::unistd::Uid;
/// # use ::sysctl::{CtlValue, Sysctl};
/// # let ctl = ::sysctl::Ctl::new("vfs.usermount").unwrap();
/// # if !Uid::current().is_root() && CtlValue::Int(0) == ctl.value().unwrap() {
/// #     return;
/// # };
/// use nix::mount::{MntFlags, Nmount, unmount};
/// use std::ffi::CString;
/// use tempfile::tempdir;
///
/// let mountpoint = tempdir().unwrap();
/// let target = tempdir().unwrap();
///
/// let fstype = CString::new("fstype").unwrap();
/// let nullfs = CString::new("nullfs").unwrap();
/// Nmount::new()
///     .str_opt(&fstype, &nullfs)
///     .str_opt_owned("fspath", mountpoint.path().to_str().unwrap())
///     .str_opt_owned("target", target.path().to_str().unwrap())
///     .nmount(MntFlags::empty()).unwrap();
///
/// unmount(mountpoint.path(), MntFlags::empty()).unwrap();
/// ```
///
/// # See Also
/// * [`nmount(2)`](https://www.freebsd.org/cgi/man.cgi?query=nmount)
/// * [`nullfs(5)`](https://www.freebsd.org/cgi/man.cgi?query=nullfs)
#[cfg(target_os = "freebsd")]
#[derive(Debug, Default)]
pub struct Nmount<'a> {
    // n.b. notgull: In reality, this is a list that contains
    //               both mutable and immutable pointers.
    //               Be careful using this.
    iov: Vec<libc::iovec>,
    is_owned: Vec<bool>,
    marker: PhantomData<&'a ()>,
}

#[cfg(target_os = "freebsd")]
impl<'a> Nmount<'a> {
    /// Helper function to push a slice onto the `iov` array.
    fn push_slice(&mut self, val: &'a [u8], is_owned: bool) {
        self.iov.push(libc::iovec {
            iov_base: val.as_ptr().cast_mut().cast(),
            iov_len: val.len(),
        });
        self.is_owned.push(is_owned);
    }

    /// Helper function to push a pointer and its length onto the `iov` array.
    fn push_pointer_and_length(
        &mut self,
        val: *const u8,
        len: usize,
        is_owned: bool,
    ) {
        self.iov.push(libc::iovec {
            iov_base: val as *mut _,
            iov_len: len,
        });
        self.is_owned.push(is_owned);
    }

    /// Helper function to push a `nix` path as owned.
    fn push_nix_path<P: ?Sized + NixPath>(&mut self, val: &P) {
        val.with_nix_path(|s| {
            let len = s.to_bytes_with_nul().len();
            let ptr = s.to_owned().into_raw() as *const u8;

            self.push_pointer_and_length(ptr, len, true);
        })
        .unwrap();
    }

    /// Add an opaque mount option.
    ///
    /// Some file systems take binary-valued mount options.  They can be set
    /// with this method.
    ///
    /// # Safety
    ///
    /// Unsafe because it will cause `Nmount::nmount` to dereference a raw
    /// pointer.  The user is responsible for ensuring that `val` is valid and
    /// its lifetime outlives `self`!  An easy way to do that is to give the
    /// value a larger scope than `name`
    ///
    /// # Examples
    /// ```
    /// use libc::c_void;
    /// use nix::mount::Nmount;
    /// use std::ffi::CString;
    /// use std::mem;
    ///
    /// // Note that flags outlives name
    /// let mut flags: u32 = 0xdeadbeef;
    /// let name = CString::new("flags").unwrap();
    /// let p = &mut flags as *mut u32 as *mut c_void;
    /// let len = mem::size_of_val(&flags);
    /// let mut nmount = Nmount::new();
    /// unsafe { nmount.mut_ptr_opt(&name, p, len) };
    /// ```
    pub unsafe fn mut_ptr_opt(
        &mut self,
        name: &'a CStr,
        val: *mut c_void,
        len: usize,
    ) -> &mut Self {
        self.push_slice(name.to_bytes_with_nul(), false);
        self.push_pointer_and_length(val.cast(), len, false);
        self
    }

    /// Add a mount option that does not take a value.
    ///
    /// # Examples
    /// ```
    /// use nix::mount::Nmount;
    /// use std::ffi::CString;
    ///
    /// let read_only = CString::new("ro").unwrap();
    /// Nmount::new()
    ///     .null_opt(&read_only);
    /// ```
    pub fn null_opt(&mut self, name: &'a CStr) -> &mut Self {
        self.push_slice(name.to_bytes_with_nul(), false);
        self.push_slice(&[], false);
        self
    }

    /// Add a mount option that does not take a value, but whose name must be
    /// owned.
    ///
    ///
    /// This has higher runtime cost than [`Nmount::null_opt`], but is useful
    /// when the name's lifetime doesn't outlive the `Nmount`, or it's a
    /// different string type than `CStr`.
    ///
    /// # Examples
    /// ```
    /// use nix::mount::Nmount;
    ///
    /// let read_only = "ro";
    /// let mut nmount: Nmount<'static> = Nmount::new();
    /// nmount.null_opt_owned(read_only);
    /// ```
    pub fn null_opt_owned<P: ?Sized + NixPath>(
        &mut self,
        name: &P,
    ) -> &mut Self {
        self.push_nix_path(name);
        self.push_slice(&[], false);
        self
    }

    /// Add a mount option as a [`CStr`].
    ///
    /// # Examples
    /// ```
    /// use nix::mount::Nmount;
    /// use std::ffi::CString;
    ///
    /// let fstype = CString::new("fstype").unwrap();
    /// let nullfs = CString::new("nullfs").unwrap();
    /// Nmount::new()
    ///     .str_opt(&fstype, &nullfs);
    /// ```
    pub fn str_opt(&mut self, name: &'a CStr, val: &'a CStr) -> &mut Self {
        self.push_slice(name.to_bytes_with_nul(), false);
        self.push_slice(val.to_bytes_with_nul(), false);
        self
    }

    /// Add a mount option as an owned string.
    ///
    /// This has higher runtime cost than [`Nmount::str_opt`], but is useful
    /// when the value's lifetime doesn't outlive the `Nmount`, or it's a
    /// different string type than `CStr`.
    ///
    /// # Examples
    /// ```
    /// use nix::mount::Nmount;
    /// use std::path::Path;
    ///
    /// let mountpoint = Path::new("/mnt");
    /// Nmount::new()
    ///     .str_opt_owned("fspath", mountpoint.to_str().unwrap());
    /// ```
    pub fn str_opt_owned<P1, P2>(&mut self, name: &P1, val: &P2) -> &mut Self
    where
        P1: ?Sized + NixPath,
        P2: ?Sized + NixPath,
    {
        self.push_nix_path(name);
        self.push_nix_path(val);
        self
    }

    /// Create a new `Nmount` struct with no options
    pub fn new() -> Self {
        Self::default()
    }

    /// Actually mount the file system.
    pub fn nmount(&mut self, flags: MntFlags) -> NmountResult {
        const ERRMSG_NAME: &[u8] = b"errmsg\0";
        let mut errmsg = vec![0u8; 255];

        // nmount can return extra error information via a "errmsg" return
        // argument.
        self.push_slice(ERRMSG_NAME, false);

        // SAFETY: we are pushing a mutable iovec here, so we can't use
        //         the above method
        self.iov.push(libc::iovec {
            iov_base: errmsg.as_mut_ptr().cast(),
            iov_len: errmsg.len(),
        });

        let niov = self.iov.len() as c_uint;
        let iovp = self.iov.as_mut_ptr();
        let res = unsafe { libc::nmount(iovp, niov, flags.bits()) };
        match Errno::result(res) {
            Ok(_) => Ok(()),
            Err(error) => {
                let errmsg = if errmsg[0] == 0 {
                    None
                } else {
                    CStr::from_bytes_until_nul(&errmsg[..]).ok()
                };
                Err(NmountError::new(error, errmsg))
            }
        }
    }
}

#[cfg(target_os = "freebsd")]
impl<'a> Drop for Nmount<'a> {
    fn drop(&mut self) {
        for (iov, is_owned) in self.iov.iter().zip(self.is_owned.iter()) {
            if *is_owned {
                // Free the owned string.  Safe because we recorded ownership,
                // and Nmount does not implement Clone.
                unsafe {
                    drop(CString::from_raw(iov.iov_base as *mut c_char));
                }
            }
        }
    }
}

/// Unmount the file system mounted at `mountpoint`.
///
/// Useful flags include
/// * `MNT_FORCE` -     Unmount even if still in use.
#[cfg_attr(
    target_os = "freebsd",
    doc = "
* `MNT_BYFSID` -    `mountpoint` is not a path, but a file system ID
                    encoded as `FSID:val0:val1`, where `val0` and `val1`
                    are the contents of the `fsid_t val[]` array in decimal.
                    The file system that has the specified file system ID
                    will be unmounted.  See
                    [`statfs`](crate::sys::statfs::statfs) to determine the
                    `fsid`.
"
)]
pub fn unmount<P>(mountpoint: &P, flags: MntFlags) -> Result<()>
where
    P: ?Sized + NixPath,
{
    let res = mountpoint.with_nix_path(|cstr| unsafe {
        libc::unmount(cstr.as_ptr(), flags.bits())
    })?;

    Errno::result(res).map(drop)
}
