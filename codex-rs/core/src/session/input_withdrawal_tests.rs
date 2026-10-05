use super::*;
use crate::session::UserInputMetadata;
use pretty_assertions::assert_eq;

#[test]
fn withdrawal_refuses_context_owned_or_ambiguous_input_without_mutation() {
    let protected = TurnInput::UserInput {
        content: Vec::new(),
        client_id: Some("context-owned".to_string()),
        metadata: UserInputMetadata::default(),
    };
    let eligible = TurnInput::UserInput {
        content: Vec::new(),
        client_id: Some("duplicate".to_string()),
        metadata: UserInputMetadata {
            withdrawal_allowed: true,
            ..Default::default()
        },
    };
    let items = vec![protected, eligible.clone(), eligible];
    let mut queue = TurnInputQueue {
        items: items.clone(),
    };
    for id in ["context-owned", "duplicate"] {
        assert_eq!(
            queue.withdraw_user_input(id),
            InputWithdrawal::NotWithdrawable
        );
        assert_eq!(queue.items, items);
    }
}
