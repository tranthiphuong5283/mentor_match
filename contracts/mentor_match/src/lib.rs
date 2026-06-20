#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol};

// -----------------------------------------------------------------------------
// Status codes returned by `get_request_status`.
// -----------------------------------------------------------------------------

/// Request is open and waiting for the mentee to select a mentor.
pub const STATUS_OPEN: u32 = 0;
/// Mentee has selected a mentor; sessions can now be recorded.
pub const STATUS_SELECTED: u32 = 1;
/// Mentee has marked the request as completed.
pub const STATUS_COMPLETED: u32 = 2;
/// Request id is unknown to this contract.
pub const STATUS_NOT_FOUND: u32 = 4;

// -----------------------------------------------------------------------------
// On-chain data structures.
// -----------------------------------------------------------------------------

/// A mentorship request posted by a mentee. Long-form goal text is *not*
/// stored on chain; only its 32-byte hash is.
#[contracttype]
#[derive(Clone)]
pub struct Request {
    pub mentee: Address,
    pub topic: Symbol,
    pub goal_hash: BytesN<32>,
    /// One of the `STATUS_*` constants above.
    pub status: u32,
    /// Number of sessions recorded against this request so far.
    pub session_count: u32,
}

/// A mentor's offer on a specific request. `message_hash` is the hash of a
/// longer pitch the mentor wants to send off-chain.
#[contracttype]
#[derive(Clone)]
pub struct Offer {
    pub mentor: Address,
    pub request_id: u64,
    pub message_hash: BytesN<32>,
}

/// One completed mentoring session, indexed by `(request_id, session_id)`.
#[contracttype]
#[derive(Clone)]
pub struct Session {
    pub mentor: Address,
    pub request_id: u64,
    pub session_hash: BytesN<32>,
    pub duration_min: u32,
    pub timestamp: u64,
}

/// A rating left by one party about the other. Stored once per
/// `(subject, rater)` pair so a single party can only rate a given address
/// once.
#[contracttype]
#[derive(Clone)]
pub struct Rating {
    pub rater: Address,
    pub subject: Address,
    pub score: u32,
    pub comment_hash: BytesN<32>,
    pub timestamp: u64,
}

// -----------------------------------------------------------------------------
// Contract
// -----------------------------------------------------------------------------

/// `MentorMatch` is a peer-mentorship coordination contract. A mentee posts
/// a request for help, mentors offer, the mentee picks one, sessions are
/// recorded, and both parties rate each other. No native asset (XLM / USDC)
/// transfer is performed by this contract; it is intentionally pure
/// coordination logic so it can be reused with any payment rail.
#[contract]
pub struct MentorMatch;

#[contractimpl]
impl MentorMatch {
    /// Mentee posts a new mentorship request. The mentee must authorize the
    /// transaction. The `request_id` must be unique within the contract and
    /// is treated as a client-supplied identifier (e.g. a UUID v4). The
    /// long-form goal description is hashed off-chain and only the
    /// 32-byte digest is stored.
    pub fn post_request(
        env: Env,
        mentee: Address,
        request_id: u64,
        topic: Symbol,
        goal_hash: BytesN<32>,
    ) {
        mentee.require_auth();

        let key = (symbol_short!("req"), request_id);
        if env.storage().instance().has(&key) {
            panic!("request id already in use");
        }

        let request = Request {
            mentee: mentee.clone(),
            topic,
            goal_hash,
            status: STATUS_OPEN,
            session_count: 0,
        };
        env.storage().instance().set(&key, &request);
    }

    /// Mentor offers to mentor on an open request. The mentor must
    /// authorize the transaction. Each `(request_id, mentor)` pair can
    /// only offer at most once, which prevents spam.
    pub fn offer_mentorship(
        env: Env,
        mentor: Address,
        request_id: u64,
        message_hash: BytesN<32>,
    ) {
        mentor.require_auth();

        let req_key = (symbol_short!("req"), request_id);
        let request: Request = env
            .storage()
            .instance()
            .get(&req_key)
            .expect("request not found");
        if request.status != STATUS_OPEN {
            panic!("request is not open");
        }

        let offer_key = (symbol_short!("offer"), request_id, mentor.clone());
        if env.storage().instance().has(&offer_key) {
            panic!("mentor has already offered on this request");
        }

        let offer = Offer {
            mentor: mentor.clone(),
            request_id,
            message_hash,
        };
        env.storage().instance().set(&offer_key, &offer);
    }

    /// Mentee selects one of the mentors that offered on their request.
    /// The request is moved from `STATUS_OPEN` to `STATUS_SELECTED` and
    /// the chosen mentor is locked in for that request. Only the original
    /// mentee can call this.
    pub fn select_mentor(
        env: Env,
        mentee: Address,
        request_id: u64,
        mentor: Address,
    ) {
        mentee.require_auth();

        let req_key = (symbol_short!("req"), request_id);
        let mut request: Request = env
            .storage()
            .instance()
            .get(&req_key)
            .expect("request not found");
        if request.mentee != mentee {
            panic!("only the original mentee can select a mentor");
        }
        if request.status != STATUS_OPEN {
            panic!("request is not open");
        }

        let offer_key = (symbol_short!("offer"), request_id, mentor.clone());
        if !env.storage().instance().has(&offer_key) {
            panic!("mentor did not offer on this request");
        }

        request.status = STATUS_SELECTED;
        env.storage().instance().set(&req_key, &request);

        let sel_key = (symbol_short!("sel"), request_id);
        env.storage().instance().set(&sel_key, &mentor);
    }

    /// Mentor records a completed session against a request that is in
    /// the `SELECTED` state. Sessions are indexed sequentially from 0 so
    /// a full, ordered history is available on chain. The session notes
    /// themselves stay off-chain; only the 32-byte hash is recorded.
    pub fn record_session(
        env: Env,
        mentor: Address,
        request_id: u64,
        session_hash: BytesN<32>,
        duration_min: u32,
    ) {
        mentor.require_auth();

        let req_key = (symbol_short!("req"), request_id);
        let mut request: Request = env
            .storage()
            .instance()
            .get(&req_key)
            .expect("request not found");
        if request.status != STATUS_SELECTED {
            panic!("request is not in the selected state");
        }

        let sel_key = (symbol_short!("sel"), request_id);
        let selected: Address = env
            .storage()
            .instance()
            .get(&sel_key)
            .expect("selected mentor is not recorded");
        if selected != mentor {
            panic!("only the selected mentor can record sessions");
        }

        let session_id = request.session_count;
        let session = Session {
            mentor: mentor.clone(),
            request_id,
            session_hash,
            duration_min,
            timestamp: env.ledger().timestamp(),
        };
        let sess_key = (symbol_short!("sess"), request_id, session_id);
        env.storage().instance().set(&sess_key, &session);

        request.session_count += 1;
        env.storage().instance().set(&req_key, &request);
    }

    /// Either party rates the other. The `mentor` parameter is the address
    /// being rated and `party` is the rater, so this single function covers
    /// "mentee rates mentor" and "mentor rates mentee". Scores must be in
    /// 1..=5 and a single rater can only rate a given subject once. No
    /// payment is involved.
    pub fn rate(
        env: Env,
        party: Address,
        mentor: Address,
        rating: u32,
        comment_hash: BytesN<32>,
    ) {
        party.require_auth();
        if rating < 1 || rating > 5 {
            panic!("rating must be between 1 and 5");
        }
        if party == mentor {
            panic!("cannot rate yourself");
        }

        let rate_key = (symbol_short!("rate"), mentor.clone(), party.clone());
        if env.storage().instance().has(&rate_key) {
            panic!("you have already rated this address");
        }

        let rating_entry = Rating {
            rater: party.clone(),
            subject: mentor.clone(),
            score: rating,
            comment_hash,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().instance().set(&rate_key, &rating_entry);
    }

    /// Returns the current status of a request:
    /// `0` = Open, `1` = Selected, `2` = Completed, `4` = Not found.
    /// Anyone can call this; it reveals no private content.
    pub fn get_request_status(env: Env, request_id: u64) -> u32 {
        let key = (symbol_short!("req"), request_id);
        env.storage()
            .instance()
            .get::<_, Request>(&key)
            .map(|r| r.status)
            .unwrap_or(STATUS_NOT_FOUND)
    }
}
