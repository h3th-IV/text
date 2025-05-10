use std::{ffi::CString, fs::File, io::{self}, os::fd::FromRawFd};
use std::str;
use memmap2::{Mmap, MmapAsRawDesc, MmapMut};

/* write to file takes in two function arguments */
/* arg_one : filename to store contents of file */
/* arg_two : contents to actually store in file */
pub fn _write_to_file(filename: &'static str, contents: Vec<&[u8]>) -> io::Result<()> {
    let path = CString::new(filename).unwrap();
    let descriptor = unsafe {
        libc::open(path.as_ptr(), libc::O_RDWR | libc::O_CREAT, 0644)
    };
    if descriptor < 0  { return Err(io::Error::last_os_error());}
    let file = unsafe { File::from_raw_fd(descriptor) };
    file.set_len(contents.len() as u64).unwrap();
    let mut write_mut = unsafe {MmapMut::map_mut(&file)?};
    for i in 0..contents.len() {
        write_mut[..contents.len()].copy_from_slice(contents[i]);
    }
    write_mut.flush()?;
    Ok(())
}

/* here we take in two arguments file and contents */
/* filename implements the MmapAsRawDesc trait for reads */
/* we then read it safely from memory and return a string */
pub fn _read_from_file<T>(filename: T, contents: Vec<&[u8]>) -> io::Result<String> 
where T: MmapAsRawDesc 
{
    let read_mut = unsafe { Mmap::map(filename)? };
    let final_read = str::from_utf8(&read_mut[..contents.len()]).unwrap();
    Ok(final_read.to_string())
}