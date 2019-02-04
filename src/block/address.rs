extern crate ring;

#[derive(Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Address(pub [u8; 20]);

impl std::convert::From<[u8; 20]> for Address {
    fn from(input: [u8; 20]) -> Address {
        return Address(input);
    }
}

impl std::convert::From<Address> for [u8; 20] {
    fn from(input: Address) -> [u8; 20] {
        return input.0;
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        for byte_idx in 0..20 {
            if self.0[byte_idx] != other.0[byte_idx] {
                return false;
            }
        }
        return true;
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..20 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Address;

    #[test]
    fn eq() {
        let some_addr = Address(hex!("0000000000111111111122222222223333333333"));
        let same_addr = Address(hex!("0000000000111111111122222222223333333333"));
        assert_eq!(some_addr == same_addr, true);

        let some_addr = Address(hex!("0000000000111111111122222222223333333333"));
        let other_addr = Address(hex!("1234000000111111111122222222223333333333"));
        assert_eq!(some_addr == other_addr, false);
    }

    #[test]
    fn from_u8() {
        let source = hex!("1122334455112233445511223344551122334455");
        let should_be = Address(hex!("1122334455112233445511223344551122334455"));
        let result: Address = Address::from(source);
        assert_eq!(should_be, result);
    }

    #[test]
    fn into_u8() {
        let should_be = hex!("1122334455112233445511223344551122334455");
        let source = Address(hex!("1122334455112233445511223344551122334455"));
        let result: [u8; 20] = source.into();
        assert_eq!(should_be, result);
    }
}
