# mentor_match

## Project Title
mentor_match

## Project Description
mentor_match is a peer-mentorship coordination dApp built on the Stellar
blockchain using Soroban smart contracts. It helps a mentee find an
experienced mentor for a specific learning goal, lets mentors volunteer
their time, records every session on chain, and lets both sides rate each
other. Sensitive text (goal descriptions, offer messages, session notes,
rating comments) is kept off-chain; only the 32-byte hash of that text is
written to the ledger, so participants can prove what was said without
exposing the full content on a public chain.

The project tackles a real problem: in universities, bootcamps, and
open-source communities, mentorship usually happens in group chats where
there is no shared, tamper-proof record of who helped whom, on what topic,
and how it went. mentor_match gives every request, offer, session, and
rating a verifiable on-chain trail that anyone can audit, and it does so
without ever holding the participants' funds.

## Project Vision
Our vision is to make learning mentorship as open, transparent, and
portable as a public-good service. By anchoring each step of a
mentor-mentee relationship on Stellar, we want to enable:

- Portable reputation for mentors across cohorts, courses, and platforms.
- Verifiable session histories for students who need to show evidence of
  learning (e.g. scholarship applications, university portfolios, job
  interviews).
- Communities that can run mentorship programs without a central
  operator having to host a database or trust a single admin.

In the long term, mentor_match aims to become a lightweight trust layer
for any peer-to-peer learning community, from a local coding club to a
global accelerator.

## Key Features
1. **Request Posting** - A mentee publishes a request with a topic and
   the hash of a longer goal description. The mentee's signature is
   required via `require_auth`, so no one can post a request on their
   behalf.
2. **Mentor Offers** - Any Stellar address can offer to mentor on an
   open request by submitting a hashed message. Each
   `(request, mentor)` pair can offer at most once, which prevents spam.
3. **Mentor Selection** - The mentee reviews the offers on chain and
   picks one. The request transitions from `Open` to `Selected` and the
   chosen mentor is locked in for the rest of the engagement.
4. **Session Recording** - The selected mentor logs every completed
   session with a duration and the hash of session notes, building an
   immutable, ordered history of work done.
5. **Mutual Ratings** - Both the mentee and the mentor can leave a
   1-to-5 star rating and a hashed comment about the other party, with
   each rater allowed to rate a given subject only once.
6. **Public Status View** - Anyone can call `get_request_status` to
   inspect the current state of any request (`0` = Open, `1` = Selected,
   `2` = Completed) without revealing any private content.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** work dApp — see `contracts/mentor_match/src/lib.rs` for the full mentor_match business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<CCVQJSFLSGEFEO5HKSIRIOIJWLNZMK3YYIOSLFN45ARU27UW6DI7KXIA>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/a4423a44ed67d6706e6084c82ce17ecc61defee5d1fda096ffca02981a3cc023>`


## Future Scope
- **Endorsement tokens** - mint a non-transferable, soulbound token for
  mentors that summarizes their cumulative rating, which they can show in
  any Stellar wallet.
- **Stable-coin payments** - integrate a USDC-based escrow so mentees can
  pre-fund sessions and the mentor is paid automatically once a session
  is recorded.
- **Topic registry** - an on-chain allowlist of topics so communities
  can run curated mentorship tracks (e.g. "Rust", "Machine Learning",
  "Public Speaking").
- **Dispute resolution** - an optional arbitrator address that can void
  a rating or a session record if both parties agree to a rollback.
- **Frontend dApp** - a React/Next.js UI using Freighter to browse
  requests, submit offers, and view a mentor's public history.
- **Cross-chain reputation** - bridge aggregated mentor scores to other
  networks so reputation is portable across the wider Web3 ecosystem.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `mentor_match` (work)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
