//! Runs the aardvark dns server with provided config
use crate::config;
use crate::server::serve;
use clap::Parser;
use nix::unistd::{dup2, fork, ForkResult};
use std::fs::OpenOptions;
use std::io::Error;
use std::os::unix::io::AsRawFd;

#[derive(Parser, Debug)]
pub struct Run {}

impl Run {
    /// The run command runs the aardvark-dns server with the given configuration.
    pub fn new() -> Self {
        Self {}
    }

    pub fn exec(
        &self,
        input_dir: String,
        port: u32,
        filter_search_domain: String,
    ) -> Result<(), Error> {
        // fork and verify if server is running
        // and exit parent
        // setsid() ensures that there is no controlling terminal on the child process

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                log::debug!("starting aardvark on a child with pid {}", child);
                // verify aardvark here and block till all the ip are ready
                match config::parse_configs(&input_dir) {
                    Ok((_, listen_ip_v4, listen_ip_v6)) => {
                        for (_, ip_list) in listen_ip_v4 {
                            for ip in ip_list {
                                serve::wait_till_aardvark_server_ready(
                                    std::net::IpAddr::V4(ip),
                                    port,
                                );
                            }
                        }
                        for (_, ip_list) in listen_ip_v6 {
                            for ip in ip_list {
                                serve::wait_till_aardvark_server_ready(
                                    std::net::IpAddr::V6(ip),
                                    port,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("unable to parse config: {}", e),
                        ))
                    }
                }

                Ok(())
            }
            Ok(ForkResult::Child) => {
                // remove any controlling terminals
                // but don't hardstop if this fails
                let _ = unsafe { libc::setsid() }; // check https://docs.rs/libc
                                                   // close fds -> stdout, stdin and stderr
                let dev_null = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("/dev/null")
                    .map_err(|e| std::io::Error::new(e.kind(), format!("/dev/null: {}", e)));
                // redirect stdout, stdin and stderr to /dev/null
                if let Ok(dev_null) = dev_null {
                    let fd = dev_null.as_raw_fd();
                    let _ = dup2(fd, 0);
                    let _ = dup2(fd, 1);
                    let _ = dup2(fd, 2);
                    if fd < 2 {
                        std::mem::forget(dev_null);
                    }
                }

                if let Err(er) = serve::serve(&input_dir, port, &filter_search_domain) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error starting server {}", er),
                    ));
                }
                Ok(())
            }
            Err(err) => {
                log::debug!("fork failed with error {}", err);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("fork failed with error: {}", err),
                ));
            }
        }
    }
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}
