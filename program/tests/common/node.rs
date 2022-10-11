use crate::common::{env, port, Error, Result};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};

<<<<<<< HEAD
/// Run gear with docker.
=======
/// Run gear-node with docker.
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)
pub struct Node {
    /// child process
    ps: Child,
    /// websocket port
    port: u16,
}

impl Node {
    /// node websocket addr
    pub fn ws(&self) -> String {
        format!("ws://{}:{}", port::LOCALHOST, self.port)
    }

<<<<<<< HEAD
    /// Run gear with docker in development mode.
    pub fn dev() -> Result<Self> {
        let port = port::pick();
        let ps = Command::new(env::bin("gear"))
=======
    /// Run gear-node with docker in development mode.
    pub fn dev() -> Result<Self> {
        let port = port::pick();
        let ps = Command::new(env::bin("gear-node"))
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)
            .args(["--ws-port", &port.to_string(), "--tmp", "--dev"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self { ps, port })
    }

    /// Wait for the block importing
    pub fn wait(&mut self, log: &str) -> Result<String> {
        let stderr = self.ps.stderr.take();
        let reader = BufReader::new(stderr.ok_or(Error::EmptyStderr)?);
        for line in reader.lines().flatten() {
            if line.contains(log) {
                return Ok(line);
            }
        }

        Err(Error::EmptyStderr)
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.ps.kill().expect("Failed to kill process")
    }
}
