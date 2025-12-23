//use named_pipe::PipeClient;
//use serde_json::Value;
//use std::thread::sleep;

use std::io::{self, Write, ErrorKind};
use std::process::Command;
use std::time::Duration;

pub use named_pipe::PipeClient;   // 🔑 REQUIRED
use serde_json::Value;

pub fn spawn_mpv_with_pipe(pipe_name: &str) -> io::Result<std::process::Child> {
    Command::new(r"C:\Users\HP\Downloads\bootstrapper\mpv.exe")
        //.arg("--no-video")
        .arg("--idle=yes")
        .arg("--force-window=no")
        .arg("--volume=50")
        .arg(format!("--input-ipc-server={}", pipe_name))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
}

pub fn connect_pipe_with_retry(
    pipe_name: &str,
    tries: usize,
    delay_ms: u64,
) -> io::Result<PipeClient> {
    for _ in 0..tries {
        if let Ok(pipe) = PipeClient::connect(pipe_name) {
            return Ok(pipe);
        }
        std::thread::sleep(Duration::from_millis(delay_ms));
    }

    Err(io::Error::new(io::ErrorKind::Other, "Failed to connect pipe"))
}

pub fn send_json_command(
    pipe: &mut PipeClient,
    pipe_name: &str,
    cmd: Value,
) -> io::Result<()> {
    let data = serde_json::to_vec(&cmd)?;

    match pipe.write_all(&data).and_then(|_| pipe.write_all(b"\n")) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {
            *pipe = connect_pipe_with_retry(pipe_name, 20, 100)?;
            pipe.write_all(&data)?;
            pipe.write_all(b"\n")
        }
        Err(e) => Err(e),
    }
}
