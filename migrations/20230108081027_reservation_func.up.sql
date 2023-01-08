CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE(LIKE rsvp.reservations )
AS $$
BEGIN
    IF UID IS NULL AND RID IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE timespan && during;
    ELSEIF UID IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
    ELSEIF RID IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timespan;
    ELSE
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND resource_id = rid AND during @> timespan;
    END IF;
END;
$$ LANGUAGE plpgsql;
