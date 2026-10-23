//! Register strategy directives before the first native market observation.
use pine_ir::ScriptMode;

use crate::{HistoricalRuntime, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(crate) fn initialize_native_risk(&mut self) -> Result<(), RuntimeError> {
        if self.native_risk_initialized
            || self.external_execution.is_some()
            || self.program.script_mode != ScriptMode::Strategy
        {
            return Ok(());
        }
        let configuration = crate::input_metadata::native_risk_configuration(
            &self.program,
            &self.input_overrides,
            &self.request_environment,
            self.execution_limits,
        );
        if !configuration.unresolved_call_site_ids.is_empty() {
            return Err(RuntimeError {
                message: format!(
                    "E_NATIVE_RISK_UNSUPPORTED: native risk directives require fully resolved immutable configuration; unresolved call sites: {:?}",
                    configuration.unresolved_call_site_ids
                ),
            });
        }
        self.strategy_broker.configure_native_risk(&configuration);
        self.native_risk_initialized = true;
        Ok(())
    }
}
