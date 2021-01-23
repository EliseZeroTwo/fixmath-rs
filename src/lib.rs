#[allow(dead_code)]

pub fn no_rounding() -> bool {
    #[cfg(feature = "no-rounding")]
    return true;
    false
}

pub mod fix16;

#[cfg(test)]
mod tests {
    use crate::fix16::Fix16;

    #[test]
    fn it_works() {
        
    }
}
