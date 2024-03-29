pub fn mac_address_to_bytes(mac_address: &str) -> Option<Vec<u8>> {
    let parts: Vec<&str> = mac_address.split(':').collect();

    if parts.len() != 6 {
        return None; // MAC address should have 6 segments separated by ':'
    }

    let mut bytes = Vec::with_capacity(6);
    for part in parts {
        match u8::from_str_radix(part, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return None, // Invalid hexadecimal digit
        }
    }

    Some(bytes)
}
