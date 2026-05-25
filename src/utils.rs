use libc::{getprotobynumber, getservbyport, protoent, servent};
use std::os::raw::c_int;
use std::ptr::null;

const K: u64 = 10u64.pow(3);
const M: u64 = 10u64.pow(6);
const G: u64 = 10u64.pow(9);
const T: u64 = 10u64.pow(12);

pub fn get_protocol_from_number(number: u8) -> Option<String> {
    unsafe {
        let protocol: *mut protoent = getprotobynumber(number as c_int);
        if protocol.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr((*protocol).p_name)
            .to_string_lossy()
            .into_owned();
        Some(name)
    }
}

pub fn get_service_from_port(port: u16, protocol: Option<String>) -> Option<String> {
    let proto_c = protocol.and_then(|proto| std::ffi::CString::new(proto).ok());
    let protocol_ptr = proto_c
        .as_ref()
        .map_or(null(), |c_string| c_string.as_ptr());

    unsafe {
        let service: *mut servent = getservbyport(port.to_be().into(), protocol_ptr);
        if service.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr((*service).s_name)
            .to_string_lossy()
            .into_owned();
        Some(name)
    }
}

pub fn humanize(value: u64) -> String {
    match value {
        0..K => format!("{}", value),
        K..M => format!("{:.1} k", value as f64 / K as f64),
        M..G => format!("{:.1} M", value as f64 / M as f64),
        G..T => format!("{:.1} B", value as f64 / G as f64),
        T.. => format!("{:.1} T", value as f64 / T as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(1, "icmp" ; "icmp")]
    #[test_case(17, "udp" ; "udp")]
    fn test_get_valid_proto(number: u8, kind: &str) {
        assert_eq!(get_protocol_from_number(number), Some(kind.to_string()));
    }

    #[test]
    fn test_get_invalid_proto() {
        assert_eq!(get_protocol_from_number(179), None);
    }

    #[test_case(443, "tcp", "https" ; "443/tcp")]
    #[test_case(443, "udp", "https" ; "443/udp")]
    #[test_case(80, "tcp", "http" ; "80/tcp")]
    fn test_get_valid_service_explicit(port: u16, protocol: &str, service: &str) {
        assert_eq!(
            get_service_from_port(port, Some(protocol.to_string())),
            Some(service.to_string())
        );
    }

    #[test_case(443, "https" ; "443")]
    #[test_case(80, "http" ; "80")]
    fn test_get_valid_service_implicit(port: u16, service: &str) {
        assert_eq!(get_service_from_port(port, None), Some(service.to_string()));
    }

    #[test]
    fn test_get_invalid_service() {
        assert_eq!(get_service_from_port(17910, None), None);
        assert_eq!(get_service_from_port(17910, Some("tcp".to_string())), None);
        assert_eq!(get_service_from_port(17910, Some("udp".to_string())), None);
    }

    #[test_case(123, "123" ; "hundreds")]
    #[test_case(123456, "123.5 k" ; "thousands")]
    #[test_case(123456789, "123.5 M" ; "millions")]
    #[test_case(123456789012, "123.5 B" ; "billions")]
    #[test_case(123456789012345, "123.5 T" ; "trillions")]
    fn test_humanize(number: u64, pretty: &str) {
        assert_eq!(humanize(number), pretty.to_string());
    }
}
