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
