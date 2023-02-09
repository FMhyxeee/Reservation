CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    during TSTZRANGE,
    page integer DEFAULT 1,
    is_desc bool DEFAULT true,
    page_size integer DEFAULT 10
) RETURNS TABLE(LIKE rsvp.reservations )AS $$
DECLARE
    _sql text;
BEGIN
    -- format the query based on parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s ORDER BY lower(timespan) %s LIMIT %s OFFSET %s',
        during,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = || quote_literal(uid) AND resource_id = || quote_literal(rid)'
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

END;
$$ LANGUAGE plpgsql;
