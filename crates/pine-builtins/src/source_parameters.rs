/// Pine source positions differ from the stable normalized runtime slots.
pub fn strategy_exit_source_parameter(index: usize, version: u16) -> Option<&'static str> {
    const COMMON: &[&str] = &[
        "id",
        "from_entry",
        "qty",
        "qty_percent",
        "profit",
        "limit",
        "loss",
        "stop",
        "trail_price",
        "trail_points",
        "trail_offset",
        "oca_name",
        "comment",
    ];
    const LEGACY: &[&str] = &[
        "when",
        "alert_message",
        "comment_profit",
        "comment_loss",
        "comment_trailing",
        "alert_profit",
        "alert_loss",
        "alert_trailing",
        "disable_alert",
    ];
    const MODERN: &[&str] = &[
        "comment_profit",
        "comment_loss",
        "comment_trailing",
        "alert_message",
        "alert_profit",
        "alert_loss",
        "alert_trailing",
        "disable_alert",
    ];
    if index < COMMON.len() {
        return COMMON.get(index).copied();
    }
    let tail = if version >= 5 { MODERN } else { LEGACY };
    tail.get(index - COMMON.len()).copied()
}

/// Source order includes `when` before Pine v6; normalized runtime slots do not.
pub fn strategy_close_source_parameter(index: usize, version: u16) -> Option<&'static str> {
    const LEGACY: &[&str] = &[
        "id",
        "when",
        "comment",
        "qty",
        "qty_percent",
        "alert_message",
    ];
    const V5: &[&str] = &[
        "id",
        "when",
        "comment",
        "qty",
        "qty_percent",
        "alert_message",
        "immediately",
        "disable_alert",
    ];
    const V6: &[&str] = &[
        "id",
        "comment",
        "qty",
        "qty_percent",
        "alert_message",
        "immediately",
        "disable_alert",
    ];
    let parameters = match version {
        1..=4 => LEGACY,
        5 => V5,
        _ => V6,
    };
    parameters.get(index).copied()
}
