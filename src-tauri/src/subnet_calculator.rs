use std::net::Ipv4Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubnetCalculationError {
    MissingSlash,
    InvalidIpv4Address,
    InvalidPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Subnet {
    pub address: Ipv4Addr,
    pub prefix_length: u8,
    pub network_address: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub wildcard_mask: Ipv4Addr,
    /// Traditional broadcast address; absent for /31 and /32.
    pub broadcast_address: Option<Ipv4Addr>,
    pub first_usable_address: Ipv4Addr,
    pub last_usable_address: Ipv4Addr,
    pub total_addresses: u64,
    pub usable_host_count: u64,
}

/// Calculates a single IPv4 CIDR locally; it performs no network operations.
pub fn calculate_ipv4_subnet(cidr: &str) -> Result<Ipv4Subnet, SubnetCalculationError> {
    let (address_text, prefix_text) = cidr
        .split_once('/')
        .ok_or(SubnetCalculationError::MissingSlash)?;
    if prefix_text.contains('/') {
        return Err(SubnetCalculationError::InvalidPrefix);
    }
    let address = address_text
        .parse::<Ipv4Addr>()
        .map_err(|_| SubnetCalculationError::InvalidIpv4Address)?;
    let prefix_length = prefix_text
        .parse::<u8>()
        .ok()
        .filter(|prefix| *prefix <= 32)
        .ok_or(SubnetCalculationError::InvalidPrefix)?;

    let address_bits = u32::from(address);
    let mask_bits = if prefix_length == 0 {
        0
    } else {
        u32::MAX << (32 - prefix_length)
    };
    let network_bits = address_bits & mask_bits;
    let broadcast_bits = network_bits | !mask_bits;
    let total_addresses = 1_u64 << (32 - prefix_length);
    let (first_bits, last_bits, usable_host_count, broadcast_address) = match prefix_length {
        0..=30 => (
            network_bits + 1,
            broadcast_bits - 1,
            total_addresses - 2,
            Some(Ipv4Addr::from(broadcast_bits)),
        ),
        31 => (network_bits, broadcast_bits, 2, None),
        _ => (network_bits, network_bits, 1, None),
    };

    Ok(Ipv4Subnet {
        address,
        prefix_length,
        network_address: Ipv4Addr::from(network_bits),
        netmask: Ipv4Addr::from(mask_bits),
        wildcard_mask: Ipv4Addr::from(!mask_bits),
        broadcast_address,
        first_usable_address: Ipv4Addr::from(first_bits),
        last_usable_address: Ipv4Addr::from(last_bits),
        total_addresses,
        usable_host_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{calculate_ipv4_subnet, SubnetCalculationError as Error};
    use std::net::Ipv4Addr;

    #[test]
    fn calculates_standard_ipv4_subnet_fields() {
        let subnet = calculate_ipv4_subnet("192.168.10.99/24").unwrap();
        assert_eq!(subnet.address, Ipv4Addr::new(192, 168, 10, 99));
        assert_eq!(subnet.network_address, Ipv4Addr::new(192, 168, 10, 0));
        assert_eq!(subnet.netmask, Ipv4Addr::new(255, 255, 255, 0));
        assert_eq!(subnet.wildcard_mask, Ipv4Addr::new(0, 0, 0, 255));
        assert_eq!(
            subnet.broadcast_address,
            Some(Ipv4Addr::new(192, 168, 10, 255))
        );
        assert_eq!(subnet.first_usable_address, Ipv4Addr::new(192, 168, 10, 1));
        assert_eq!(subnet.last_usable_address, Ipv4Addr::new(192, 168, 10, 254));
        assert_eq!(subnet.total_addresses, 256);
        assert_eq!(subnet.usable_host_count, 254);
    }

    #[test]
    fn handles_zero_thirty_one_and_thirty_two_prefixes() {
        let zero = calculate_ipv4_subnet("10.1.2.3/0").unwrap();
        assert_eq!(zero.network_address, Ipv4Addr::UNSPECIFIED);
        assert_eq!(zero.broadcast_address, Some(Ipv4Addr::BROADCAST));
        assert_eq!(zero.total_addresses, 1_u64 << 32);
        assert_eq!(zero.usable_host_count, (1_u64 << 32) - 2);

        let point_to_point = calculate_ipv4_subnet("192.0.2.7/31").unwrap();
        assert_eq!(point_to_point.network_address, Ipv4Addr::new(192, 0, 2, 6));
        assert_eq!(point_to_point.broadcast_address, None);
        assert_eq!(
            point_to_point.first_usable_address,
            Ipv4Addr::new(192, 0, 2, 6)
        );
        assert_eq!(
            point_to_point.last_usable_address,
            Ipv4Addr::new(192, 0, 2, 7)
        );
        assert_eq!(point_to_point.usable_host_count, 2);

        let host = calculate_ipv4_subnet("192.0.2.7/32").unwrap();
        assert_eq!(host.network_address, Ipv4Addr::new(192, 0, 2, 7));
        assert_eq!(host.broadcast_address, None);
        assert_eq!(host.usable_host_count, 1);
    }

    #[test]
    fn rejects_invalid_ipv4_cidr_inputs_explicitly() {
        assert_eq!(calculate_ipv4_subnet("192.0.2.1"), Err(Error::MissingSlash));
        assert_eq!(
            calculate_ipv4_subnet("example.com/24"),
            Err(Error::InvalidIpv4Address)
        );
        assert_eq!(
            calculate_ipv4_subnet("2001:db8::1/64"),
            Err(Error::InvalidIpv4Address)
        );
        for cidr in [
            "192.0.2.1/",
            "192.0.2.1/-1",
            "192.0.2.1/33",
            "192.0.2.1/24/1",
        ] {
            assert_eq!(
                calculate_ipv4_subnet(cidr),
                Err(Error::InvalidPrefix),
                "{cidr}"
            );
        }
    }
}
