CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    during TSTZRANGE,
    status rsvp.reservation_status,
    page integer DEFAULT 1,
    is_desc bool DEFAULT true,
    page_size integer DEFAULT 10
) RETURNS TABLE(LIKE rsvp.reservations )AS $$
DECLARE
    _sql text;
BEGIN
    -- if page_size not between 10 and 100, set it to 10
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    IF page < 1 THEN
        page := 1;
    END IF;
    -- format the query based on parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s AND %s ORDER BY lower(timespan) %s LIMIT %L::Integer OFFSET %L::Integer',
        during,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id =' || quote_literal(uid) ||'AND resource_id =' || quote_literal(rid)
        END,
        CASE
            WHEN status IS NULL THEN 'TRUE'
            ELSE 'status = ' || quote_literal(status)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size,
        (page - 1) * page_size

    );

    -- execute the query
    RETURN QUERY EXECUTE _sql;

    -- log the sql
    RAISE NOTICE '%', _sql;

END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION rsvp.filter(
    uid text,
    rid text,
    status rsvp.reservation_status,
    cursor bigint DEFAULT NULL,
    is_desc bool DEFAULT FALSE,
    page_size integer DEFAULT 10
) RETURNS TABLE(LIKE rsvp.reservations )AS $$
DECLARE
    _sql text;
    _offset bigint;
BEGIN

    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    -- if the cursor is null or less than 0, set it to 1
    -- if is_desc is true, else set it to max int

    IF cursor IS NULL OR cursor < 0 THEN
        IF is_desc THEN
            cursor := 9223372036854775807;
        ELSE
            cursor := 0;
        END IF;
    END IF;
    -- format the query based on parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %s AND status = %L AND %s ORDER BY id %s LIMIT %L::Integer',
        CASE
            WHEN is_desc THEN 'id <= ' || quote_literal(cursor)
            ELSE 'id >= ' || quote_literal(cursor)
        END,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id =' || quote_literal(uid) ||'AND resource_id =' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size + 1
    );

    -- execute the query
    RETURN QUERY EXECUTE _sql;

    -- log the sql
    RAISE NOTICE '%', _sql;

END;
$$ LANGUAGE plpgsql;
