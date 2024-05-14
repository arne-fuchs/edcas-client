use crate::edcas::backend::evm::edcas_contract::Floating;
use std::str::FromStr;
use log::debug;

/**
    Takes the decimal point and the floating point of a Floating Struct of the edcas_contract and converts it as f64
    This function is not efficient, because it works with strings.
*/
pub fn floating_to_f64(decimal: i128, floating_point: u8) -> f64 {
    let mut eccentricity = "".to_string();
    if decimal < 0 {
        eccentricity.push('-');
    }
    for _ in 0..floating_point {
        eccentricity.push('0');
    }
    eccentricity.push_str(decimal.abs().to_string().as_str());
    eccentricity.insert(eccentricity.len() - floating_point as usize, '.');
    f64::from_str(eccentricity.as_str()).unwrap()
}

/**
   Takes a string like "0.234" and converts it to floating
*/
pub fn generate_floating_from_string(decimal: String) -> Floating {
    Floating {
        decimal: decimal.replace('.', "").parse().unwrap_or(0),
        floating_point: decimal.split('.').nth(1).unwrap_or("").len() as u8,
    }
}
