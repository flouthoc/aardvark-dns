//! Runs the aardvark dns server with provided config
use crate::config;
use crate::server::serve;
use clap::Parser;
use log::debug;
use nix::unistd::{fork, ForkResult};
use std::io::Error;

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
                debug!(
                    "Setting up aardvark server with input directory as {:?}",
                    input_dir
                );
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
