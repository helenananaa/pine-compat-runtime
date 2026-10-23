use super::{QuantityGrid, QuantityRules, QuantityStep, RequestTimeframe};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartContext {
    symbol: String,
    timeframe: RequestTimeframe,
    min_move: u32,
    price_scale: u32,
    quantity_scale: u32,
    quantity_precision_configured: bool,
    quantity_step: Option<QuantityStep>,
    minimum_contract: Option<QuantityStep>,
    point_value_bits: u64,
    currency: String,
}

impl ChartContext {
    #[must_use]
    pub fn new(symbol: impl Into<String>, timeframe: RequestTimeframe) -> Self {
        Self {
            symbol: symbol.into(),
            timeframe,
            min_move: 1,
            price_scale: 100,
            quantity_scale: 1,
            quantity_precision_configured: false,
            quantity_step: None,
            minimum_contract: None,
            point_value_bits: 1.0_f64.to_bits(),
            currency: "USD".to_owned(),
        }
    }

    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// The host supplies the symbol's quote currency; no FX lookup is performed.
    pub fn with_currency(mut self, currency: impl Into<String>) -> Result<Self, &'static str> {
        let currency = currency.into();
        if currency.is_empty() || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err("chart currency must be a nonempty uppercase currency code");
        }
        self.currency = currency;
        Ok(self)
    }

    #[must_use]
    pub fn timeframe(&self) -> &RequestTimeframe {
        &self.timeframe
    }

    /// Configure the host-provided price grid. The core never looks up instruments.
    pub fn with_price_grid(
        mut self,
        min_move: u32,
        price_scale: u32,
    ) -> Result<Self, &'static str> {
        if min_move == 0 || price_scale == 0 {
            return Err("chart minMove and priceScale must be positive integers");
        }
        self.min_move = min_move;
        self.price_scale = price_scale;
        Ok(self)
    }

    #[must_use]
    pub fn min_move(&self) -> u32 {
        self.min_move
    }

    #[must_use]
    pub fn price_scale(&self) -> u32 {
        self.price_scale
    }

    #[must_use]
    pub fn min_tick(&self) -> f64 {
        f64::from(self.min_move) / f64::from(self.price_scale)
    }

    /// Decimal quantity precision supplied by the host (0 through 9).
    /// This profile represents a minimum contract of 10^-precision units.
    pub fn with_quantity_precision(mut self, precision: u32) -> Result<Self, &'static str> {
        if self
            .quantity_step
            .is_some_and(|step| step.decimals() > precision)
        {
            return Err("quantityStep exceeds the supplied quantityPrecision");
        }
        self.quantity_scale = 10_u32
            .checked_pow(precision)
            .ok_or("chart quantity precision must be between 0 and 9")?;
        self.quantity_precision_configured = true;
        Ok(self)
    }

    #[must_use]
    pub fn min_contract(&self) -> f64 {
        self.minimum_contract
            .or(self.quantity_step)
            .map_or_else(|| 1.0 / f64::from(self.quantity_scale), QuantityStep::value)
    }

    /// Exact contract increment, independent of the exchange's minimum order.
    pub fn with_quantity_step(mut self, value: &str) -> Result<Self, &'static str> {
        let step = QuantityStep::parse(value)?;
        if self.quantity_precision_configured && step.decimals() > self.quantity_scale.ilog10() {
            return Err("quantityStep exceeds the supplied quantityPrecision");
        }
        self.quantity_step = Some(step);
        Ok(self)
    }

    /// Minimum positive order quantity. Forced reductions still use the grid.
    pub fn with_min_contract(mut self, value: &str) -> Result<Self, &'static str> {
        self.minimum_contract = Some(QuantityStep::parse(value)?);
        Ok(self)
    }

    pub(crate) fn quantity_rules(&self) -> QuantityRules {
        QuantityRules {
            grid: self.quantity_step.map(QuantityGrid::Decimal).or_else(|| {
                self.configured_quantity_scale()
                    .map(QuantityGrid::LegacyPrecision)
            }),
            minimum: self.minimum_contract,
        }
    }

    /// Money in quote currency represented by one price point per contract.
    pub fn with_point_value(mut self, value: f64) -> Result<Self, &'static str> {
        if !value.is_finite() || value <= 0.0 {
            return Err("chart pointValue must be positive and finite");
        }
        // Validated positive finite bits preserve exact ChartContext equality.
        self.point_value_bits = value.to_bits();
        Ok(self)
    }

    #[must_use]
    pub fn point_value(&self) -> f64 {
        f64::from_bits(self.point_value_bits)
    }

    pub(crate) fn quantity_scale(&self) -> u32 {
        self.quantity_scale
    }

    pub(crate) fn configured_quantity_scale(&self) -> Option<u32> {
        self.quantity_precision_configured
            .then_some(self.quantity_scale)
    }

    #[must_use]
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = symbol.into();
        self
    }

    #[must_use]
    pub fn with_timeframe(mut self, timeframe: RequestTimeframe) -> Self {
        self.timeframe = timeframe;
        self
    }
}

impl Default for ChartContext {
    fn default() -> Self {
        Self {
            symbol: "NASDAQ:AAPL".to_owned(),
            timeframe: RequestTimeframe::default(),
            min_move: 1,
            price_scale: 100,
            quantity_scale: 1,
            quantity_precision_configured: false,
            quantity_step: None,
            minimum_contract: None,
            point_value_bits: 1.0_f64.to_bits(),
            currency: "USD".to_owned(),
        }
    }
}
