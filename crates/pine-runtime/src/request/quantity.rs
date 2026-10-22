//! Exact decimal exchange quantity increments, independent of price and money.

pub const CONTRACT_METADATA_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantityStep {
    coefficient: u64,
    decimals: u32,
    value_bits: u64,
}

const INVALID: &str = "quantity rule must be a positive decimal with at most 18 fractional places and a u64 coefficient, preserving its shortest float decimal";

impl QuantityStep {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        let (coefficient, decimals) = decimal_parts(value)?;
        let text = decimal_text(coefficient.to_string().into_bytes(), decimals, false);
        let number: f64 = text.parse().map_err(|_| INVALID)?;
        if !number.is_finite()
            || number <= 0.
            || decimal_parts(&number.to_string())? != (coefficient, decimals)
        {
            return Err(INVALID);
        }
        Ok(Self {
            coefficient,
            decimals,
            value_bits: number.to_bits(),
        })
    }

    #[must_use]
    pub fn value(self) -> f64 {
        f64::from_bits(self.value_bits)
    }

    pub(crate) fn decimals(self) -> u32 {
        self.decimals
    }

    pub(crate) fn truncate(self, quantity: f64) -> f64 {
        self.round(quantity, false)
    }

    pub(crate) fn floor(self, quantity: f64) -> f64 {
        self.round(quantity, true)
    }

    fn units(self, quantity: f64) -> (Vec<u8>, bool, u64) {
        // f64 Display uses ordinary decimal notation, including small values.
        let text = quantity.abs().to_string();
        let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
        let places = self.decimals as usize;
        let mut units = Vec::with_capacity(whole.len() + places);
        units.extend_from_slice(whole.as_bytes());
        units.extend_from_slice(&fraction.as_bytes()[..fraction.len().min(places)]);
        units.resize(whole.len() + places, b'0');
        let discarded = fraction.as_bytes()[fraction.len().min(places)..]
            .iter()
            .any(|digit| *digit != b'0');
        let mut remainder = 0_u128;
        for &digit in &units {
            remainder = (remainder * 10 + u128::from(digit - b'0')) % u128::from(self.coefficient);
        }
        (units, discarded, remainder as u64)
    }

    fn round(self, quantity: f64, floor: bool) -> f64 {
        if !quantity.is_finite() || quantity == 0. {
            return quantity;
        }
        let (mut units, discarded, remainder) = self.units(quantity);
        let mut borrow = u128::from(remainder);
        for digit in units.iter_mut().rev() {
            let subtraction = (borrow % 10) as u8;
            borrow /= 10;
            if *digit - b'0' < subtraction {
                *digit += 10;
                borrow += 1;
            }
            *digit -= subtraction;
        }
        debug_assert_eq!(borrow, 0);
        if floor && quantity < 0. && (discarded || remainder != 0) {
            let mut carry = u128::from(self.coefficient);
            for digit in units.iter_mut().rev() {
                let sum = u128::from(*digit - b'0') + carry;
                *digit = b'0' + (sum % 10) as u8;
                carry = sum / 10;
            }
            if carry != 0 {
                let mut prefix = carry.to_string().into_bytes();
                prefix.extend(units);
                units = prefix;
            }
        }
        let text = decimal_text(units, self.decimals, quantity < 0.);
        let value: f64 = text.parse().expect("decimal quantity");
        // A float transport must retain a grid-aligned shortest decimal. An
        // unrepresentable huge quantity fails closed, rather than emitting a
        // value that a decimal host would reject or round in another direction.
        if !value.is_finite() {
            return f64::NAN;
        }
        let (_, discarded, remainder) = self.units(value);
        if discarded || remainder != 0 {
            f64::NAN
        } else {
            value
        }
    }
}

fn decimal_parts(value: &str) -> Result<(u64, u32), &'static str> {
    if value.is_empty() || value.len() > 80 || value.trim() != value {
        return Err(INVALID);
    }
    let value = value.strip_prefix('+').unwrap_or(value);
    let (mantissa, exponent) = if let Some(index) = value.find(['e', 'E']) {
        let exponent = value[index + 1..].parse::<i32>().map_err(|_| INVALID)?;
        (&value[..index], exponent)
    } else {
        (value, 0)
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits = format!("{whole}{fraction}");
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return Err(INVALID);
    }
    let mut places = (fraction.len() as i32)
        .checked_sub(exponent)
        .ok_or(INVALID)?;
    while places > 0 && digits.ends_with('0') {
        digits.pop();
        places -= 1;
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() || !(-19..=18).contains(&places) {
        return Err(INVALID);
    }
    let mut coefficient = digits.parse::<u64>().map_err(|_| INVALID)?;
    if places < 0 {
        coefficient = coefficient
            .checked_mul(10_u64.checked_pow((-places) as u32).ok_or(INVALID)?)
            .ok_or(INVALID)?;
        places = 0;
    }
    Ok((coefficient, places as u32))
}

fn decimal_text(mut units: Vec<u8>, places: u32, negative: bool) -> String {
    let places = places as usize;
    if units.len() <= places {
        let mut prefix = vec![b'0'; places + 1 - units.len()];
        prefix.extend(units);
        units = prefix;
    }
    if places != 0 {
        units.insert(units.len() - places, b'.');
    }
    if negative {
        units.insert(0, b'-');
    }
    String::from_utf8(units).expect("decimal ASCII")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuantityGrid {
    LegacyPrecision(u32),
    Decimal(QuantityStep),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct QuantityRules {
    pub(crate) grid: Option<QuantityGrid>,
    pub(crate) minimum: Option<QuantityStep>,
}

impl QuantityRules {
    #[cfg(test)]
    pub(crate) fn legacy(scale: Option<u32>) -> Self {
        Self {
            grid: scale.map(QuantityGrid::LegacyPrecision),
            minimum: None,
        }
    }

    pub(crate) fn default_quantity(self, quantity: f64) -> f64 {
        let rounded = match self.grid {
            None => quantity,
            Some(QuantityGrid::LegacyPrecision(scale)) => {
                (quantity * f64::from(scale)).floor() / f64::from(scale)
            }
            Some(QuantityGrid::Decimal(step)) => step.floor(quantity),
        };
        self.minimum_quantity(rounded)
    }

    pub(crate) fn explicit_quantity(self, quantity: f64) -> f64 {
        self.minimum_quantity(self.truncate(quantity, false))
    }

    pub(crate) fn margin_cover(self, quantity: f64) -> f64 {
        self.truncate(quantity, true)
    }

    pub(crate) fn reduction(self, quantity: f64, available: f64) -> f64 {
        if quantity >= available {
            available
        } else {
            self.explicit_quantity(quantity)
        }
    }

    fn minimum_quantity(self, quantity: f64) -> f64 {
        if self
            .minimum
            .is_some_and(|minimum| quantity.abs() < minimum.value())
        {
            0.
        } else {
            quantity
        }
    }

    fn truncate(self, quantity: f64, margin: bool) -> f64 {
        let grid = self
            .grid
            .or(margin.then_some(QuantityGrid::LegacyPrecision(1)));
        match grid {
            None => quantity,
            Some(QuantityGrid::Decimal(step)) => step.truncate(quantity),
            Some(QuantityGrid::LegacyPrecision(scale)) => {
                if margin {
                    return (quantity * f64::from(scale)).trunc() / f64::from(scale);
                }
                if !quantity.is_finite() {
                    return quantity;
                }
                let decimal = quantity.to_string();
                let Some(point) = decimal.find('.') else {
                    return quantity;
                };
                let precision = scale.ilog10() as usize;
                let end = if precision == 0 {
                    point
                } else {
                    (point + 1 + precision).min(decimal.len())
                };
                decimal[..end].parse().expect("truncated finite quantity")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_decimal_steps_are_canonical_and_reject_ambiguous_inputs() {
        assert_eq!(
            QuantityStep::parse("0.25").unwrap(),
            QuantityStep::parse("25e-2").unwrap()
        );
        assert_eq!(QuantityStep::parse("+0.2500").unwrap().value(), 0.25);
        assert_eq!(QuantityStep::parse("2e-12").unwrap().value(), 2e-12);
        for value in [
            "",
            "0",
            "-1",
            "NaN",
            "inf",
            "1/4",
            " 1",
            "1 ",
            "1e-19",
            "1e100",
            "1e-2147483648",
            "1.2.3",
            "1e1e1",
            "0.100000000000000001",
        ] {
            assert!(QuantityStep::parse(value).is_err(), "{value}");
        }
    }

    #[test]
    fn exact_rounding_matches_independent_decimal_oracles() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/quantity-grid-oracles.json"
        ))
        .unwrap();
        assert_eq!(fixture["cases"].as_array().unwrap().len(), 1026);
        for row in fixture["cases"].as_array().unwrap() {
            let quantity = row["qty"].as_f64().unwrap();
            let step = QuantityStep::parse(row["step"].as_str().unwrap()).unwrap();
            for (name, actual) in [
                ("toward_zero", step.truncate(quantity)),
                ("floor", step.floor(quantity)),
            ] {
                if let Some(expected) = row[name].as_f64() {
                    assert_eq!(actual, expected, "{row}: {name}");
                } else {
                    assert!(actual.is_nan(), "unrepresentable transport: {row}");
                }
            }
        }
    }

    #[test]
    fn minimum_is_independent_and_forced_cover_uses_only_the_increment() {
        let rules = QuantityRules {
            grid: Some(QuantityGrid::Decimal(QuantityStep::parse("0.25").unwrap())),
            minimum: Some(QuantityStep::parse("2").unwrap()),
        };
        assert_eq!(rules.explicit_quantity(1.99), 0.);
        assert_eq!(rules.explicit_quantity(2.49), 2.25);
        assert_eq!(rules.margin_cover(-0.49), -0.25);
        assert_eq!(rules.default_quantity(-2.49), -2.5);
    }
}
