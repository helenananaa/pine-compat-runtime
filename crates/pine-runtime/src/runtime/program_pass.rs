use crate::runtime::statements::StmtControl;
use crate::{HistoricalRuntime, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(crate) fn run_program_statements(&mut self) -> Result<(), RuntimeError> {
        self.pending_alert_events.clear();
        let program = self.program.clone();
        for statement in &program.statements {
            match self.eval_stmt(statement) {
                Ok(StmtControl::None) => {}
                Ok(StmtControl::Break | StmtControl::Continue) => {
                    return Err(RuntimeError::escaped_loop_control());
                }
                Err(_) if self.pending_loop_control.take().is_some() => {
                    return Err(RuntimeError::escaped_loop_control());
                }
                Err(error) => return Err(error),
            }
        }
        self.snapshot_persistent_array_aliases()?;
        self.flush_alert_events();
        Ok(())
    }
}
