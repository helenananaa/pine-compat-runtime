use crate::signature::{Accepts, BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};
use pine_ir::ValueKind;

const CURRENCY_RATE_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "from",
        accepts: Accepts::Kind(ValueKind::String),
        optional: false,
    },
    BuiltinParam {
        name: "to",
        accepts: Accepts::Kind(ValueKind::String),
        optional: false,
    },
    BuiltinParam {
        name: "ignore_invalid_currency",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
];

const REQUEST_SECURITY_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "symbol",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "timeframe",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "expression",
        accepts: Accepts::Any,
        optional: false,
    },
    BuiltinParam {
        name: "gaps",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "lookahead",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "ignore_invalid_symbol",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "currency",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "calc_bars_count",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
];

const REQUEST_SECURITY_LOWER_TF_PARAMS: &[BuiltinParam] = &[
    BuiltinParam {
        name: "symbol",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "timeframe",
        accepts: Accepts::SimpleString,
        optional: false,
    },
    BuiltinParam {
        name: "expression",
        accepts: Accepts::Any,
        optional: false,
    },
    BuiltinParam {
        name: "ignore_invalid_symbol",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "currency",
        accepts: Accepts::SimpleString,
        optional: true,
    },
    BuiltinParam {
        name: "ignore_invalid_timeframe",
        accepts: Accepts::BoolCompatible,
        optional: true,
    },
    BuiltinParam {
        name: "calc_bars_count",
        accepts: Accepts::SimpleInt,
        optional: true,
    },
];

pub(crate) const SIGNATURES: &[BuiltinSignature] = &[
    BuiltinSignature {
        name: "request.currency_rate",
        phase: BuiltinPhase::Phase1Core,
        params: CURRENCY_RATE_PARAMS,
        returns: ReturnSpec::Fixed(crate::namespaces::types::SERIES_FLOAT),
        variadic: false,
    },
    BuiltinSignature {
        name: "request.security",
        phase: BuiltinPhase::Phase1Core,
        params: REQUEST_SECURITY_PARAMS,
        returns: ReturnSpec::SeriesFromArg(2),
        variadic: false,
    },
    BuiltinSignature {
        name: "request.security_lower_tf",
        phase: BuiltinPhase::Phase1Core,
        params: REQUEST_SECURITY_LOWER_TF_PARAMS,
        returns: ReturnSpec::SeriesFromArg(2),
        variadic: false,
    },
];
