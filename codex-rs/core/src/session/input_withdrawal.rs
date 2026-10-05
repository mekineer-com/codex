//! Withdraws only input still owned by the active turn's queue.

use super::TurnInput;
use super::TurnInputQueue;
use super::session::Session;
use crate::codex_thread::CodexThread;
use crate::state::TaskKind;

#[cfg(test)]
#[path = "input_withdrawal_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputWithdrawal {
    Withdrawn,
    NotPending,
    TurnChanged,
    NotWithdrawable,
}

impl CodexThread {
    /// Withdraws one pending user message without interrupting the active task.
    /// Already-applied settings and acceptance telemetry are not rolled back.
    pub async fn withdraw_pending_input(
        &self,
        expected_turn_id: &str,
        client_id: &str,
    ) -> InputWithdrawal {
        self.session
            .withdraw_pending_input(expected_turn_id, client_id)
            .await
    }
}

impl Session {
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "turn identity and pending input removal must remain atomic"
    )]
    pub(crate) async fn withdraw_pending_input(
        &self,
        expected_turn_id: &str,
        client_id: &str,
    ) -> InputWithdrawal {
        let active = self.active_turn.lock().await;
        let Some(turn) = active.as_ref() else {
            return InputWithdrawal::TurnChanged;
        };
        let Some(task) = turn.task.as_ref() else {
            return InputWithdrawal::TurnChanged;
        };
        if task.turn_context.sub_id != expected_turn_id || task.cancellation_token.is_cancelled() {
            return InputWithdrawal::TurnChanged;
        }
        if task.kind != TaskKind::Regular || client_id.is_empty() {
            return InputWithdrawal::NotWithdrawable;
        }
        let mut state = turn.turn_state.lock().await;
        state.pending_input.withdraw_user_input(client_id)
    }
}

impl TurnInputQueue {
    fn withdraw_user_input(&mut self, client_id: &str) -> InputWithdrawal {
        let mut matches = self.items.iter().enumerate().filter(|(_, input)| {
            matches!(input, TurnInput::UserInput { client_id: Some(id), .. } if id == client_id)
        });
        let Some((index, TurnInput::UserInput { metadata, .. })) = matches.next() else {
            return InputWithdrawal::NotPending;
        };
        if !metadata.withdrawal_allowed || matches.next().is_some() {
            return InputWithdrawal::NotWithdrawable;
        }
        self.items.remove(index);
        InputWithdrawal::Withdrawn
    }
}
