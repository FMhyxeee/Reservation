# Core Reservation
- Feature Name: core-reservation
- Start Date: 2022年12月14日22:09:06

## Summary
A core reservation system that solves the problem of resource reserving a resource for a period of time. We laverage postgres EXCLUSIVE constraint to ensure the resource is not reserved by multiple users at the same time.

# Motivation
we need a common solution for various reservation requirements, such as: 1) calendar booking. 2) hotel/book room reservation. 3) meeting room reservation. 4) resource reservation. 5) etc. Repeatedly implement the same logic is not a good idea, so we need a common solution.

# Guide-level explanation

## Service interface

we would use gRPC as the service interface, below is the proto file:
```proto

    enum ReservationStatus {
        UNKNOWN = 0;
        PENDING = 1;
        CONFIRMED = 2;
        BLOCKED = 3;
    }

    enum ReservationUpdateType {
        UNKNOWN = 0;
        INSERT = 1;
        UPDATE = 2;
        DELETE = 3;
    }

    message Reservation {
        string id = 1;
        string user_id = 2;
        ReservationStatus status = 3;

        // resource reservation window
        string source_id = 4;
        google.protobuf.Timestamp start = 5;
        google.protobuf.Timestamp end = 6;

        // extra note
        string note = 7;

    }

    service ReservationService {
        rpc reserve(ReserveRequest) returns (ReserveResponse);
        rpc confirm(ConfirmRequest) returns (ReserveResponse);
        rpc update(UpdateRequest) returns (UpdateResponse);
        rpc cancel(CancelRequest) returns (CancelResponse);
        rpc get(GetRequest) returns (GetResponse);
        rpc query(QueryRequest) returns (stream Reservation);
        rpc listen(ListenRequest) returns (stream Reservation);
    }

    message ReserveRequest {
        Reservation reservation = 1;
    }

    message ReserveResponse {
        Reservation reservation = 1;
    }

    message UpdateRequest {
        string note = 1;
    }

    message UpdateResponse {
        Reservation reservation = 1;
    }

    message CancelRequest {
        string id = 1;
    }

    message CancelResponse {
        Reservation reservation = 1;
    }

    message GetRequest {
        string id = 1;
    }

    message GetResponse {
        Reservation reservation = 1;
    }

    message QueryRequest {
        string source_id = 1;
        string user_id= 2;

        // use status to filter result, IF UNKNOW, return all reservations
        ReservationStatus status = 3;
        google.protobuf.Timestamp start = 4;
        google.protobuf.Timestamp end = 5;
    }

    // No QueryResponse, use stream instead

    message ListenRequest {}
    message ListenResponse {
        ReservationOp op = 1;
        Reservation reservation = 2;
    }




```

## Database Schema

```sql
    CREATE SCHEMA rsvp;
    CREATE TYPE rsvp.reservation_status AS ENUM ('unknown','pending','confirmed','blocked');
    CREATE TYPE rsvp.reservation_update_type as ENUM ('unknown','insert','update','delete');
    CREATE TABLE rsvp.reservation (
        id uuid NOT NULL DEFAULT uuid_generate_v4(),
        user_id varchar(64) NOT NULL,
        status rsvp.reservation_status NOT NULL default 'pending',

        resource_id varchar(64) NOT NULL,
        timespan tstzrange NOT NULL,


        not TEXT,

        CONSTRAINT reservation_pkey PRIMARY KEY (id),
        CONSTRAINT reservation_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
    );

    CREATE INDEX reservation_resource_id_idx ON rsvp.reservation (resource_id);
    CREATE INDEX reservation_user_id_idx ON rsvp.reservation (user_id);


    -- reservation change queue
    CREATE TABLE rsvp.reservation_change (
        id SERIAL NOT NULL,
        reservation_id uuid NOT NULL,
        op rsvp.reservation_update_type NOT NULL,
    );

    -- if user_id is null, find all reservations within during for the resource
    -- if resource_id is null, find all reservations within during for the user
    -- if both are null, find all reservations within during
    -- if both set, find all reservations within during for the resource and user
    CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during:tstzrange)
    RETURNS TABLE resp.reservation AS $$ $$ LANGUAGE plgsql;

    CREATE OR REPLACE FUNCTION resp.reservation_trigger() RETURNS trigger AS 
    $$ 
    BEGIN
        IF TG_OP = "INSERT" THEN
            -- update reservation_changes
            INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'create');

        ELSEIF TG_OP = "UPDATE" THEN
            -- if status changed, update reservation_changes
            IF OLD.status <> NEW.status THEN
                INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'update');
            END IF;
        ELSEIF TG_OP = "DELETE" THEN
            -- update reservation_changes
            INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (OLD.id, 'delete');

        END IF;

        NOTIFY reservation_update;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;
    CREATE TRIGGER reservation_trigger 
        AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservation
        FOR EACH ROW EXECUTE PROCEDURE resp.reservation_trigger();

```

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

# Drawbacks
[drawbacks]: #drawbacks

Why should we *not* do this?

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

# Prior art
[prior-art]: #prior-art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- For language, library, cargo, tools, and compiler proposals: Does this feature exist in other programming languages and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to, this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other languages, provide readers of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an adaptation from other languages.

Note that while precedent set by other languages is some motivation, it does not on its own motivate an RFC.
Please also take into consideration that rust sometimes intentionally diverges from common language features.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?

# Future possibilities
[future-possibilities]: #future-possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.
