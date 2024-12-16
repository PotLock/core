use crate::*;

pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";
/// source metadata update
pub(crate) fn log_election_created_event(election: Election, candidates: &Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "election_created",
                "data": [
                    {
                        "election": election,
                        "candidates": candidates,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

pub(crate) fn log_vote_event(election_id: ElectionId, vote: (AccountId, u32)) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "vote",
                "data": [
                    {
                        "election_id": election_id,
                        "vote": vote,
                    }
                ]
            })
        )
        .as_ref(),
    );
}
