use libc;
use std::io::{self, Read, Seek, SeekFrom};
use std::os::unix::io::AsRawFd;
use tempfile::tempfile;

pub fn capture_stdout<F>(func: F) -> io::Result<String>
where
    F: FnOnce(),
{
    // Create a temporary file to capture stdout
    let mut temp_file = tempfile()?;
    let temp_file_fd = temp_file.as_raw_fd();

    // Save the original stdout file descriptor
    let original_stdout_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };

    // Redirect stdout to the temporary file
    unsafe {
        libc::dup2(temp_file_fd, libc::STDOUT_FILENO);
    }

    // Execute the function
    func();

    // Restore the original stdout
    unsafe {
        libc::dup2(original_stdout_fd, libc::STDOUT_FILENO);
        libc::close(original_stdout_fd);
    }

    // Read the captured output from the temporary file
    temp_file.seek(SeekFrom::Start(0))?;
    let mut output = String::new();
    temp_file.read_to_string(&mut output)?;

    Ok(output)
}
