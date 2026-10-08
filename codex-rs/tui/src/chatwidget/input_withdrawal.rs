//! Restore a server-owned prompt only after confirmed withdrawal.

use super::*;

impl ChatWidget {
    pub(crate) fn composer_is_empty_for_recall(&self) -> bool {
        let draft = self.bottom_pane.composer_draft_snapshot();
        draft.text.is_empty()
            && draft.text_elements.is_empty()
            && draft.local_images.is_empty()
            && draft.remote_image_urls.is_empty()
            && draft.mention_bindings.is_empty()
            && draft.pending_pastes.is_empty()
    }

    pub(super) fn request_pending_steer_recall(&mut self) {
        if !self.composer_is_empty_for_recall() {
            self.add_warning_message(
                "Clear the input field before recalling a sent message.".to_string(),
            );
            return;
        }
        let Some(pending) = self
            .input_queue
            .pending_steers
            .iter()
            .filter(|pending| pending.source == UserMessageSource::Prompt)
            .max_by_key(|pending| pending.recall_order)
        else {
            return;
        };
        let Some(turn_id) = pending.accepted_turn_id.clone() else {
            self.add_warning_message("Message submission is still being confirmed.".to_string());
            return;
        };
        let Some(thread_id) = self.thread_id else {
            return;
        };
        self.submit_op(AppCommand::WithdrawSteer {
            thread_id,
            turn_id,
            client_id: pending.client_id.clone(),
        });
    }

    pub(crate) fn acknowledge_pending_steer(&mut self, client_id: &str, turn_id: String) {
        if let Some(pending) = self
            .input_queue
            .pending_steers
            .iter_mut()
            .find(|pending| pending.client_id == client_id)
        {
            pending.accepted_turn_id = Some(turn_id);
        }
    }

    pub(crate) fn on_pending_steer_withdrawn(&mut self, client_id: &str) {
        let Some(index) = self
            .input_queue
            .pending_steers
            .iter()
            .position(|pending| pending.client_id == client_id)
        else {
            return;
        };
        let pending = self
            .input_queue
            .pending_steers
            .remove(index)
            .expect("located pending input");
        self.restore_user_message_to_composer(user_message_for_restore(
            pending.user_message,
            &pending.history_record,
        ));
        self.refresh_startup_recovery();
        self.refresh_pending_input_preview();
        self.request_redraw();
    }
}
