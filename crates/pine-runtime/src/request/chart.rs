use super::RequestTimeframe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartContext {
    symbol: String,
    timeframe: RequestTimeframe,
    min_move: u32,
    price_scale: u32,
    quantity_scale: u32,
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
        }
    }

    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
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
        self.quantity_scale = 10_u32
            .checked_pow(precision)
            .ok_or("chart quantity precision must be between 0 and 9")?;
        Ok(self)
    }

    #[must_use]
    pub fn min_contract(&self) -> f64 {
        1.0 / f64::from(self.quantity_scale)
    }

    pub(crate) fn quantity_scale(&self) -> u32 {
        self.quantity_scale
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
        }
    }
}
