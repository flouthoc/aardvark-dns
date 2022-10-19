//use super::*;

#[cfg(test)]
// perform unit tests for config, backend and lookup logic
// following tests will not test server and event loop since
// event-loop and server can be tested via integration tests
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use aardvark_dns::backend::DNSResult;
    use aardvark_dns::config;
    /* -------------------------------------------- */
    // --------- Test aardvark-dns config ---------
    /* -------------------------------------------- */
    #[test]
    // Test loading of config file from directory
    fn test_loading_config_file() {
        match config::parse_configs("src/test/config/podman") {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Test loading of config file from directory with custom DNS for containers
    fn test_loading_config_file_with_dns_servers() {
        match config::parse_configs("src/test/config/podman_custom_dns_servers") {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Parse config files from stub data
    fn test_parsing_config_files() {
        match config::parse_configs("src/test/config/podman") {
            Ok((_, listen_ip_v4, _)) => {
                listen_ip_v4.contains_key("podman");
                assert_eq!(listen_ip_v4["podman"].len(), 1);
                assert_eq!("10.88.0.1".parse(), Ok(listen_ip_v4["podman"][0]));
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Parse bad config files must fail
    fn test_parsing_bad_config_files() {
        match config::parse_configs("src/test/config/podman_bad_config") {
            Ok((_, _, _)) => panic!("parsing bad config must fail"),
            Err(_) => {}
        }
    }
    /* -------------------------------------------- */
    // -------Verify backend custom dns server ----
    /* -------------------------------------------- */
    #[test]
    // Backend must populate ctr_dns_servers via custom
    // DNS servers for container from the aardvark config
    fn test_backend_custom_dns_server() {
        match config::parse_configs("src/test/config/podman_custom_dns_servers") {
            Ok((backend, _, _)) => {
                // Should contain custom DNS server 8.8.8.8
                let mut dns_server = backend
                    .ctr_dns_server
                    .get(&IpAddr::V4(Ipv4Addr::new(10, 88, 0, 2)));
                let mut expected_dns_server = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
                assert_eq!(dns_server.unwrap().clone().unwrap()[0], expected_dns_server);

                // Should contain custom DNS servers 3.3.3.3 and 1.1.1.1
                dns_server = backend
                    .ctr_dns_server
                    .get(&IpAddr::V4(Ipv4Addr::new(10, 88, 0, 5)));
                expected_dns_server = IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3));
                assert_eq!(dns_server.unwrap().clone().unwrap()[0], expected_dns_server);
                expected_dns_server = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
                assert_eq!(dns_server.unwrap().clone().unwrap()[1], expected_dns_server);
                expected_dns_server = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
                assert_eq!(dns_server.unwrap().clone().unwrap()[2], expected_dns_server);

                // Shoudld not contain any DNS server
                dns_server = backend
                    .ctr_dns_server
                    .get(&IpAddr::V4(Ipv4Addr::new(10, 88, 0, 3)));
                assert_eq!(dns_server.unwrap().clone(), None);
            }
            Err(e) => panic!("{}", e),
        }
    }

    /* -------------------------------------------- */
    // -------Test aardvark-dns lookup logic ------
    /* -------------------------------------------- */
    #[test]
    // Check lookup query from backend and simulate
    // dns request from same container to itself but
    // aardvark must return one ip address i.e v4.
    // Request address must be v4.
    // Same container --> (resolve) Same container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_same_container_request_from_v4_on_v4_entries() {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend.lookup(&"10.88.0.2".parse().unwrap(), "condescendingnash") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(ip_vec.len(), 1);
                        assert_eq!("10.88.0.2".parse(), Ok(ip_vec[0]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // case-insensitive dns request from same container
    // to itself but aardvark must return one ip address i.e v4.
    // Request address must be v4.
    // Same container --> (resolve) Same container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_same_container_request_from_v4_on_v4_entries_case_insensitive(
    ) {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend.lookup(&"10.88.0.2".parse().unwrap(), "helloworld") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(ip_vec.len(), 1);
                        assert_eq!("10.88.0.5".parse(), Ok(ip_vec[0]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // case-insensitive dns request from same container
    // to itself but aardvark must return one ip address i.e v4.
    // Request address must be v4.
    // Same container --> (resolve) Same container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_same_container_request_from_v4_on_v4_entries_case_insensitive_uppercase(
    ) {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend.lookup(&"10.88.0.2".parse().unwrap(), "HELLOWORLD") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(ip_vec.len(), 1);
                        assert_eq!("10.88.0.5".parse(), Ok(ip_vec[0]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // nx_domain on bad lookup queries.
    fn test_lookup_queries_from_backend_simulate_nx_domain() {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend.lookup(&"10.88.0.2".parse().unwrap(), "somebadquery") {
                    DNSResult::NXDomain => {}
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from same container to itself but
    // aardvark must return one ip address i.e v4.
    // Request address must be v4.
    // Same container --> (resolve) different container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_different_container_request_from_v4() {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend.lookup(&"10.88.0.2".parse().unwrap(), "trustingzhukovsky") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(ip_vec.len(), 1);
                        assert_eq!("10.88.0.4".parse(), Ok(ip_vec[0]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from same container to itself but
    // aardvark must return one ip address i.e v4.
    // Request address must be v4.
    // Same container --> (resolve) different container name by alias --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_different_container_request_from_v4_by_alias() {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => match backend.lookup(&"10.88.0.2".parse().unwrap(), "ctr1") {
                DNSResult::Success(ip_vec) => {
                    // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                    assert_eq!(ip_vec.len(), 1);
                    assert_eq!("10.88.0.4".parse(), Ok(ip_vec[0]));
                }
                _ => panic!("unexpected dns result"),
            },
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from same container to itself but
    // aardvark must return two ip address for v4 and v6.
    // Request address must be v4.
    // Same container --> (resolve) Same container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_same_container_request_from_v4_on_v6_and_v4_entries(
    ) {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                match backend.lookup(&"10.89.0.2".parse().unwrap(), "test1") {
                    DNSResult::Success(ip_vec) => {
                        // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                        assert_eq!(ip_vec.len(), 2);
                        assert_eq!("10.89.0.2".parse(), Ok(ip_vec[0]));
                        assert_eq!("fdfd:733b:dc3:220b::2".parse(), Ok(ip_vec[1]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from same container to itself but
    // aardvark must return two ip address for v4 and v6.
    // Request address must be v6.
    // Same container --> (resolve) Same container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_same_container_request_from_v6_on_v6_and_v4_entries(
    ) {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                match backend.lookup(&"fdfd:733b:dc3:220b::2".parse().unwrap(), "test1") {
                    DNSResult::Success(ip_vec) => {
                        // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                        assert_eq!(ip_vec.len(), 2);
                        assert_eq!("10.89.0.2".parse(), Ok(ip_vec[0]));
                        assert_eq!("fdfd:733b:dc3:220b::2".parse(), Ok(ip_vec[1]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from container to another container but
    // aardvark must return two ip address for v4 and v6.
    // Request address must be v6.
    // Same container --> (resolve) different container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_different_container_request_from_v6_on_v6_and_v4_entries(
    ) {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                match backend.lookup(&"fdfd:733b:dc3:220b::2".parse().unwrap(), "test2") {
                    DNSResult::Success(ip_vec) => {
                        // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                        assert_eq!(ip_vec.len(), 2);
                        assert_eq!("10.89.0.3".parse(), Ok(ip_vec[0]));
                        assert_eq!("fdfd:733b:dc3:220b::3".parse(), Ok(ip_vec[1]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from container to another container but
    // aardvark must return two ip address for v4 and v6.
    // Request address must be v6.
    // Same container --> (resolve) different container name --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_different_container_request_from_v4_on_v6_and_v4_entries(
    ) {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                match backend.lookup(&"10.89.0.2".parse().unwrap(), "test2") {
                    DNSResult::Success(ip_vec) => {
                        // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                        assert_eq!(ip_vec.len(), 2);
                        assert_eq!("10.89.0.3".parse(), Ok(ip_vec[0]));
                        assert_eq!("fdfd:733b:dc3:220b::3".parse(), Ok(ip_vec[1]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check lookup query from backend and simulate
    // dns request from container to another container but
    // aardvark must return two ip address for v4 and v6.
    // Request address must be v6.
    // Same container --> (resolve) different container by id --> (on) Same Network
    fn test_lookup_queries_from_backend_simulate_different_container_request_by_id_from_v4_on_v6_and_v4_entries(
    ) {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                match backend.lookup(&"10.89.0.2".parse().unwrap(), "88dde8a24897") {
                    DNSResult::Success(ip_vec) => {
                        // verfiy length for issues like: https://github.com/containers/aardvark-dns/issues/120
                        assert_eq!(ip_vec.len(), 2);
                        assert_eq!("10.89.0.3".parse(), Ok(ip_vec[0]));
                        assert_eq!("fdfd:733b:dc3:220b::3".parse(), Ok(ip_vec[1]));
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    /* -------------------------------------------- */
    // ---Test aardvark-dns reverse lookup logic --
    /* -------------------------------------------- */
    #[test]
    // Check reverse lookup query from backend and simulate
    // dns request from same container to itself by IP
    // aardvark must return container name and alias
    // Same container --> (resolve) Same ip  --> (on) Same Network
    fn test_reverse_lookup_queries_from_backend_by_ip_v4() {
        match config::parse_configs("src/test/config/podman") {
            Ok((backend, _, _)) => {
                match backend
                    .reverse_lookup(&"10.88.0.4".parse().unwrap(), &"10.88.0.4".parse().unwrap())
                {
                    Some(lookup_vec) => {
                        assert_eq!(
                            &vec![
                                "trustingzhukovsky".to_string(),
                                "ctr1".to_string(),
                                "ctra".to_string()
                            ],
                            lookup_vec
                        );
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check reverse lookup query from backend and simulate
    // dns request from same container to itself by IP
    // aardvark must return container name and alias
    // Same container --> (resolve) Same ip  --> (on) Same Network
    fn test_reverse_lookup_queries_from_backend_by_ip_v6() {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, _, _)) => {
                match backend.reverse_lookup(
                    &"fdfd:733b:dc3:220b::2".parse().unwrap(),
                    &"fdfd:733b:dc3:220b::2".parse().unwrap(),
                ) {
                    Some(lookup_vec) => {
                        assert_eq!(
                            &vec!["test1".to_string(), "7b46c7ad93fc".to_string()],
                            lookup_vec
                        );
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
    /* -------------------------------------------- */
    // ---------Test aardvark-dns backend ---------
    /* -------------------------------------------- */
    #[test]
    // Check ip_mappings generated by backend
    fn test_generated_ip_mappings_in_backend() {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                backend
                    .ip_mappings
                    .contains_key(&"fdfd:733b:dc3:220b::2".parse().unwrap());
                backend
                    .ip_mappings
                    .contains_key(&"10.89.0.3".parse().unwrap());
                assert_eq!(
                    vec!["podman_v6_entries"],
                    backend.ip_mappings[&"fdfd:733b:dc3:220b::2".parse().unwrap()]
                );
                assert_eq!(
                    vec!["podman_v6_entries"],
                    backend.ip_mappings[&"10.89.0.3".parse().unwrap()]
                );
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check name_mappings generated by backend
    fn test_generated_name_mappings_in_backend() {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                // check if contains key
                backend.name_mappings.contains_key("podman_v6_entries");
                // container id must be in name entries
                backend.name_mappings["podman_v6_entries"].contains_key("7b46c7ad93fc");
                backend.name_mappings["podman_v6_entries"].contains_key("88dde8a24897");
                // container names must be in name entries
                backend.name_mappings["podman_v6_entries"].contains_key("test1");
                backend.name_mappings["podman_v6_entries"].contains_key("test2");
                assert_eq!(
                    "10.89.0.3".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["test2"][0])
                );
                assert_eq!(
                    "fdfd:733b:dc3:220b::3".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["test2"][1])
                );
                // name entries must contain all ip addresses for container test1
                assert_eq!(
                    "10.89.0.2".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["test1"][0])
                );
                assert_eq!(
                    "fdfd:733b:dc3:220b::2".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["test1"][1])
                );
                // name entries must contain all ip addresses for container with id 7b46c7ad93fc
                assert_eq!(
                    "10.89.0.2".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["7b46c7ad93fc"][0])
                );
                assert_eq!(
                    "fdfd:733b:dc3:220b::2".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["7b46c7ad93fc"][1])
                );
                // name entries must contain all ip addresses for container with id 88dde8a24897
                assert_eq!(
                    "10.89.0.3".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["88dde8a24897"][0])
                );
                assert_eq!(
                    "fdfd:733b:dc3:220b::3".parse(),
                    Ok(backend.name_mappings["podman_v6_entries"]["88dde8a24897"][1])
                );
            }
            Err(e) => panic!("{}", e),
        }
    }
    #[test]
    // Check reverse_mappings generated by backend
    fn test_generated_reverse_mappings_in_backend() {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                listen_ip_v6.contains_key("podman_v6_entries");
                listen_ip_v4.contains_key("podman_v6_entries");
                // all ips must have reverse lookups
                backend.reverse_mappings["podman_v6_entries"]
                    .contains_key(&"10.89.0.3".parse().unwrap());
                backend.reverse_mappings["podman_v6_entries"]
                    .contains_key(&"10.89.0.2".parse().unwrap());
                backend.reverse_mappings["podman_v6_entries"]
                    .contains_key(&"fdfd:733b:dc3:220b::2".parse().unwrap());
                backend.reverse_mappings["podman_v6_entries"]
                    .contains_key(&"fdfd:733b:dc3:220b::3".parse().unwrap());
            }
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    // Parse a config which contains multiple ipv4 and ipv6 addresses ona single line
    fn test_parse_multiple_ipv4_ipv6_addresses() {
        match config::parse_configs("src/test/config/podman_v6_entries") {
            Ok((backend, listen_ip_v4, listen_ip_v6)) => {
                assert_eq!(
                    listen_ip_v4["podman_v6_entries_proper"],
                    vec![
                        "10.0.0.1".parse::<Ipv4Addr>().unwrap(),
                        "10.0.1.1".parse().unwrap()
                    ]
                );
                assert_eq!(
                    listen_ip_v6["podman_v6_entries_proper"],
                    vec![
                        "fdfd::1".parse::<Ipv6Addr>().unwrap(),
                        "fddd::1".parse().unwrap()
                    ]
                );
                match backend.lookup(&"10.0.0.2".parse().unwrap(), "testmulti1") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(
                            ip_vec,
                            vec![
                                "10.0.0.2".parse::<IpAddr>().unwrap(),
                                "10.0.1.2".parse().unwrap(),
                                "fdfd::2".parse().unwrap(),
                                "fddd::2".parse().unwrap()
                            ]
                        )
                    }
                    _ => panic!("unexpected dns result"),
                }

                match backend.lookup(&"10.0.0.2".parse().unwrap(), "testmulti2") {
                    DNSResult::Success(ip_vec) => {
                        assert_eq!(
                            ip_vec,
                            vec![
                                "10.0.0.3".parse::<IpAddr>().unwrap(),
                                "10.0.1.3".parse().unwrap(),
                                "fdfd::3".parse().unwrap(),
                                "fddd::3".parse().unwrap()
                            ]
                        )
                    }
                    _ => panic!("unexpected dns result"),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
